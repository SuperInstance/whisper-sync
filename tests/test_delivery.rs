use whisper_sync::{Whisper, WhisperType, WhisperFilter, DeliveryMode};
use crate::WhisperFilter;

#[test]
fn test_delivery_mode_enum() {
    let _unicast = DeliveryMode::Unicast;
    let _multicast = DeliveryMode::Multicast;
    let _broadcast = DeliveryMode::Broadcast;
}

#[test]
fn test_filter_unicast_to_me() {
    let filter = WhisperFilter::new();
    let wt = WhisperType::Status { health: 0.9, load: 0.2 };
    let w = Whisper::new("agent1".to_string(), Some("agent2".to_string()), wt, 1800);
    assert!(filter.is_relevant(&w, "agent2"));
}

#[test]
fn test_filter_unicast_not_me() {
    let filter = WhisperFilter::new();
    let wt = WhisperType::Status { health: 0.9, load: 0.2 };
    let w = Whisper::new("agent1".to_string(), Some("agent2".to_string()), wt, 1800);
    assert!(!filter.is_relevant(&w, "agent3"));
}

#[test]
fn test_filter_broadcast() {
    let filter = WhisperFilter::new();
    let wt = WhisperType::Alert { severity: "warn".into(), message: "test".into() };
    let w = Whisper::new("agent1".to_string(), None, wt, 1800);
    // No `to` field = broadcast
    assert!(filter.is_relevant(&w, "agent3"));
}

#[test]
fn test_filter_matches_type() {
    let filter = WhisperFilter::new();
    let wt = WhisperType::Discovery { service: "svc".into(), endpoint: "ep".into() };
    let w = Whisper::new("a".to_string(), None, wt, 1800);
    assert!(filter.matches_type(&w, "discovery"));
    assert!(!filter.matches_type(&w, "status"));
}
