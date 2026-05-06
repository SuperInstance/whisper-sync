use crate::{PlatoTransport, Whisper, WhisperType};
use crate::filter::WhisperFilter;
use std::sync::Arc;
use tokio::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

pub struct Inbox {
    transport: PlatoTransport,
    filter: WhisperFilter,
    last_polled: Arc<Mutex<u64>>,
    agent_id: String,
}

impl Inbox {
    pub fn new(transport: PlatoTransport, agent_id: String) -> Self {
        Self {
            transport,
            filter: WhisperFilter::new(),
            last_polled: Arc::new(Mutex::new(0)),
            agent_id,
        }
    }

    /// Poll for new whispers addressed to this agent
    pub async fn check(&self) -> Result<Vec<Whisper>, crate::plato_transport::PlatoError> {
        let since = {
            let last = self.last_polled.lock().await;
            *last
        };

        let since_str = if since == 0 {
            "0".to_string()
        } else {
            since.to_string()
        };

        let tiles = self.transport.poll_whispers(&since_str, Some("whisper")).await?;

        let mut whispers = Vec::new();
        for tile in tiles {
            if let Ok(w) = serde_json::from_str::<Whisper>(&tile.content) {
                if self.filter.is_relevant(&w, &self.agent_id) {
                    whispers.push(w);
                }
            }
        }

        // Update last_polled to now
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        *self.last_polled.lock().await = now;

        Ok(whispers)
    }

    /// Start polling loop with given interval (seconds)
    pub async fn listen<F, Fut>(&self, interval_secs: u64, mut callback: F)
    where
        F: FnMut(Vec<Whisper>) -> Fut,
        Fut: std::future::Future<Output = ()>,
    {
        loop {
            match self.check().await {
                Ok(whispers) => {
                    if !whispers.is_empty() {
                        callback(whispers).await;
                    }
                }
                Err(e) => {
                    tracing::warn!("inbox poll error: {}", e);
                }
            }
            tokio::time::sleep(tokio::time::Duration::from_secs(interval_secs)).await;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inbox_creation() {
        let pt = PlatoTransport::new();
        let inbox = Inbox::new(pt, "test-agent".to_string());
        assert_eq!(inbox.agent_id, "test-agent");
    }
}
