use crate::flow::{Provider, Consumer, Subscription, Sender, Message};
use alloc::boxed::Box;
use alloc::vec::Vec;
use alloc::sync::Arc;
use spin::RwLock;
use async_trait::async_trait;

struct ConsumerHolder<T: Message> {
    id: u64,
    consumer: Box<dyn Consumer<T>>
}

struct SubscriptionImpl<T: Message> {
    id: u64,
    consumers: Arc<RwLock<Vec<ConsumerHolder<T>>>>,
    dropped: bool
}

impl<T: Message> Subscription for SubscriptionImpl<T> {
    fn get_id(&self) -> u64 {
        self.id
    }

    fn cancel(mut self) {
        let mut consumers = self.consumers.write();
        if let Some(idx) = consumers.iter().position(|x| x.id == self.id) {
            consumers.remove(idx);
        }
        self.dropped = true
    }
}

impl<T: Message> Drop for SubscriptionImpl<T> {
    fn drop(&mut self) {
        if self.dropped {
            return
        }
        let mut consumers = self.consumers.write();
        if let Some(idx) = consumers.iter().position(|x| x.id == self.id) {
            consumers.remove(idx);
        }
    }
}

pub struct Producer<T: Message> {
    consumers: Arc<RwLock<Vec<ConsumerHolder<T>>>>,
    id_counter: u64,
}

impl<T: Message> Producer<T> {
    pub fn new() -> Producer<T> {
        Producer {
            consumers: Arc::new(RwLock::new(Vec::new())),
            id_counter: 0
        }
    }
}

impl<T: 'static + Message> Provider<T> for Producer<T> {
    fn add_consumer(&mut self, consumer: Box<dyn Consumer<T>>) -> Box<dyn Subscription> {
        let id = self.id_counter;
        self.id_counter += 1;
        let mut consumers = self.consumers.write();
        consumers.push(ConsumerHolder {
            id,
            consumer,
        });
        Box::new(SubscriptionImpl {id, consumers: self.consumers.clone(), dropped: false })
    }
}

#[async_trait]
impl<T: Message> Sender<T> for Producer<T> {
    async fn send(&mut self, message: T) {
        for consumer in self.consumers.read().iter() {
            let x = consumer.consumer.consume(&message);
            x.await;
        }
    }
}