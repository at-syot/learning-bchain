use super::cyphers::{Decoder, Encoder};
use super::transaction::Transaction;
use chrono::Utc;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct BlockHeader {
    time_stamp: i64,
    prev_hash: String,
    height: u64,
    nonce: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Block {
    pub header: BlockHeader,
    pub transactions: Vec<Transaction>,
}

impl Encoder for Block {
    fn encode(&self) -> Result<Vec<u8>, String> {
        bincode::serialize(&self).map_err(|e| e.to_string())
    }
}

impl Decoder for Block {
    fn decode(&self, encoded: &Vec<u8>) -> Result<Box<Self>, String> {
        bincode::deserialize(&encoded[..])
            .map_err(|e| e.to_string())
            .map(|decoded| decoded) as Result<Box<Block>, String>
    }
}

impl Block {
    pub fn new(prev_hash: String, nonce: u64, height: u64, transactions: Vec<Transaction>) -> Self {
        let header = BlockHeader {
            prev_hash,
            time_stamp: Utc::now().timestamp(),
            height,
            nonce,
        };
        Block {
            transactions,
            header,
        }
    }

    pub fn hash(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(&self.header).map(sha256::digest)
    }
}

#[test]
fn test_encode_block() {
    let encoding = Block::new(String::from("prev_hash"), 1990, 0, vec![]);
    let encoded = &encoding.encode();
    println!("{:?}", encoded);

    let _ = encoding.decode(encoded.as_ref().unwrap()).map(|decoded| {
        println!("decoded block----- {:?}", decoded);
    });
    assert!(false)
}
