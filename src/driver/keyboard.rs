use pc_keyboard::DecodedKey;
use crate::flow::Message;

#[derive(Debug)]
pub struct KeyboardMessage {
    pub key: DecodedKey
}

impl Message for KeyboardMessage {}