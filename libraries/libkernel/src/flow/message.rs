use super::Message;
use alloc::borrow::ToOwned;
use alloc::string::String;
use alloc::vec::Vec;

#[derive(Clone, Debug)]
pub struct StringMessage(String);

impl StringMessage {
    pub fn new(message: &str) -> Self {
        StringMessage(message.to_owned())
    }

    pub fn get(&self) -> &str {
        &self.0
    }
}

impl Message for StringMessage {}

#[derive(Clone, Debug)]
pub struct ByteArrayMessage(Vec<u8>);

impl ByteArrayMessage {
    pub fn new(message: &[u8]) -> Self {
        ByteArrayMessage(message.to_owned())
    }

    pub fn get(&self) -> &[u8] {
        &self.0
    }
}

impl Message for ByteArrayMessage {}

primitive_message!(I8Message i8);
primitive_message!(U8Message u8);
primitive_message!(U16Message u16);
primitive_message!(I16Message i16);
primitive_message!(U32Message u32);
primitive_message!(I32Message i32);
primitive_message!(U64Message u64);
primitive_message!(I64Message i64);
primitive_message!(F32Message f32);
primitive_message!(F64Message f64);
primitive_message!(CharMessage char);
primitive_message!(BoolMessage bool);
