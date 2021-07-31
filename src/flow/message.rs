use alloc::string::String;
use crate::flow::Message;
use alloc::borrow::ToOwned;

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

primitive_message!(U8Message u8);
primitive_message!(U16Message u16);