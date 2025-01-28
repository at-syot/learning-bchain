use k256::ecdsa::signature::{Signer, Verifier};
use k256::ecdsa::Error as EcdsaErr;
use k256::ecdsa::{Signature as EcdsaSignature, SigningKey, VerifyingKey};
use k256::elliptic_curve::rand_core::OsRng;
use k256::SecretKey;

pub trait Encoder {
    fn encode(&self) -> Result<Vec<u8>, String>;
}

pub trait Decoder {
    fn decode(&self, encoded: &Vec<u8>) -> Result<Box<Self>, String>;
}

pub struct PrivateKey {
    key_bytes: Vec<u8>,
}

impl PrivateKey {
    pub fn generate() -> Self {
        let pk = SecretKey::random(&mut OsRng);
        Self {
            key_bytes: pk.to_bytes().to_vec(),
        }
    }

    pub fn public_key(&self) -> PublicKey {
        PublicKey::from(&self)
    }

    pub fn sign(&self, data: &[u8]) -> Result<Signature, EcdsaErr> {
        let signing_key = SigningKey::from_slice(&self.as_bytes());
        if let Ok(sk) = signing_key {
            let signature = sk.sign(data);
            return Ok(Signature { signature });
        }

        Err(signing_key.unwrap_err())
    }

    pub fn as_bytes(&self) -> &Vec<u8> {
        &self.key_bytes
    }

    pub fn as_hex(&self) -> String {
        hex::encode(&self.key_bytes)
    }
}

pub struct PublicKey {
    key_bytes: Vec<u8>,
}

impl PublicKey {
    pub fn from(private_key: &PrivateKey) -> Self {
        let pk_bytes = private_key.as_bytes();
        let public_key_bytes = SecretKey::from_slice(&pk_bytes)
            .unwrap() // ignore err
            .public_key()
            .to_sec1_bytes()
            .to_vec();

        Self {
            key_bytes: public_key_bytes,
        }
    }

    pub fn verify(&self, signature: &Signature, data: &[u8]) -> Result<(), EcdsaErr> {
        let verifying_key = VerifyingKey::from_sec1_bytes(&self.key_bytes);
        if let Err(e) = verifying_key {
            return Err(e);
        }

        verifying_key.unwrap().verify(&data, &signature.signature)
    }
}

pub struct Signature {
    signature: EcdsaSignature,
}

impl Signature {
    pub fn as_bytes(&self) -> Vec<u8> {
        self.signature.to_der().to_bytes().to_vec()
    }

    pub fn as_hex(&self) -> String {
        hex::encode(self.signature.to_der().to_bytes())
    }
}

#[test]
fn sign_block() {
    use super::block::Block;
    let b = Block::new("prev_hash".into(), 0, 0, vec![]);
    let encoded_b = &b.encode().unwrap();

    let private_key = PrivateKey::generate();
    // let signature = KeysPair::sign(hex_private.as_ref(), &encoded_b).unwrap();
    let signature = private_key.sign(&encoded_b).unwrap();
    assert_eq!(signature.as_bytes().is_empty(), false);

    let public_key = private_key.public_key();
    let result = public_key.verify(&signature, &encoded_b);
    assert!(result.is_ok());
}
