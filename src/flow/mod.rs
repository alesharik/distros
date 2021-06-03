use alloc::boxed::Box;
use core::fmt::Debug;
use async_trait::async_trait;

mod producer;
mod manager;

pub use producer::Producer;

pub trait Message: Send + Debug {}

#[async_trait]
pub trait Consumer<T: Message> {
    async fn consume(&self, message: &T);

    async fn close(&self, sub: &Box<dyn Subscription>);
}

pub trait Subscription {
    fn get_id(&self) -> u64;

    fn cancel(self);
}

pub trait Provider<T: Message> {
    fn add_consumer(&mut self, consumer: Box<dyn Consumer<T>>) -> Box<dyn Subscription>;
}

#[async_trait]
pub trait Sender<T: Message> {
    async fn send(&mut self, message: T);
}