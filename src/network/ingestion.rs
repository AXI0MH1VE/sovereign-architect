//! P2P Data Ingestion Layer
//!
//! Decentralized network for financial data streaming using libp2p.
//! Enables sovereign nodes to share fragility signals without central authority.

use libp2p::{
    gossipsub::{self, Gossipsub, GossipsubEvent, MessageAuthenticity, ValidationMode},
    identity::Keypair,
    swarm::{SwarmBuilder, SwarmEvent},
    Multiaddr, PeerId, Swarm,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;
use tokio::sync::mpsc;

use crate::core::lagrangian::BankState;

/// Financial data packet for P2P network
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataPacket {
    /// Timestamp (Unix epoch milliseconds)
    pub timestamp: u64,
    /// Source node ID
    pub source: String,
    /// Bank state data
    pub state: BankState,
    /// Fragility score
    pub fragility: f64,
    /// Signature (verification)
    pub signature: Vec<u8>,
}

/// Network configuration
#[derive(Debug, Clone)]
pub struct NetworkConfig {
    /// Listen address
    pub listen_addr: String,
    /// Bootstrap peers
    pub bootstrap_peers: Vec<String>,
    /// Topic for gossipsub
    pub topic: String,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            listen_addr: \"/ip4/0.0.0.0/tcp/0\".to_string(),
            bootstrap_peers: vec![],
            topic: \"olo-fragility\".to_string(),
        }
    }
}

/// P2P network ingestion engine
pub struct IngestionEngine {
    swarm: Swarm<Gossipsub>,
    topic: gossipsub::IdentTopic,
    data_rx: mpsc::Receiver<DataPacket>,
    data_tx: mpsc::Sender<DataPacket>,
}

impl IngestionEngine {
    /// Create new ingestion engine
    pub fn new(config: NetworkConfig) -> Result<Self, Box<dyn Error>> {
        // Generate keypair
        let local_key = Keypair::generate_ed25519();
        let local_peer_id = PeerId::from(local_key.public());

        // Create gossipsub
        let gossipsub_config = gossipsub::GossipsubConfigBuilder::default()
            .heartbeat_interval(std::time::Duration::from_secs(10))
            .validation_mode(ValidationMode::Strict)
            .build()
            .expect(\"Valid gossipsub config\");

        let mut gossipsub = Gossipsub::new(
            MessageAuthenticity::Signed(local_key.clone()),
            gossipsub_config,
        )
        .expect(\"Failed to create gossipsub\");

        // Subscribe to topic
        let topic = gossipsub::IdentTopic::new(&config.topic);
        gossipsub.subscribe(&topic)?;

        // Create swarm
        let swarm = SwarmBuilder::with_tokio_executor(
            libp2p::Transport::boxed(libp2p::tcp::tokio::Transport::default()),
            gossipsub,
            local_peer_id,
        )
        .build();

        // Create channel for data packets
        let (data_tx, data_rx) = mpsc::channel(1000);

        Ok(Self {
            swarm,
            topic,
            data_rx,
            data_tx,
        })
    }

    /// Start listening for incoming data
    pub async fn listen(&mut self, addr: Multiaddr) -> Result<(), Box<dyn Error>> {
        self.swarm.listen_on(addr)?;
        Ok(())
    }

    /// Publish data packet to network
    pub async fn publish(&mut self, packet: DataPacket) -> Result<(), Box<dyn Error>> {
        let data = serde_json::to_vec(&packet)?;
        self.swarm
            .behaviour_mut()
            .publish(self.topic.clone(), data)?;
        Ok(())
    }

    /// Process network events
    pub async fn process_events(&mut self) -> Result<Option<DataPacket>, Box<dyn Error>> {
        loop {
            tokio::select! {
                event = self.swarm.select_next_some() => {
                    match event {
                        SwarmEvent::Behaviour(GossipsubEvent::Message {
                            message,
                            ..
                        }) => {
                            // Deserialize data packet
                            if let Ok(packet) = serde_json::from_slice::<DataPacket>(&message.data) {
                                return Ok(Some(packet));
                            }
                        }
                        _ => {}
                    }
                }
                packet = self.data_rx.recv() => {
                    if let Some(p) = packet {
                        self.publish(p).await?;
                    }
                }
            }
        }
    }

    /// Get sender for publishing data
    pub fn get_sender(&self) -> mpsc::Sender<DataPacket> {
        self.data_tx.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_engine_creation() {
        let config = NetworkConfig::default();
        let engine = IngestionEngine::new(config);
        assert!(engine.is_ok());
    }

    #[test]
    fn test_data_packet_serialization() {
        let packet = DataPacket {
            timestamp: 1234567890,
            source: \"test-node\".to_string(),
            state: BankState {
                assets: 1000.0,
                liabilities: 900.0,
                equity: 100.0,
                leverage: 9.0,
            },
            fragility: 15.0,
            signature: vec![1, 2, 3, 4],
        };

        let serialized = serde_json::to_string(&packet);
        assert!(serialized.is_ok());

        let deserialized = serde_json::from_str::<DataPacket>(&serialized.unwrap());
        assert!(deserialized.is_ok());
    }
}
