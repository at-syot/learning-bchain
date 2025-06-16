use super::cyphers::{PrivateKey, PublicKey, Signature};
use super::transaction::{TxBuilder, TxBuilderResult};
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

    // TODO: creating transaction
    // - request blockchain history tx of given wallet's address (UTXO input)
    // - compute trx outputs
    // - inject inputs, outputs when creating tx
    // - sign tx
    // ### DONE ### creating transaction
    pub fn create_transaction(&self, receiver_addr: String, value: f64) -> TxBuilderResult {
        TxBuilder::new(self.address.clone(), receiver_addr, value)
            .inputs(vec![])
            .outputs(vec![])
            .build()
    }

    pub fn sign_data(&self, data: &[u8]) -> Result<Signature, EcdsaErr> {
        self.private_key.sign(&data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::cyphers::Encoder;
    use crate::core::transaction::Transaction;

    // give a better test name.
    #[test]
    fn wallet() {
        let w = Wallet::new(vec![]).unwrap();
        let trx_result = w.create_transaction("recv_hex".to_string(), 1.0);
        dbg!(&trx_result);
        assert!(trx_result.is_ok());

        let mut trx = trx_result.unwrap();
        let signing_result = w.sign_data(&mut trx.data.as_slice());
        dbg!(&signing_result);
        assert!(signing_result.is_ok());

        trx.signature = Some(signing_result.unwrap());

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
        let ser_trx_result = clone_trx.encode();
        assert!(ser_trx_result.is_ok());

        let de_trx: Transaction = bincode::deserialize(&ser_trx_result.unwrap()).expect("de_trx");
        let after_valid = w
            .public_key
            .verify(&de_trx.signature.unwrap(), &de_trx.data);
        assert!(after_valid.is_ok());
    }
}
