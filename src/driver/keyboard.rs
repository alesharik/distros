use libkernel::flow::Message;
use pc_keyboard::DecodedKey;

#[derive(Debug)]
pub struct KeyboardMessage {
    pub key: DecodedKey,
}

impl Message for KeyboardMessage {}
