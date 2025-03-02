#[derive(Debug, bitcode::Decode, bitcode::Encode)]
pub enum Message {
  Identify,
}
