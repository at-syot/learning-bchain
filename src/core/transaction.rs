use super::cyphers::{Encoder, Signature};
use serde::{Deserialize, Serialize};

// Miles stone
// creating transaction
// wallet -> create trx -> send to blockchain -> boardcasting to all nodes

// TODO:
// - generate wallet address from public key: DONE
// - create & sign transaction [trx-data, trx-signature]: DONE
// - POC: public key recovery from singnature: DONE
//
// DOING
// - blockchain network

#[derive(Debug, Serialize, Deserialize)]
pub struct TransactionData {
    sender_addr: String,
    recipient_addr: String,
    value: f64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Transaction {
    pub data: Vec<u8>,
    pub signature: Option<Signature>,
}

impl Transaction {
    pub fn new(sender_addr: String, recipient_addr: String, value: f64) -> Self {
        let trx_data = TransactionData {
            sender_addr,
            recipient_addr,
            value,
        };
        let serialized_trx_data = bincode::serialize(&trx_data).unwrap();
        Transaction {
            data: serialized_trx_data,
            signature: None,
        }
    }
}

impl Encoder for Transaction {
    fn encode(&self) -> Result<Vec<u8>, String> {
        // bincode::serialize(self).map_err(|e| e.to_string())
        Ok(vec![])
    }
}

#[test]
fn trx() {
    let trx = Transaction::new("a".to_string(), "b".to_string(), 1.0);
}
