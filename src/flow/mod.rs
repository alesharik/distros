use alloc::boxed::Box;
use async_trait::async_trait;
use core::fmt::Debug;

mod manager;
mod producer;

pub use manager::{FlowManager, FlowManagerError};
pub use producer::Producer;

pub trait Message: Send + Sync + Debug {}

#[async_trait]
pub trait Consumer<T: Message>: Sync + Send {
    async fn consume(&self, message: &T);

    async fn close(&self, sub: &Box<dyn Subscription>);
}

pub trait Subscription: Send + Sync {
    fn get_id(&self) -> u64;

    fn cancel(self);
}

pub trait Provider<T: Message>: Send + Sync {
    fn add_consumer(&mut self, consumer: Box<dyn Consumer<T>>) -> Box<dyn Subscription>;
}

#[async_trait]
pub trait Sender<T: Message>: Send + Sync {
    async fn send(&mut self, message: T);
}
