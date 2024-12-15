use std::{sync::mpsc, thread};

use discord_rich_presence::{activity, DiscordIpc, DiscordIpcClient};

pub struct Rpc {
    sender: mpsc::Sender<RpcUpdate>,
}

pub enum RpcUpdate {
    Update {
        state: Option<String>,
        details: Option<String>,
    },
    Shutdown,
}

impl Rpc {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel();

        thread::spawn(move || {
            let mut client = match DiscordIpcClient::new("1317814620152528948") {
                Ok(mut c) => match c.connect() {
                    Ok(_) => Some(c),
                    Err(e) => {
                        eprintln!("Failed to connect to Discord RPC: {}", e);
                        None
                    }
                },
                Err(e) => {
                    eprintln!("Failed to create Discord RPC client: {}", e);
                    None
                }
            };

            let mut current_state = String::new();
            let mut current_details = String::new();

            loop {
                match rx.recv() {
                    Ok(RpcUpdate::Update { state, details }) => {
                        if let Some(s) = state {
                            current_state = s;
                        }
                        if let Some(d) = details {
                            current_details = d;
                        }
                        if let Some(c) = &mut client {
                            if let Err(e) = c.set_activity(
                                activity::Activity::new()
                                    .state(&current_state)
                                    .details(&current_details),
                            ) {
                                eprintln!("Failed to set Discord RPC activity: {}", e);
                            }
                        }
                    }
                    Ok(RpcUpdate::Shutdown) => break,
                    Err(e) => {
                        eprintln!("RPC channel error: {}", e);
                        break;
                    }
                }
            }

            if let Some(mut c) = client {
                if let Err(e) = c.close() {
                    eprintln!("Failed to close Discord RPC connection: {}", e);
                }
            }
        });

        Rpc { sender: tx }
    }

    pub fn update(&self, state: Option<&str>, details: Option<&str>) {
        let update = RpcUpdate::Update {
            state: state.map(|s| s.to_string()),
            details: details.map(|d| d.to_string()),
        };

        if let Err(e) = self.sender.send(update) {
            eprintln!("Failed to send RPC update: {}", e);
        }
    }
}

impl Drop for Rpc {
    fn drop(&mut self) {
        let _ = self.sender.send(RpcUpdate::Shutdown);
    }
}
