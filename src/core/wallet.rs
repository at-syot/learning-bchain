use super::cyphers::{Encoder, PrivateKey, PublicKey, Signature};
use super::transaction::Transaction;
use k256::ecdsa::Error as EcdsaErr;

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
        // address can follow bitcoin or ETH : will be impl
        let address = hex::encode(&public_key.as_bytes().to_vec()[0..20]);
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

    pub fn sign_transaction(&self, trx: &Transaction) -> Result<Signature, EcdsaErr> {
        trx.encode()
            .map_err(|e| EcdsaErr::from_source(e))
            .and_then(|encoded_trx| self.private_key.sign(&encoded_trx))
    }
}
