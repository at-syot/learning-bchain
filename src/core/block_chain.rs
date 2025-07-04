use super::block::Block;
use super::transaction::{Transaction, TransactionData};

const DIFFICULTY: u8 = 3;
const MINNING_SENDER: &'static str = "blockchain";
const MINNING_REWARD: f64 = 1.0;

#[derive(Debug)]
pub struct BlockChain {
    pub mem_pool: Vec<Transaction>, // pending trxs
    pub chain: Vec<Block>,          // should be ref with lifetime specified
    pub block_chain_address: Option<String>,
}

impl BlockChain {
    pub fn new() -> Self {
        // create genesis block
        let mut b = Block::new("hash_0".into(), 0, 1, vec![]);
        b.gen_hash();

        BlockChain {
            chain: vec![b],
            mem_pool: vec![],
            block_chain_address: None,
        }
    }

    pub fn create_block(&mut self, prev_hash: String, nonce: u64) -> Option<&Block> {
        let mut b = Block::new(
            prev_hash,
            nonce,
            self.chain.len() as u64,
            self.mem_pool.to_vec(),
        );
        b.gen_hash();

        self.chain.push(b);

        // empty the mem_pool
        self.mem_pool = vec![];

        self.chain.last()
    }

    pub fn latest_block(&self) -> Option<&Block> {
        self.chain.last()
    }

    // for testing
    fn block_nth(&mut self, nth: usize) -> Option<&mut Block> {
        self.chain.iter_mut().nth(nth)
    }

    pub fn add_transaction(&mut self, tx: Transaction) -> Option<&Transaction> {
        self.mem_pool.push(tx);
        self.mem_pool.last()
    }

    pub fn txs_of_addr(&self, addr: String) -> Vec<Transaction> {
        // let mut balance = 0f64;
        // let mut spended = 0f64;
        // let mut received = 0f64;
        // 'chain_loop: for b in &self.chain {
        //     for tx in &b.transactions {
        //         let txdata_de_result = bincode::deserialize::<TransactionData>(&tx.data[..]);
        //         if let Err(_) = txdata_de_result {
        //             break 'chain_loop;
        //         }
        //
        //         let txdata = txdata_de_result.unwrap();
        //         if addr == txdata.receiver_addr {
        //             // have received money
        //             received += txdata.value;
        //         }
        //         if addr == txdata.sender_addr {
        //             // have spended money
        //             spended += txdata.value
        //         }
        //     }
        // }
        //
        // balance = received - spended;

        let mut addr_txs: Vec<Transaction> = vec![];
        'chain_loop: for b in &self.chain {
            for tx in &b.transactions {
                let txdata_de_result = bincode::deserialize::<TransactionData>(&tx.data[..]);
                if let Err(_) = txdata_de_result {
                    break 'chain_loop;
                }

                let txdata = txdata_de_result.unwrap();
                if txdata.sender_addr.eq(&addr) || txdata.receiver_addr.eq(&addr) {
                    addr_txs.push(tx.clone());
                }
            }
        }

        addr_txs
    }

    pub fn valid_proof(&self, adding_block: &mut Block) -> bool {
        match adding_block
            .gen_hash()
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
            let prev_hash = self.latest_block().unwrap().hash();
            let transactions = self.mem_pool.to_vec();
            let block_height = self.chain.len();
            let mut adding_block = Block::new(prev_hash, nonce, block_height as u64, transactions);
            if self.valid_proof(&mut adding_block) {
                break;
            }
            nonce += 1;
        }
        nonce
    }

    pub fn minning(&mut self) {
        // self.add_transaction(
        //     MINNING_SENDER.into(),
        //     self.block_chain_address.as_ref().unwrap().clone(),
        //     MINNING_REWARD,
        // );

        let prev_hash = self.latest_block().unwrap().hash();
        let nonce = self.proof_of_work();
        self.create_block(prev_hash, nonce);
        println!("action=minning status=success");
    }

    pub fn is_valid(&mut self) -> bool {
        self.chain
            .iter_mut()
            .skip(1)
            .all(|b| b.hash() == b.gen_hash().unwrap())
    }

    pub fn inspect(&self) {
        // println!("BlockChain {:?}", self);
        for (i, block) in self.chain.iter().enumerate() {
            println!("{} Block {} {}", "#".repeat(25), i, "#".repeat(25));
            println!("-> {:?}", block);
            println!("-> {:?}", self.mem_pool);
            println!("{}", "*".repeat(30));
            println!("\n\n");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn utxo() {
        // ## sending money A(sender)->B(receiver) : flow
        // A wallet: create trx
        // push trx to blockchain's network
        // dedicate node||peer to basic verify the trx [trx's structure, signature, double spending(trx_id is exist?)]
        //    if trx is valid -> broadcast trx to all peers
        //    if trx isn't valid -> reject trx
        // peers help verify&confirm that trx :: minimum peers confirmation (where is that come from?)

        // balance come from :: previous
    }

    #[test]
    fn test_is_valid() {
        let mut bc = BlockChain::new();

        // create block 1
        let genesis_hash = bc.latest_block().unwrap().hash();
        let block_1 = bc.create_block(genesis_hash, 0).unwrap();

        // create block 2
        let block_1_hash = block_1.hash();
        let block_2 = bc.create_block(block_1_hash, 1).unwrap();
        let block_2_hash = block_2.hash();

        assert!(&bc.is_valid());

        bc.block_nth(1).unwrap().header.time_stamp = chrono::Utc::now().timestamp();
        bc.block_nth(1).unwrap().header.nonce = 90;
        println!("\n\n-----------\n\n");
        assert_eq!(&bc.is_valid(), &false);
    }
}
