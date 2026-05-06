use crate::Whisper;

pub struct WhisperFilter {
    default_ttl: u64,
}

impl WhisperFilter {
    pub fn new() -> Self {
        Self { default_ttl: 1800 } // 30 minutes
    }

    /// Check if a whisper is relevant to a given agent
    pub fn is_relevant(&self, whisper: &Whisper, agent_id: &str) -> bool {
        // Must not be expired
        if whisper.is_expired() {
            return false;
        }

        // Check unicast: must be addressed to this agent (or be a broadcast/multicast)
        if let Some(ref to) = whisper.to {
            return to == agent_id;
        }

        // No `to` field means broadcast/multicast — relevant to all
        true
    }

    /// Check if whisper matches a specific type
    pub fn matches_type(&self, whisper: &Whisper, type_name: &str) -> bool {
        whisper.whisper_type.variant_name() == type_name
    }

    /// Check if whisper has a specific tag
    pub fn has_tag(&self, whisper: &Whisper, tag: &str) -> bool {
        whisper.tags().iter().any(|t| t == tag)
    }
}

impl Default for WhisperFilter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::WhisperType;

    #[test]
    fn test_is_relevant_unicast_to_me() {
        let filter = WhisperFilter::new();
        let wt = WhisperType::Status { health: 0.9, load: 0.2 };
        let w = Whisper::new("agent1".to_string(), Some("agent2".to_string()), wt, 1800);
        assert!(filter.is_relevant(&w, "agent2"));
    }

    #[test]
    fn test_is_relevant_unicast_not_me() {
        let filter = WhisperFilter::new();
        let wt = WhisperType::Status { health: 0.9, load: 0.2 };
        let w = Whisper::new("agent1".to_string(), Some("agent2".to_string()), wt, 1800);
        assert!(!filter.is_relevant(&w, "agent3"));
    }

    #[test]
    fn test_is_relevant_broadcast() {
        let filter = WhisperFilter::new();
        let wt = WhisperType::Alert { severity: "warn".to_string(), message: "test".to_string() };
        let w = Whisper::new("agent1".to_string(), None, wt, 1800);
        // No `to` field = broadcast, relevant to all
        assert!(filter.is_relevant(&w, "agent3"));
    }

    #[test]
    fn test_is_relevant_expired() {
        use chrono::{Duration, Utc};
        let filter = WhisperFilter::new();
        let wt = WhisperType::Status { health: 0.9, load: 0.2 };
        let mut w = Whisper::new("agent1".to_string(), Some("agent2".to_string()), wt, 60);
        w.timestamp = Utc::now() - Duration::minutes(5);
        assert!(!filter.is_relevant(&w, "agent2"));
    }

    #[test]
    fn test_matches_type() {
        let filter = WhisperFilter::new();
        let wt = WhisperType::Discovery { service: "svc".to_string(), endpoint: "ep".to_string() };
        let w = Whisper::new("a".to_string(), None, wt, 1800);
        assert!(filter.matches_type(&w, "discovery"));
        assert!(!filter.matches_type(&w, "status"));
    }
}
