use crate::flow::{Message, Provider, Sender, Subscription, AnyConsumer};
use alloc::boxed::Box;
use alloc::sync::Arc;
use alloc::vec::Vec;
use async_trait::async_trait;
use spin::RwLock;
use core::marker::PhantomData;

struct ConsumerHolder {
    id: u64,
    consumer: Box<dyn AnyConsumer>,
}

struct SubscriptionImpl {
    id: u64,
    consumers: Arc<RwLock<Vec<ConsumerHolder>>>,
    dropped: bool,
}

impl Subscription for SubscriptionImpl {
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

impl Drop for SubscriptionImpl {
    fn drop(&mut self) {
        if self.dropped {
            return;
        }
        let mut consumers = self.consumers.write();
        if let Some(idx) = consumers.iter().position(|x| x.id == self.id) {
            consumers.remove(idx);
        }
    }
}

pub struct Producer<T: Message + 'static> {
    consumers: Arc<RwLock<Vec<ConsumerHolder>>>,
    id_counter: u64,
    msg_type: PhantomData<T>,
}

impl<T: Message + 'static> Producer<T> {
    pub fn new() -> Producer<T> {
        Producer {
            consumers: Arc::new(RwLock::new(Vec::new())),
            id_counter: 0,
            msg_type: PhantomData::default(),
        }
    }

    pub fn send_async(&self, message: T) {
        spawn!(Producer::send_async_inner(message, self.consumers.clone()));
    }

    async fn send_async_inner(message: T, consumers: Arc<RwLock<Vec<ConsumerHolder>>>) {
        for consumer in consumers.read().iter() {
            let x = consumer.consumer.consume_msg(&message);
            x.await;
        }
    }
}

impl<T: 'static + Message> Provider for Producer<T> {
    fn add_consumer(&mut self, consumer: Box<dyn AnyConsumer>) -> Box<dyn Subscription> {
        let id = self.id_counter;
        self.id_counter += 1;
        let mut consumers = self.consumers.write();
        consumers.push(ConsumerHolder { id, consumer });
        Box::new(SubscriptionImpl {
            id,
            consumers: self.consumers.clone(),
            dropped: false,
        })
    }
}

#[async_trait]
impl<T: Message> Sender for Producer<T> {
    type Msg = T;

    async fn send(&mut self, message: T) {
        for consumer in self.consumers.read().iter() {
            let x = consumer.consumer.consume_msg(&message);
            x.await;
        }
    }
}
