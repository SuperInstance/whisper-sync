use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Whisper type variants
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "variant")]
pub enum WhisperType {
    Status { health: f64, load: f64 },
    Discovery { service: String, endpoint: String },
    Help { question: String, tags: Vec<String> },
    Insight { summary: String, confidence: f64, source_theorem: Option<String> },
    Trust { agent: String, trust_score: f64 },
    Alert { severity: String, message: String },
}

impl WhisperType {
    pub fn variant_name(&self) -> &'static str {
        match self {
            WhisperType::Status { .. } => "status",
            WhisperType::Discovery { .. } => "discovery",
            WhisperType::Help { .. } => "help",
            WhisperType::Insight { .. } => "insight",
            WhisperType::Trust { .. } => "trust",
            WhisperType::Alert { .. } => "alert",
        }
    }
}

/// A whisper message between fleet agents
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Whisper {
    pub id: String,
    pub from: String,
    pub to: Option<String>,
    #[serde(flatten)]
    pub whisper_type: WhisperType,
    pub timestamp: DateTime<Utc>,
    pub ttl_seconds: u64,
}

impl Whisper {
    pub fn new(from: String, to: Option<String>, whisper_type: WhisperType, ttl_seconds: u64) -> Self {
        Self {
            id: uuid_v4(),
            from,
            to,
            whisper_type,
            timestamp: Utc::now(),
            ttl_seconds,
        }
    }

    pub fn is_expired(&self) -> bool {
        let age = Utc::now().signed_duration_since(self.timestamp);
        age.num_seconds() as u64 > self.ttl_seconds
    }

    pub fn tags(&self) -> Vec<String> {
        let mut tags = vec![
            "whisper".to_string(),
            format!("type:{}", self.whisper_type.variant_name()),
        ];
        if self.to.is_some() {
            tags.push("unicast".to_string());
        }
        tags
    }
}

/// Generate a simple UUID v4 (minimal impl)
fn uuid_v4() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let random: u64 = (now * 0x5deace66d) as u64;
    format!("{:016x}-{:04x}-4{:03x}-{:04x}-{:012x}",
            now as u64,
            (random >> 48) as u16,
            (random >> 32) as u16 & 0x0fff,
            ((random >> 16) as u16 & 0x4000) | 0xa000,
            random & 0xffffffffffff)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_whisper_creation() {
        let wt = WhisperType::Status { health: 0.95, load: 0.3 };
        let w = Whisper::new("agent1".to_string(), Some("agent2".to_string()), wt, 1800);
        assert_eq!(w.from, "agent1");
        assert_eq!(w.to, Some("agent2".to_string()));
        assert!(!w.is_expired());
    }

    #[test]
    fn test_whisper_expired() {
        use chrono::Duration;
        let mut w = Whisper::new(
            "agent1".to_string(),
            None,
            WhisperType::Alert { severity: "info".to_string(), message: "test".to_string() },
            60,
        );
        w.timestamp = Utc::now() - Duration::minutes(5);
        assert!(w.is_expired());
    }

    #[test]
    fn test_tags() {
        let wt = WhisperType::Discovery { service: "test".to_string(), endpoint: "http://test".to_string() };
        let w = Whisper::new("a".to_string(), None, wt, 1800);
        let tags = w.tags();
        assert!(tags.contains(&"whisper".to_string()));
        assert!(tags.contains(&"type:discovery".to_string()));
    }
}
