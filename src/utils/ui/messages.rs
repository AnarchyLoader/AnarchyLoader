use std::{
    sync::mpsc::{self, TryRecvError},
    time::Duration,
};

use crate::MyApp;

#[derive(Debug)]
pub struct ToastsMessages {
    pub sender: mpsc::Sender<String>,
    pub receiver: mpsc::Receiver<String>,
}

impl ToastsMessages {
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::channel();
        ToastsMessages { sender, receiver }
    }
}

pub trait MessageSender {
    fn raw(&self, message: &str);
    fn success(&self, message: &str);
    fn error(&self, message: &str);
}

impl MessageSender for mpsc::Sender<String> {
    fn raw(&self, message: &str) {
        self.send(message.to_string()).unwrap();
    }

    fn success(&self, message: &str) {
        self.send(format!("SUCCESS: {}", message)).unwrap();
    }

    fn error(&self, message: &str) {
        self.send(format!("ERROR: {}", message)).unwrap();
    }
}

impl MyApp {
    pub(crate) fn update_rpc_status_selecting(&mut self, ctx: &egui::Context) {
        let version = format!("v{}", env!("CARGO_PKG_VERSION"));
        let status = if let Some(hack) = &self.app.selected_hack {
            format!("Selected {}", hack.name)
        } else {
            "Selecting hack".to_string()
        };
        log::debug!(
            "<MAIN> Updating RPC status to: version={}, status={}",
            version,
            status
        );
        self.rpc.update(Some(&version), Some(&status), Some("home"));

        ctx.send_viewport_cmd(egui::ViewportCommand::EnableButtons {
            close: true,
            minimized: true,
            maximize: true,
        });
    }

    pub fn handle_received_messages(&mut self, ctx: &egui::Context) {
        match self.communication.messages.receiver.try_recv() {
            Ok(message) => {
                if message.starts_with("SUCCESS: ") {
                    self.handle_successful_injection_message(message.clone());
                    self.update_rpc_status_selecting(ctx);
                } else if message.starts_with("ERROR: ") {
                    self.handle_error_message(message.clone());
                    self.update_rpc_status_selecting(ctx);
                } else {
                    self.handle_raw_message(message.clone());
                }
            }
            Err(TryRecvError::Empty) => {}
            Err(e) => {
                log::error!("[MESSAGES] Error receiving from channel: {:?}", e);
            }
        }
    }

    fn handle_raw_message(&mut self, message: String) {
        let message = message.trim_start_matches("RAW: ");
        self.toasts.info(message);
    }

    fn handle_successful_injection_message(&mut self, message: String) {
        let name = message.trim_start_matches("SUCCESS: ").to_string();
        self.toasts
            .success(format!("Successfully injected {}", name))
            .duration(Some(Duration::from_secs(4)));

        if !name.starts_with("steam") {
            self.app.stats.increment_inject_count(&name);
        }
    }

    fn handle_error_message(&mut self, message: String) {
        let error = message.trim_start_matches("ERROR: ").to_string();
        self.toasts
            .error(error)
            .duration(Some(Duration::from_secs(4)));
    }
}
