use alloc::boxed::Box;
use alloc::sync::Arc;
use core::marker::PhantomData;
use core::sync::atomic::{AtomicBool, Ordering};
use libkernel::flow::{AnyConsumer, Message, Provider, Sender, Subscription};
use async_trait::async_trait;

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

pub trait VarHandler<T: Message + 'static>: Send + Sync {
    fn get(&self) -> T;

    fn set(&self, v: T);
}

pub struct VarProvider<F: VarHandler<T> + 'static, T: Message + 'static> {
    handler: Arc<F>,
    message_type: PhantomData<T>,
}

impl<F: VarHandler<T> + 'static, T: Message + 'static> VarProvider<F, T> {
    pub fn new(handler: F) -> VarProvider<F, T> {
        VarProvider {
            handler: Arc::new(handler),
            message_type: PhantomData::default()
        }
    }

    async fn send(handler: Arc<F>, consumer: Box<dyn AnyConsumer>, dropped: Arc<AtomicBool>) {
        use core::ops::Deref;

        if !dropped.load(Ordering::SeqCst) {
            let msg = handler.get();
            consumer.consume_msg(&msg).await;
            consumer.close_consumer(&SubscriptionImpl { dropped }).await;
        }
    }
}

impl<F: VarHandler<T> + 'static, T: 'static + Message> Provider for VarProvider<F, T> {
    fn add_consumer(&mut self, consumer: Box<dyn AnyConsumer>) -> Box<dyn Subscription> {
        let dropped = Arc::new(AtomicBool::new(false));
        crate::futures::spawn(VarProvider::send(
            self.handler.clone(),
            consumer,
            dropped.clone(),
        ));
        Box::new(SubscriptionImpl { dropped })
    }
}

#[async_trait]
impl<F: VarHandler<T> + 'static, T: 'static + Message> Sender for VarProvider<F, T> {
    type Msg = T;

    async fn send(&mut self, message: Self::Msg) {
        self.handler.set(message);
    }
}