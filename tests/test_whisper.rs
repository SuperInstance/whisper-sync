use whisper_sync::{Whisper, WhisperType, WhisperFilter};
use chrono::{Duration, Utc};

#[test]
fn test_whisper_serialization() {
    let wt = WhisperType::Status { health: 0.95, load: 0.3 };
    let w = Whisper::new("agent1".to_string(), Some("agent2".to_string()), wt, 1800);

    let json = serde_json::to_string(&w).expect("serialize");
    let parsed: Whisper = serde_json::from_str(&json).expect("deserialize");

    assert_eq!(parsed.from, "agent1");
    assert_eq!(parsed.to, Some("agent2".to_string()));
}

#[test]
fn test_whisper_type_variant_names() {
    assert_eq!(WhisperType::Status { health: 1.0, load: 0.0 }.variant_name(), "status");
    assert_eq!(WhisperType::Discovery { service: "svc".into(), endpoint: "ep".into() }.variant_name(), "discovery");
    assert_eq!(WhisperType::Help { question: "?".into(), tags: vec![] }.variant_name(), "help");
    assert_eq!(WhisperType::Insight { summary: "s".into(), confidence: 0.9, source_theorem: None }.variant_name(), "insight");
    assert_eq!(WhisperType::Trust { agent: "a".into(), trust_score: 0.8 }.variant_name(), "trust");
    assert_eq!(WhisperType::Alert { severity: "warn".into(), message: "m".into() }.variant_name(), "alert");
}

#[test]
fn test_whisper_expired() {
    let mut w = Whisper::new(
        "a".to_string(),
        None,
        WhisperType::Alert { severity: "info".into(), message: "test".into() },
        60,
    );
    w.timestamp = Utc::now() - Duration::minutes(5);
    assert!(w.is_expired());
}

#[test]
fn test_whisper_not_expired() {
    let w = Whisper::new(
        "a".to_string(),
        None,
        WhisperType::Status { health: 1.0, load: 0.0 },
        1800,
    );
    assert!(!w.is_expired());
}
