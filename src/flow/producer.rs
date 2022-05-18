use alloc::boxed::Box;
use alloc::sync::Arc;
use alloc::vec::Vec;
use async_trait::async_trait;
use core::marker::PhantomData;
use libkernel::flow::{AnyConsumer, Message, Provider, Sender, Subscription};
use spin::RwLock;
use futures::future::FutureExt;
use thingbuf::Recycle;

struct MessageRecycle {}

impl Recycle<ProducerMessage> for MessageRecycle {
    fn new_element(&self) -> ProducerMessage {
        ProducerMessage::Noop
    }

    fn recycle(&self, element: &mut ProducerMessage) {
        *element = ProducerMessage::Noop;
    }
}

enum ProducerMessage {
    AddConsumer(ConsumerHolder),
    RemoveConsumer(u64),
    Send(Box<dyn Message>),
    Close,
    Noop
}

impl Default for ProducerMessage {
    fn default() -> Self {
        ProducerMessage::Noop
    }
}

struct ConsumerHolder {
    id: u64,
    consumer: Box<dyn AnyConsumer>,
}

struct SubscriptionImpl {
    id: u64,
    channel: thingbuf::mpsc::Sender<ProducerMessage, MessageRecycle>,
    dropped: bool,
}

impl Subscription for SubscriptionImpl {
    fn get_id(&self) -> u64 {
        self.id
    }

    fn cancel(mut self) {
        let channel = self.channel.clone();
        let id = self.id;
        spawn!(async move {
            if let Ok(_) = channel.send(ProducerMessage::RemoveConsumer(id)).await {}
        });
        self.dropped = true
    }
}

impl Drop for SubscriptionImpl {
    fn drop(&mut self) {
        if self.dropped {
            return;
        }
        let channel = self.channel.clone();
        let id = self.id;
        spawn!(async move {
            channel.send(ProducerMessage::RemoveConsumer(id)).await;
        });
    }
}

pub struct Producer<T: Message + 'static> {
    channel: thingbuf::mpsc::Sender<ProducerMessage, MessageRecycle>,
    id_counter: u64,
    msg_type: PhantomData<T>,
}

impl<T: Message + 'static> Producer<T> {
    pub fn new() -> Producer<T> {
        let (tx, rx) = thingbuf::mpsc::with_recycle(16, MessageRecycle {});
        spawn!(Self::start_handler(rx));
        Producer {
            channel: tx,
            id_counter: 0,
            msg_type: PhantomData::default(),
        }
    }

    async fn start_handler(rx: thingbuf::mpsc::Receiver<ProducerMessage, MessageRecycle>) {
        let mut consumers = Vec::<ConsumerHolder>::new();
        loop {
            let message = rx.recv().await;
            if let None = message {
                break
            }
            match message.unwrap() {
                ProducerMessage::Send(msg) => {
                    for consumer in consumers.iter() {
                        let x = consumer.consumer.consume_msg(msg.as_ref());
                        x.await;
                    }
                }
                ProducerMessage::AddConsumer(consumer) => {
                    consumers.push(consumer);
                }
                ProducerMessage::RemoveConsumer(id) => {
                    if let Some(idx) = consumers.iter().position(|c| c.id == id) {
                        consumers.remove(idx);
                    }
                }
                ProducerMessage::Close => {
                    break
                }
                ProducerMessage::Noop => {}
            }
        }
    }

    pub fn send_async(&self, message: T) {
        let channel = self.channel.clone();
        spawn!(async move {
            if let Ok(_) = channel.send(ProducerMessage::Send(Box::new(message))).await {}
        });
    }
}

impl<T: 'static + Message> Drop for Producer<T> {
    fn drop(&mut self) {
        let channel = self.channel.clone();
        spawn!(async move {
            if let Ok(_) = channel.send(ProducerMessage::Close).await {}
        });
    }
}

impl<T: 'static + Message> Provider for Producer<T> {
    fn add_consumer(&mut self, consumer: Box<dyn AnyConsumer>) -> Box<dyn Subscription> {
        let id = self.id_counter;
        self.id_counter += 1;
        let holder = ConsumerHolder { id, consumer };
        let channel = self.channel.clone();
        spawn!(async move {
            if let Ok(_) = channel.send(ProducerMessage::AddConsumer(holder)).await {}
        });
        Box::new(SubscriptionImpl {
            id,
            channel: self.channel.clone(),
            dropped: false,
        })
    }
}

#[async_trait]
impl<T: Message> Sender for Producer<T> {
    type Msg = T;

    async fn send(&mut self, message: T) {
        if let Ok(_) = self.channel.send(ProducerMessage::Send(Box::new(message))).await {}
    }
}
