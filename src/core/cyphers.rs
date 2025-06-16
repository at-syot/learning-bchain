use k256::ecdsa::signature::{Signer, Verifier};
use k256::ecdsa::Error as EcdsaErr;
use k256::ecdsa::{RecoveryId, Signature as EcdsaSignature, SigningKey, VerifyingKey};
use k256::elliptic_curve::rand_core::OsRng;
use k256::SecretKey;

pub trait Encoder {
    fn encode(&self) -> Result<Vec<u8>, String>;
}

// TODO: remove this thread. it doesn't make any sense!
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
        let signing_key = SigningKey::from_slice(self.as_bytes())?;
        let signature = signing_key.sign(data);

        Ok(Signature { inner: signature })
    }

    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.key_bytes
    }

    pub fn as_hex(&self) -> String {
        hex::encode(&self.key_bytes)
    }
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

        verifying_key.unwrap().verify(&data, &signature.inner)
    }

    pub fn as_bytes(&self) -> Vec<u8> {
        self.key_bytes.clone()
    }

    pub fn as_hex(&self) -> String {
        hex::encode(&self.key_bytes)
    }
}

#[derive(Debug, Clone)]
pub struct Signature {
    inner: EcdsaSignature,
}

impl serde::Serialize for Signature {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use core::ops::Deref;
        // use serde::ser::Error;

        let sig_bytes = self.inner.to_bytes();
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
            .map(|sig| Signature { inner: sig })
            .map_err(|e| E::custom(""))
    }
    // Remark: maybe able to deserialize json, in-case serde::json
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
        self.inner.to_der().to_bytes().to_vec()
    }

    pub fn as_hex(&self) -> String {
        hex::encode(self.inner.to_der().to_bytes())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::block::Block;

    #[test]
    fn serd_signature() {
        let b = Block::new("prev_hash".into(), 0, 0, vec![]);
        let encoded_b = &b.encode().unwrap();

        let private_key = PrivateKey::generate().unwrap();
        let signature = private_key.sign(&encoded_b).unwrap();

        let ser = bincode::serialize(&signature).unwrap();
        let de: Signature = bincode::deserialize(ser.as_slice()).unwrap();

        let last3_ser = &ser[ser.len() - 3..ser.len()];
        let last3_after_ser: &Vec<u8> = &bincode::serialize(&de)
            .unwrap()
            .into_iter()
            .rev()
            .take(3)
            .rev()
            .collect();

        // assert: last 3 bytes, should all match.
        assert!(last3_ser == last3_after_ser);

        // assert: after serd signature is the same
        let pubk = private_key.public_key();
        let is_valid = pubk.verify(&de, &encoded_b);
        assert!(is_valid.is_ok())
    }

    #[test]
    fn poc_recv_key() {
        let b = Block::new("prev_hash".into(), 0, 0, vec![]);
        let encoded_b = &b.encode().unwrap();

        let private_key = PrivateKey::generate().unwrap();
        let signature = private_key.sign(&encoded_b).unwrap();

        let recid = RecoveryId::try_from(1u8); // 0, 1 work
        let recovered_key = VerifyingKey::recover_from_msg(
            &encoded_b.as_slice()[..],
            &signature.inner,
            recid.unwrap(),
        );
        if let Err(ref e) = recovered_key {
            dbg!(&e);
            assert!(false)
        }

        let valid = recovered_key
            .unwrap()
            .verify(&encoded_b[..], &signature.inner);
        assert!(valid.is_ok());
    }

    #[test]
    fn sign_block() {
        let b = Block::new("prev_hash".into(), 0, 0, vec![]);
        let encoded_b = &b.encode().unwrap();

        let private_key = PrivateKey::generate().unwrap();
        let signature = private_key.sign(&encoded_b).unwrap();
        assert_eq!(signature.as_bytes().is_empty(), false);

        let public_key = private_key.public_key();
        let result = public_key.verify(&signature, &encoded_b);
        assert!(result.is_ok());
    }
}
