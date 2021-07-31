use alloc::boxed::Box;
use async_trait::async_trait;
use core::fmt::Debug;

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

        impl Message for $name {}
    };
}

macro_rules! register {
    (content $path:expr => $msg:expr) => {
        use alloc::sync::Arc;
        use spin::Mutex;
        use crate::flow::{FlowManager, ContentProvider};
        FlowManager::register_endpoint(&$path, Arc::new(Mutex::new(ContentProvider::new($msg))), None)?;
    };
}

mod tree;
mod manager;
mod producer;
mod content;
mod message;

pub use manager::{FlowManager, FlowManagerError};
pub use producer::Producer;
pub use content::ContentProvider;
pub use message::*;
use core::any::TypeId;

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
unsafe impl<T> AnyConsumer for T where T: Consumer {
    fn check_type(&self, msg_type: &TypeId) -> bool {
        msg_type == &TypeId::of::<<Self as Consumer>::Msg>()
    }

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