use chrono::Utc;
use serde::Serialize;

#[derive(Debug, Serialize, Clone)]
struct Transaction {
    sender_addr: String,
    recipient_addr: String,
    value: u64,
}

impl Transaction {
    pub fn new(sender_addr: String, recipient_addr: String, value: u64) -> Self {
        Transaction {
            sender_addr,
            recipient_addr,
            value,
        }
    }
}

#[derive(Debug, Serialize)]
struct BlockHeader {
    time_stamp: i64,
    prev_hash: String,
    height: u64,
    nonce: u64,
}

#[derive(Debug, Serialize)]
struct Block {
    header: BlockHeader,
    transactions: Vec<Transaction>,
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
        serde_json::to_string(self).map(sha256::digest)
    }
}

#[derive(Debug)]
struct BlockChain {
    pub transaction_pool: Vec<Transaction>,
    pub chain: Vec<Block>, // should be ref with lifetime specified
}

impl BlockChain {
    pub fn new() -> Self {
        // create genesis block
        let b = Block::new("hash_0".into(), 0, 1, vec![]);
        BlockChain {
            chain: vec![b],
            transaction_pool: vec![],
        }
    }

    pub fn create_block(&mut self, prev_hash: String, nonce: u64) -> Option<&Block> {
        let new_block = Block::new(
            prev_hash,
            nonce,
            self.chain.len() as u64,
            self.transaction_pool.to_vec(),
        );
        self.chain.push(new_block);

        // empty the transaction_pool
        self.transaction_pool = vec![];

        self.chain.last()
    }

    pub fn latest_block(&self) -> Option<&Block> {
        self.chain.last()
    }

    pub fn add_transaction(
        &mut self,
        sender_addr: String,
        recipient_addr: String,
        value: u64,
    ) -> Option<&Transaction> {
        let tx = Transaction::new(sender_addr, recipient_addr, value);
        self.transaction_pool.push(tx);
        self.transaction_pool.last()
    }

    pub fn inspect(&self) {
        for (i, block) in self.chain.iter().enumerate() {
            println!("{} Block {} {}", "#".repeat(25), i, "#".repeat(25));
            println!("-> {:?}", block);
            println!("-> {:?}", self.transaction_pool);
            println!("{}", "*".repeat(30))
        }
    }
}

fn main() {
    let mut bc = BlockChain::new();
    bc.add_transaction("A".into(), "B".into(), 100);
    bc.add_transaction("C".into(), "D".into(), 101);
    bc.inspect();
    bc.create_block(bc.latest_block().unwrap().hash().unwrap(), 1);
    bc.inspect();

    bc.create_block(bc.latest_block().unwrap().hash().unwrap(), 2);
}
