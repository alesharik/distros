use crate::flow::{Consumer, Subscription, FlowManager};
use crate::driver::keyboard::KeyboardMessage;
use alloc::boxed::Box;
use async_trait::async_trait;
use crate::driver::mouse::MouseMessage;

pub mod keyboard;
pub mod mouse;
mod device;
mod pci;

struct KeyboardTestConsumer {}
struct KeyboardTestConsumer1 {}

#[async_trait]
impl Consumer<KeyboardMessage> for KeyboardTestConsumer {
    async fn consume(&self, message: &KeyboardMessage) {
        println!("{:?}", message.key);
    }

    async fn close(&self, sub: &Box<dyn Subscription>) {
        unreachable!()
    }
}

#[async_trait]
impl Consumer<MouseMessage> for KeyboardTestConsumer1 {
    async fn consume(&self, message: &MouseMessage) {
        println!("{:?}", message);
    }

    async fn close(&self, sub: &Box<dyn Subscription>) {
        unreachable!()
    }
}

pub fn init() {
    kblog!("Driver", "Starting device drivers");
    device::init();
    kblog!("Driver", "Device drivers started");

    let sub = FlowManager::subscribe("/dev/ps2/keyboard", Box::new(KeyboardTestConsumer {})).unwrap();
    Box::leak(sub); // keep consumer after function end
    let sub = FlowManager::subscribe("/dev/ps2/mouse", Box::new(KeyboardTestConsumer1 {})).unwrap();
    Box::leak(sub); // keep consumer after function end

    pci::print();
}