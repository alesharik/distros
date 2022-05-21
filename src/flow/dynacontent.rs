use alloc::boxed::Box;
use alloc::sync::Arc;
use core::sync::atomic::{AtomicBool, Ordering};
use libkernel::flow::{AnyConsumer, Message, Provider, Subscription};

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

pub struct DynamicContentProvider<F: Fn() -> T + Send + Sync + 'static, T: Message + 'static> {
    provider: Arc<F>,
}

impl<F: Fn() -> T + Send + Sync + 'static, T: Message + 'static> DynamicContentProvider<F, T> {
    pub fn new(provider: F) -> DynamicContentProvider<F, T> {
        DynamicContentProvider {
            provider: Arc::new(provider),
        }
    }

    async fn send(provider: Arc<F>, consumer: Box<dyn AnyConsumer>, dropped: Arc<AtomicBool>) {
        use core::ops::Deref;

        if !dropped.load(Ordering::SeqCst) {
            let msg = provider();
            consumer.consume_msg(&msg).await;
            consumer.close_consumer(&SubscriptionImpl { dropped }).await;
        }
    }
}

impl<F: Fn() -> T + Send + Sync + 'static, T: 'static + Message> Provider for DynamicContentProvider<F, T> {
    fn add_consumer(&mut self, consumer: Box<dyn AnyConsumer>) -> Box<dyn Subscription> {
        let dropped = Arc::new(AtomicBool::new(false));
        crate::futures::spawn(DynamicContentProvider::send(
            self.provider.clone(),
            consumer,
            dropped.clone(),
        ));
        Box::new(SubscriptionImpl { dropped })
    }
}
