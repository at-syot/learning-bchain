mod core;
use core::block_chain::BlockChain;

fn main() {
    let minner_address = Some(String::from("minner"));
    let mut bc = BlockChain::new();
    bc.block_chain_address = minner_address;
    bc.inspect();

    bc.add_transaction("A".into(), "B".into(), 2.0);
    bc.add_transaction("C".into(), "D".into(), 3.0);
    bc.minning();
    bc.inspect();

    bc.add_transaction("Me".into(), "Mom".into(), 1.0);
    bc.minning();
    bc.inspect();
}
