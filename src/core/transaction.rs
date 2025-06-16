use super::cyphers::{Decoder, Encoder, Signature};
use serde::{Deserialize, Serialize};

// Transaction struct
// - Version no. Flag
// - In-counter: total number of inputs
// - Inputs: List of inputs
// - Out-counter:
// - Outputs:  List of outputs
// - LockTime
// - TxId: Hash (e.g., 256-bit hash) - The unique identifier of this entire transaction.
//         It's typically the cryptographic hash of the entire transaction's data structure (excluding the TxId itself).
//         This is calculated *after* the transaction is formed.

#[derive(Debug, Serialize, Deserialize, Clone)]
struct TransactionData {
    sender_addr: String,
    receiver_addr: String,
    value: f64,

    inputs: Vec<Transaction>,
    outputs: Vec<Transaction>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Transaction {
    pub trx_id: String,
    pub data: Vec<u8>,
    pub signature: Option<Signature>,
}

impl Encoder for Transaction {
    fn encode(&self) -> Result<Vec<u8>, String> {
        bincode::serialize(self).map_err(|e| e.to_string())
    }
}

impl Decoder for Transaction {
    fn decode(&self, encoded: &Vec<u8>) -> Result<Box<Self>, String> {
        bincode::deserialize(&encoded).map_err(|e| e.to_string())
    }
}

#[derive(Debug)]
pub enum TxBuilderErr {
    RequiredInputs,
    RequiredOutputs,
    SerializeFail,
    SignTxFail,
}
pub type TxBuilderResult = Result<Transaction, TxBuilderErr>;

pub struct TxBuilder {
    sender_addr: String,
    receiver_addr: String,
    value: f64,
    inputs: Option<Vec<Transaction>>,
    outputs: Option<Vec<Transaction>>,
    tx_id: Option<String>,
}

impl TxBuilder {
    pub fn new(sender_addr: String, receiver_addr: String, value: f64) -> Self {
        Self {
            sender_addr,
            receiver_addr,
            value,
            inputs: None,
            outputs: None,
            tx_id: None,
        }
    }

    pub fn inputs(mut self, inputs: Vec<Transaction>) -> Self {
        self.inputs = Some(inputs);
        self
    }

    pub fn outputs(mut self, outputs: Vec<Transaction>) -> Self {
        self.outputs = Some(outputs);
        self
    }

    pub fn build(self) -> Result<Transaction, TxBuilderErr> {
        if self.inputs.is_none() {
            return Err(TxBuilderErr::RequiredInputs);
        }
        if self.outputs.is_none() {
            return Err(TxBuilderErr::RequiredOutputs);
        }

        let tx_data = TransactionData {
            sender_addr: self.sender_addr,
            receiver_addr: self.receiver_addr,
            value: self.value,
            inputs: self.inputs.unwrap(),
            outputs: self.outputs.unwrap(),
        };
        let encoded_tx_data =
            bincode::serialize(&tx_data).map_err(|_| TxBuilderErr::SerializeFail)?;
        Ok(Transaction {
            trx_id: String::new(),
            data: encoded_tx_data,
            signature: None,
        })
    }
}
