use k256::ecdsa::signature::{Signer, Verifier};
use k256::ecdsa::Error as EcdsaErr;
use k256::ecdsa::{Signature, SigningKey, VerifyingKey};
use k256::elliptic_curve::rand_core::OsRng;
use k256::SecretKey;

pub trait Encoder {
    fn encode(&self) -> Result<Vec<u8>, String>;
}

pub trait Decoder {
    fn decode(&self, encoded: &Vec<u8>) -> Result<Box<Self>, String>;
}

pub struct KeysPair();
impl KeysPair {
    pub fn generate_private() -> String {
        let private = SecretKey::random(&mut OsRng);
        hex::encode(private.to_bytes())
    }

    pub fn derive_public(hex_private_key: &str) -> String {
        let private_key_bytes = hex::decode(hex_private_key).unwrap();
        let temp_private = SecretKey::from_slice(&private_key_bytes).unwrap();
        let public_key = temp_private.public_key().to_sec1_bytes();
        hex::encode(&public_key)
    }

    pub fn derive_wallet_address(hex_public_key: &str) {
        let decoded_pub_key = hex::decode(hex_public_key).unwrap();
        let sha256_hash = sha256::digest(hex_public_key);
        println!("\ndecoded_pub_key {:?}", decoded_pub_key);
        println!("sha256_hash {:?}\n", sha256_hash);

        // TODO: derive bitcoin or etheream wallet's address
    }

    pub fn sign(hex_private_key: &str, data: &[u8]) -> Result<String, EcdsaErr> {
        let signing_key = hex::decode(hex_private_key)
            .map_err(|e| EcdsaErr::from_source(e))
            .and_then(|private_key_bytes| SigningKey::from_slice(&private_key_bytes))
            .map(|signing_key| signing_key as SigningKey);

        if let Ok(sk) = signing_key {
            let signature: Signature = sk.sign(data);
            let signature_as_hex = hex::encode(signature.to_der().to_bytes());
            return Ok(signature_as_hex);
        }

        Err(signing_key.unwrap_err())
    }

    pub fn verify_signature(
        hex_pub_key: &str,
        hex_signature: &str,
        data: &[u8],
    ) -> Result<(), EcdsaErr> {
        let verifying_key = hex::decode(hex_pub_key)
            .map_err(|e| EcdsaErr::from_source(e))
            .and_then(|pub_key_bytes| VerifyingKey::from_sec1_bytes(&pub_key_bytes));
        if let Err(e) = verifying_key {
            return Err(e);
        }

        let signature = hex::decode(hex_signature)
            .map_err(|e| EcdsaErr::from_source(e))
            .and_then(|signature_bytes| Signature::from_der(&signature_bytes));

        if let Err(e) = signature {
            return Err(e);
        }

        verifying_key.unwrap().verify(&data, &signature.unwrap())
    }
}

#[test]
fn test_keypair() {
    let hex_private = KeysPair::generate_private();
    println!("private {hex_private}");

    let hex_pub = KeysPair::derive_public(&hex_private);
    println!("public {}", hex_pub);

    KeysPair::derive_wallet_address(&hex_pub);

    assert!(false)
}

#[test]
fn sign_block() {
    use super::block::Block;
    let b = Block::new("prev_hash".into(), 0, 0, vec![]);
    let encoded_b = &b.encode().unwrap();

    let hex_private = KeysPair::generate_private();
    let signature = KeysPair::sign(hex_private.as_ref(), &encoded_b).unwrap();
    assert_eq!(signature.is_empty(), false);

    let hex_public = KeysPair::derive_public(hex_private.as_ref());
    let result = KeysPair::verify_signature(&hex_public, &signature, &encoded_b);
    assert!(result.is_ok());
}
