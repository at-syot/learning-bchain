pub trait Encoder {
    fn encode(&self) -> Result<Vec<u8>, String>;
}

pub trait Decoder {
    fn decode(&self, encoded: &Vec<u8>) -> Result<Box<Self>, String>;
}
