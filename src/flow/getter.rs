use libkernel::flow::{Consumer, Message, Subscription};
use futures::channel::oneshot::{Sender, Receiver, channel, Canceled};
use async_trait::async_trait;
use alloc::boxed::Box;
use spin::Mutex;

pub struct GetterReceiver<T: Message + 'static> {
    rx: Receiver<T>
}

impl<T: Message + 'static> GetterReceiver<T> {
    pub async fn receive(self) -> Result<T, Canceled> {
        self.rx.await
    }
}

pub struct GetterConsumer<T: Message + 'static> {
    tx: Mutex<Option<Sender<T>>>,
}

#[async_trait]
impl<T: Message + Clone + 'static> Consumer for GetterConsumer<T> {
    type Msg = T;

    async fn consume(&self, message: &Self::Msg) {
        let mut tx = self.tx.lock();
        if let Some(tx) = tx.take() {
            drop(tx.send(message.clone()));
        }
    }

    async fn close(&self, _sub: &dyn Subscription) {}
}

pub fn getter<T: Message + Clone + 'static>() -> (GetterConsumer<T>, GetterReceiver<T>) {
    let (tx, rx) = channel();
    (GetterConsumer { tx: Mutex::new(Some(tx)) }, GetterReceiver { rx })
}