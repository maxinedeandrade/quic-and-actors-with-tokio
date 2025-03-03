#[repr(u32)]
#[derive(Debug, bitcode::Decode, bitcode::Encode)]
pub enum Message {
  Bark,
  Meow,
}
