use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

const PLATO_BASE: &str = "http://localhost:8847";

#[derive(Debug, Serialize)]
struct WhisperTile {
    tags: Vec<String>,
    content: String,
    to: Option<String>,
}

impl WhisperTile {
    fn from_whisper(w: &crate::Whisper) -> Self {
        let content = serde_json::to_string(w).unwrap_or_default();
        Self {
            tags: w.tags(),
            content,
            to: w.to.clone(),
        }
    }
}

pub struct PlatoTransport {
    client: Client,
    base_url: String,
}

impl PlatoTransport {
    pub fn new() -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .expect("PlatoTransport client build");
        Self {
            client,
            base_url: PLATO_BASE.to_string(),
        }
    }

    pub fn new_with_base(base_url: &str) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .expect("PlatoTransport client build");
        Self {
            client,
            base_url: base_url.to_string(),
        }
    }

    /// Submit a whisper to the fleet_whispers room
    pub async fn submit_whisper(&self, whisper: &crate::Whisper) -> Result<(), PlatoError> {
        let tile = WhisperTile::from_whisper(whisper);
        let url = format!("{}/room/fleet_whispers/submit", self.base_url);

        let resp = self.client
            .post(&url)
            .json(&tile)
            .send()
            .await
            .map_err(PlatoError::Request)?;

        if resp.status().is_success() {
            Ok(())
        } else {
            Err(PlatoError::Server(resp.status().to_string()))
        }
    }

    /// Poll whispers from the fleet_whispers room since a given timestamp
    pub async fn poll_whispers(&self, since: &str, tag: Option<&str>) -> Result<Vec<PlatoTile>, PlatoError> {
        let url = if let Some(t) = tag {
            format!("{}/room/fleet_whispers/tiles?since={}&tag={}", self.base_url, since, t)
        } else {
            format!("{}/room/fleet_whispers/tiles?since={}&tag=whisper", self.base_url, since)
        };

        let resp = self.client
            .get(&url)
            .send()
            .await
            .map_err(PlatoError::Request)?;

        if !resp.status().is_success() {
            return Err(PlatoError::Server(resp.status().to_string()));
        }

        let tiles: Vec<PlatoTile> = resp.json().await.map_err(|e| PlatoError::Parse(e.to_string()))?;
        Ok(tiles)
    }
}

#[derive(Debug, Deserialize)]
pub struct PlatoTile {
    pub id: String,
    pub tags: Vec<String>,
    pub content: String,
    #[serde(default)]
    pub created_at: Option<String>,
}

#[derive(Debug)]
pub enum PlatoError {
    Request(reqwest::Error),
    Server(String),
    Parse(String),
}

impl std::fmt::Display for PlatoError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PlatoError::Request(e) => write!(f, "request error: {}", e),
            PlatoError::Server(s) => write!(f, "server error: {}", s),
            PlatoError::Parse(e) => write!(f, "parse error: {}", e),
        }
    }
}

impl std::error::Error for PlatoError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plato_transport_creation() {
        let pt = PlatoTransport::new();
        assert_eq!(pt.base_url, PLATO_BASE);
    }

    #[test]
    fn test_plato_transport_custom_base() {
        let pt = PlatoTransport::new_with_base("http://localhost:9000");
        assert_eq!(pt.base_url, "http://localhost:9000");
    }
}
