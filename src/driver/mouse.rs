use fixedbitset::FixedBitSet;
use libkernel::flow::Message;

#[derive(Debug)]
pub struct MouseMessage {
    /// 0 - left button
    /// 1 - middle button
    /// 2 - right button
    /// ...
    pub keys_pressed: FixedBitSet,
    pub movement_x: i16,
    pub movement_y: i16,
}

impl Message for MouseMessage {}
