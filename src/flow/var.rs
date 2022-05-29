use alloc::boxed::Box;
use alloc::sync::Arc;
use core::marker::PhantomData;
use core::sync::atomic::{AtomicBool, Ordering};
use libkernel::flow::{AnyConsumer, Message, Provider, Sender, Subscription};
use async_trait::async_trait;
use spin::Mutex;
use crate::flow::{FlowManager, FlowManagerError};

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

pub trait ValHandler<T: Message + 'static>: Send + Sync {
    fn get(&self) -> T;
}

pub trait VarHandler<T: Message + 'static>: Send + Sync {
    fn get(&self) -> T;

    fn set(&self, v: T);
}

struct VarHandlerProxy<T: VarHandler<M>, M: Message + 'static> {
    handler: Arc<T>,
    message_type: PhantomData<M>
}

impl<T: VarHandler<M>, M: Message + 'static> ValHandler<M> for VarHandlerProxy<T, M> {
    fn get(&self) -> M {
        self.handler.get()
    }
}

pub struct ValProviderImpl<F: ValHandler<T> + 'static, T: Message + 'static> {
    handler: Arc<F>,
    message_type: PhantomData<T>,
}

impl<F: ValHandler<T> + 'static, T: Message + 'static> ValProviderImpl<F, T> {
    async fn send(handler: Arc<F>, consumer: Box<dyn AnyConsumer>, dropped: Arc<AtomicBool>) {
        if !dropped.load(Ordering::SeqCst) {
            let msg = handler.get();
            consumer.consume_msg(&msg).await;
            consumer.close_consumer(&SubscriptionImpl { dropped }).await;
        }
    }

    pub fn register(self, path: &str) -> Result<(), FlowManagerError> {
        let provider = Arc::new(Mutex::new(self));
        FlowManager::register_endpoint::<T>(path, provider, None)
    }
}

impl<F: ValHandler<T> + 'static, T: 'static + Message> Provider for ValProviderImpl<F, T> {
    fn add_consumer(&mut self, consumer: Box<dyn AnyConsumer>) -> Box<dyn Subscription> {
        let dropped = Arc::new(AtomicBool::new(false));
        crate::process::spawn_kernel("val_handler", ValProviderImpl::send(
            self.handler.clone(),
            consumer,
            dropped.clone(),
        ));
        Box::new(SubscriptionImpl { dropped })
    }
}

pub struct VarProviderImpl<F: VarHandler<T> + 'static, T: Message + 'static> {
    handler: Arc<F>,
    val_provider: ValProviderImpl<VarHandlerProxy<F, T>, T>
}

impl<F: VarHandler<T> + 'static, T: Message + 'static> VarProviderImpl<F, T> {
    pub fn register(self, path: &str) -> Result<(), FlowManagerError> {
        let provider = Arc::new(Mutex::new(self));
        FlowManager::register_endpoint::<T>(path, provider.clone(), Some(provider))
    }
}

impl<F: VarHandler<T> + 'static, T: 'static + Message> Provider for VarProviderImpl<F, T> {
    fn add_consumer(&mut self, consumer: Box<dyn AnyConsumer>) -> Box<dyn Subscription> {
        self.val_provider.add_consumer(consumer)
    }
}

#[async_trait]
impl<F: VarHandler<T> + 'static, T: 'static + Message> Sender for VarProviderImpl<F, T> {
    type Msg = T;

    async fn send(&mut self, message: Self::Msg) {
        self.handler.set(message);
    }
}

pub struct VarProvider {}

impl VarProvider {
    pub fn new_var<F: VarHandler<T> + 'static, T: Message + 'static>(handler: F) -> VarProviderImpl<F, T> {
        let handler = Arc::new(handler);
        VarProviderImpl {
            val_provider: ValProviderImpl {
                handler: Arc::new(VarHandlerProxy {
                    handler: handler.clone(),
                    message_type: PhantomData::default()
                }),
                message_type: PhantomData::default()
            },
            handler
        }
    }

    pub fn new_val<F: ValHandler<T> + 'static, T: Message + 'static>(handler: F) -> ValProviderImpl<F, T> {
        let handler = Arc::new(handler);
        ValProviderImpl {
            handler,
            message_type: PhantomData::default()
        }
    }
}