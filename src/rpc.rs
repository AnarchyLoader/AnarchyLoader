use std::sync::{Arc, Mutex};

use discord_rich_presence::{activity, DiscordIpc, DiscordIpcClient};
pub struct Rpc {
    pub state: String,
    pub details: String,
    pub client: Option<Arc<Mutex<DiscordIpcClient>>>,
}

impl Default for Rpc {
    fn default() -> Self {
        Rpc {
            state: format!("v{}", env!("CARGO_PKG_VERSION")),
            details: "Selecting a hack".to_string(),
            client: None,
        }
    }
}

impl Rpc {
    pub fn start(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let mut client = DiscordIpcClient::new("1317814620152528948")?;
        client.connect()?;
        self.client = Some(Arc::new(Mutex::new(client)));
        Ok(())
    }

    pub fn update(&mut self) {
        if let Some(client) = &self.client {
            let mut client = client.lock().unwrap();
            client
                .set_activity(
                    activity::Activity::new()
                        .state(&self.state)
                        .details(&self.details),
                )
                .unwrap();
        }
    }

    pub fn close(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(client) = self.client.take() {
            let mut client = client.lock().unwrap();
            client.close()?;
        }
        Ok(())
    }
}

impl Drop for Rpc {
    fn drop(&mut self) {
        let _ = self.close();
    }
}
