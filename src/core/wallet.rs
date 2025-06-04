use super::cyphers::{Encoder, PrivateKey, PublicKey, Signature};
use super::transaction::Transaction;
use k256::ecdsa::Error as EcdsaErr;

#[derive(Debug)]
pub struct Wallet {
    data: Vec<u8>,

    pub address: String,

    private_key: PrivateKey,
    public_key: PublicKey,
}

impl Wallet {
    pub fn new(data: Vec<u8>) -> Result<Self, String> {
        let private_key = PrivateKey::generate()?;
        let public_key = private_key.public_key();

        // generate address: !keep simple! -- first 20 of public bytes
        // address can follow bitcoin or ETH : will be implemented
        let address = hex::encode(&public_key.as_bytes()[0..20]);
        Ok(Self {
            data,
            address,
            private_key,
            public_key,
        })
    }

    pub fn create_transaction(&self, recipient_addr: String, value: f64) -> Transaction {
        Transaction::new(self.address.clone(), recipient_addr, value)
    }

    pub fn sign_transaction(&self, trx: &mut Transaction) -> Result<(), EcdsaErr> {
        let trx_signature = self.private_key.sign(&trx.data)?;
        trx.signature = Some(trx_signature);
        Ok(())
    }
}

// Test:
// - wallet successfully sign trx
// - signed trx can safe & correctly serd to bytes
// - signed trx is still correct in both before and after serd
#[test]
fn wallet() {
    let w = Wallet::new(vec![]).unwrap();
    let mut trx = w.create_transaction("recv_hex".to_string(), 1.0);
    w.sign_transaction(&mut trx);
    dbg!(&trx.signature);

    // Before:
    // trx serialize: verify trx data - valid
    let clone_signature = trx.signature.clone();
    let valid = w
        .public_key
        .verify(&clone_signature.unwrap(), &trx.data[..]);
    assert!(valid.is_ok());

    // After:
    // - trx serialize then deserialize back to Transaction struct (is trx's data valid ?)
    let clone_trx = trx.clone();
    let ser_trx = bincode::serialize(&clone_trx).expect("ser_trx");
    let de_trx: Transaction = bincode::deserialize(&ser_trx.as_slice()).expect("de_trx");
    let after_valid = w
        .public_key
        .verify(&de_trx.signature.unwrap(), &de_trx.data);
    assert!(after_valid.is_ok());
}
