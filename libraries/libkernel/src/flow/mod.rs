use alloc::boxed::Box;
use async_trait::async_trait;
use core::any::TypeId;
use core::fmt::Debug;

#[macro_export]
macro_rules! primitive_message {
    ($name:ident $type:ty) => {
        #[derive(Clone, Debug)]
        pub struct $name($type);

        impl $name {
            pub fn new(message: $type) -> Self {
                $name(message)
            }

            pub fn get(&self) -> $type {
                self.0
            }
        }

        impl $crate::flow::Message for $name {}
    };
}

pub mod message;

pub use message::*;

pub trait Message: Send + Sync + Debug {}

#[async_trait]
pub unsafe trait AnyConsumer: Sync + Send {
    fn check_type(&self, msg_type: &TypeId) -> bool;

    async fn consume_msg(&self, message: &dyn Message);

    async fn close_consumer(&self, sub: &dyn Subscription);
}

#[async_trait]
pub trait Consumer: Sync + Send {
    type Msg: Message + 'static;

    async fn consume(&self, message: &Self::Msg);

    async fn close(&self, sub: &dyn Subscription);
}

#[async_trait]
unsafe impl<T> AnyConsumer for T
where
    T: Consumer,
{
    fn check_type(&self, msg_type: &TypeId) -> bool {
        msg_type == &TypeId::of::<<Self as Consumer>::Msg>()
    }

    #[allow(clippy::clone_double_ref)]
    async fn consume_msg(&self, message: &dyn Message) {
        let msg_ptr = unsafe {
            let ptr = message as *const dyn Message;
            let struct_ptr = ptr.to_raw_parts().0.cast::<<Self as Consumer>::Msg>();
            (&*struct_ptr).clone()
        };
        self.consume(msg_ptr).await;
    }

    async fn close_consumer(&self, sub: &dyn Subscription) {
        self.close(sub).await;
    }
}

pub trait Subscription: Send + Sync {
    fn get_id(&self) -> u64;

    fn cancel(self);
}

pub trait Provider: Send + Sync {
    fn add_consumer(&mut self, consumer: Box<dyn AnyConsumer>) -> Box<dyn Subscription>;
}

#[async_trait]
pub trait Sender: Send + Sync {
    type Msg: Message;

    async fn send(&mut self, message: Self::Msg);
}
