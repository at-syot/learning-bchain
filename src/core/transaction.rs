use super::cyphers::Encoder;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct Transaction {
    sender_addr: String,
    recipient_addr: String,
    value: f64,
}

impl Transaction {
    pub fn new(sender_addr: String, recipient_addr: String, value: f64) -> Self {
        Transaction {
            sender_addr,
            recipient_addr,
            value,
        }
    }
}

impl Encoder for Transaction {
    fn encode(&self) -> Result<Vec<u8>, String> {
        bincode::serialize(self).map_err(|e| e.to_string())
    }
}
