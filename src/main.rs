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
        serde_json::to_string(&self.header).map(sha256::digest)
    }
}

const DIFFICULTY: u8 = 3;

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

    pub fn valid_proof(&self, adding_block: &Block) -> bool {
        match adding_block
            .hash()
            .map(|hash| hash.chars().take(DIFFICULTY as usize).all(|c| c == '0'))
        {
            Ok(valid) => valid,
            Err(_) => false,
        }
    }

    pub fn proof_of_work(&self) -> u64 {
        // challenge(future nonce) + prev_hash + transactions(pool)
        let mut nonce = 0;
        loop {
            let prev_hash = self.latest_block().unwrap().hash().unwrap();
            let transactions = self.transaction_pool.to_vec();
            let adding_block = Block::new(prev_hash, nonce, 0, transactions);
            if self.valid_proof(&adding_block) {
                break;
            }
            nonce += 1;
        }
        nonce
    }

    pub fn inspect(&self) {
        for (i, block) in self.chain.iter().enumerate() {
            println!("{} Block {} {}", "#".repeat(25), i, "#".repeat(25));
            println!("-> {:?}", block);
            println!("-> {:?}", self.transaction_pool);
            println!("{}", "*".repeat(30));
            println!("\n\n");
        }
    }
}

fn main() {
    let mut bc = BlockChain::new();
    bc.inspect();

    bc.add_transaction("A".into(), "B".into(), 100);
    bc.add_transaction("C".into(), "D".into(), 101);
    let next_block_nonce = bc.proof_of_work();
    bc.create_block(bc.latest_block().unwrap().hash().unwrap(), next_block_nonce);
    bc.inspect();

    bc.create_block(bc.latest_block().unwrap().hash().unwrap(), 2);
}
