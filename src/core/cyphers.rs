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

#[derive(Debug)]
pub struct PrivateKey {
    key_bytes: [u8; 32],
}

impl PrivateKey {
    pub fn generate() -> Result<Self, String> {
        let pk = SecretKey::random(&mut OsRng);
        let pk_bytes: Result<[u8; 32], String> = pk
            .to_bytes()
            .to_vec()
            .try_into()
            .map_err(|_| "private_key bytes size should equal to 32".into());

        match pk_bytes {
            Ok(key_bytes) => Ok(Self { key_bytes }),
            Err(e) => Err(e),
        }
    }

    pub fn public_key(&self) -> PublicKey {
        PublicKey::from(&self)
    }

    pub fn sign(&self, data: &[u8]) -> Result<Signature, EcdsaErr> {
        let signing_key = SigningKey::from_slice(self.as_bytes());
        if let Ok(sk) = signing_key {
            let signature = sk.sign(data);
            return Ok(Signature { signature });
        }

        Err(signing_key.unwrap_err())
    }

    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.key_bytes
    }

    pub fn as_hex(&self) -> String {
        hex::encode(&self.key_bytes)
    }
}

#[test]
fn test_pk() {
    if let Ok(pk) = PrivateKey::generate() {
        eprintln!("pk bytes: {:?}, hex: {}", pk.key_bytes, pk.as_hex());
    };

    assert!(false);
}

#[derive(Debug)]
pub struct PublicKey {
    key_bytes: Vec<u8>,
}

impl PublicKey {
    pub fn from(private_key: &PrivateKey) -> Self {
        let pk_bytes = private_key.as_bytes();
        let public_key_bytes = SecretKey::from_slice(pk_bytes)
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

    pub fn as_bytes(&self) -> Vec<u8> {
        self.key_bytes.clone()
    }

    pub fn as_hex(&self) -> String {
        hex::encode(&self.key_bytes)
    }
}

// TODO: impl custom serialize & deserialize for Signature
// because Block { Trxs: Vec<Trx { -> singature <- }> }; Block is serd!
#[derive(Debug, Clone)]
pub struct Signature {
    signature: EcdsaSignature,
}

impl serde::Serialize for Signature {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use core::ops::Deref;
        // use serde::ser::Error;

        let sig_bytes = self.signature.to_bytes();
        let sig_bytes = sig_bytes.deref();
        dbg!(sig_bytes);
        serializer.serialize_bytes(sig_bytes)
    }
}

struct SignatureVisitor;
impl<'de> serde::de::Visitor<'de> for SignatureVisitor {
    type Value = Signature;
    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("")
    }

    fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        EcdsaSignature::from_slice(v)
            .map(|sig| Signature { signature: sig })
            .map_err(|e| E::custom(""))
    }
}

impl<'de> serde::Deserialize<'de> for Signature {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_bytes(SignatureVisitor)
    }
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
fn serd_signature() {
    use super::block::Block;
    let b = Block::new("prev_hash".into(), 0, 0, vec![]);
    let encoded_b = &b.encode().unwrap();

    let private_key = PrivateKey::generate().unwrap();
    let signature = private_key.sign(&encoded_b).unwrap();

    let ser = bincode::serialize(&signature).unwrap();
    dbg!(&ser);
    let de: Signature = bincode::deserialize(ser.as_slice()).unwrap();
    dbg!(&de);

    eprintln!("again {:?}", bincode::serialize(&de));

    // last 5 bytes, should all match.
    // check, after serd signature is the same

    assert!(false)
}

#[test]
fn sign_block() {
    use super::block::Block;
    let b = Block::new("prev_hash".into(), 0, 0, vec![]);
    let encoded_b = &b.encode().unwrap();

    let private_key = PrivateKey::generate().unwrap();
    let signature = private_key.sign(&encoded_b).unwrap();
    assert_eq!(signature.as_bytes().is_empty(), false);

    let public_key = private_key.public_key();
    let result = public_key.verify(&signature, &encoded_b);
    assert!(result.is_ok());
}
