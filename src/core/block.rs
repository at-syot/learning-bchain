use super::transaction::Transaction;
use chrono::Utc;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct BlockHeader {
    time_stamp: i64,
    prev_hash: String,
    height: u64,
    nonce: u64,
}

#[derive(Debug, Serialize)]
pub struct Block {
    pub header: BlockHeader,
    pub transactions: Vec<Transaction>,
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
