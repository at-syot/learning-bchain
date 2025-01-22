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
