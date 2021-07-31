use crate::flow::{Message, Provider, Subscription, AnyConsumer};
use core::sync::atomic::{AtomicBool, Ordering};
use alloc::boxed::Box;
use alloc::sync::Arc;

struct SubscriptionImpl {
    dropped: Arc<AtomicBool>,
}

impl Subscription for SubscriptionImpl {
    #[inline]
    fn get_id(&self) -> u64 {
        0
    }

    #[inline]
    fn cancel(self) {
        self.dropped.store(true, Ordering::SeqCst);
    }
}

impl Drop for SubscriptionImpl {
    fn drop(&mut self) {
        self.dropped.store(true, Ordering::SeqCst);
    }
}

pub struct ContentProvider<T: Message + 'static> {
    data: Arc<T>,
}

impl<T: Message + 'static> ContentProvider<T> {
    pub fn new(data: T) -> ContentProvider<T> {
        ContentProvider { data: Arc::new(data) }
    }

    async fn send(data: Arc<T>, consumer: Box<dyn AnyConsumer>, dropped: Arc<AtomicBool>) {
        use core::ops::Deref;

        if !dropped.load(Ordering::SeqCst) {
            consumer.consume_msg(data.deref()).await;
            consumer.close_consumer(&SubscriptionImpl { dropped }).await;
        }
    }
}

impl<T: 'static + Message> Provider for ContentProvider<T> {
    fn add_consumer(&mut self, consumer: Box<dyn AnyConsumer>) -> Box<dyn Subscription> {
        let dropped = Arc::new(AtomicBool::new(false));
        crate::futures::spawn(ContentProvider::send(self.data.clone(), consumer, dropped.clone()));
        Box::new(SubscriptionImpl { dropped })
    }
}
