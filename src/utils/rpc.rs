use std::{sync::mpsc, thread};

use discord_rich_presence::{
    activity::{self, Assets},
    DiscordIpc, DiscordIpcClient,
};

pub struct Rpc {
    pub sender: mpsc::Sender<RpcUpdate>,
    enabled: bool,
}

pub enum RpcUpdate {
    Update {
        state: Option<String>,
        details: Option<String>,
        small_image: Option<String>,
    },
    Shutdown,
}

impl Rpc {
    pub fn new(enabled: bool) -> Self {
        let (tx, rx) = mpsc::channel();

        if enabled {
            thread::spawn(move || {
                let mut client = match DiscordIpcClient::new("1317814620152528948") {
                    Ok(mut c) => match c.connect() {
                        Ok(_) => Some(c),
                        Err(e) => {
                            log::error!("Failed to connect to Discord RPC: {}", e);
                            None
                        }
                    },
                    Err(e) => {
                        log::error!("Failed to create Discord RPC client: {}", e);
                        None
                    }
                };

                let mut current_state = String::new();
                let mut current_details = String::new();
                let mut current_small_image = None;

                loop {
                    match rx.recv() {
                        Ok(RpcUpdate::Update {
                            state,
                            details,
                            small_image,
                        }) => {
                            if let Some(s) = state {
                                current_state = s;
                            }
                            if let Some(d) = details {
                                current_details = d;
                            }
                            if let Some(i) = small_image {
                                current_small_image = Some(i);
                            }

                            if let Some(c) = &mut client {
                                let mut activity = activity::Activity::new()
                                    .state(&current_state)
                                    .details(&current_details)
                                    .assets(activity::Assets::new().large_image("logo"));

                                if let Some(ref image) = current_small_image {
                                    activity = activity.assets(
                                        Assets::new().large_image("logo").small_image(image),
                                    );
                                }

                                if let Err(e) = c.set_activity(activity) {
                                    log::error!("Failed to set Discord RPC activity: {}", e);
                                }
                            }
                        }
                        Ok(RpcUpdate::Shutdown) => break,
                        Err(e) => {
                            log::error!("RPC channel error: {}", e);
                            break;
                        }
                    }
                }

                if let Some(mut c) = client {
                    if let Err(e) = c.close() {
                        log::error!("Failed to close Discord RPC connection: {}", e);
                    }
                }
            });
        }

        Rpc {
            sender: tx,
            enabled,
        }
    }

    pub fn update(&self, state: Option<&str>, details: Option<&str>, small_image: Option<&str>) {
        if self.enabled {
            log::debug!(
                "Updating RPC: state: {:?}, details: {:?}, small_image: {:?}",
                state,
                details,
                small_image
            );
            let update = RpcUpdate::Update {
                state: state.map(|s| s.to_string()),
                details: details.map(|d| d.to_string()),
                small_image: small_image.map(|i| i.to_string()),
            };

            if let Err(e) = self.sender.send(update) {
                log::error!("Failed to send RPC update: {}", e);
            }
        }
    }
}
