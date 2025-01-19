mod core;
use core::block_chain::BlockChain;

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
