use crate::{PlatoTransport, Whisper};
use crate::filter::WhisperFilter;

pub enum DeliveryMode {
    Unicast,
    Multicast,
    Broadcast,
}

pub struct DeliveryService {
    transport: PlatoTransport,
    filter: WhisperFilter,
}

impl DeliveryService {
    pub fn new(transport: PlatoTransport) -> Self {
        Self {
            transport,
            filter: WhisperFilter::new(),
        }
    }

    /// Send a whisper to a specific agent (unicast)
    pub async fn send_unicast(&self, whisper: &Whisper) -> Result<(), crate::plato_transport::PlatoError> {
        self.transport.submit_whisper(whisper).await
    }

    /// Send a whisper to room members (multicast)
    /// For multicast, we leave `to` field empty — all room members receive it
    pub async fn send_multicast(&self, whisper: &Whisper) -> Result<(), crate::plato_transport::PlatoError> {
        self.transport.submit_whisper(whisper).await
    }

    /// Send a whisper to all fleet agents (broadcast)
    /// Same as multicast — PLATO room is the broadcast domain
    pub async fn send_broadcast(&self, whisper: &Whisper) -> Result<(), crate::plato_transport::PlatoError> {
        self.transport.submit_whisper(whisper).await
    }

    /// Filter a list of whispers, returning only unexpired + relevant ones for this agent
    pub fn filter_whispers(&self, whispers: Vec<Whisper>, agent_id: &str) -> Vec<Whisper> {
        whispers.into_iter()
            .filter(|w| self.filter.is_relevant(w, agent_id))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::WhisperType;

    #[test]
    fn test_delivery_mode_variants() {
        let _ = DeliveryMode::Unicast;
        let _ = DeliveryMode::Multicast;
        let _ = DeliveryMode::Broadcast;
    }
}
