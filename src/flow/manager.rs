use alloc::boxed::Box;
use crate::flow::{Provider, Sender, Message, Consumer, Subscription};
use alloc::sync::Arc;
use rpds::HashTrieMapSync;
use alloc::string::String;
use core::any::Any;
use spin::{Mutex, RwLock};
use alloc::borrow::ToOwned;

pub enum FlowManagerError {
    ProviderNotFound,
    WrongMessageType,
    SendNotSupported
}

struct Endpoint<T: Message> {
    provider: Arc<RwLock<dyn Provider<T> + Send>>,
    sender: Option<Arc<RwLock<dyn Sender<T> + Send>>>
}

struct FlowManagerInner {
    endpoints: HashTrieMapSync<String, RwLock<Box<dyn Any + Send>>>
}

lazy_static!(
    static ref INNER: Mutex<Option<FlowManagerInner>> = Mutex::new(None);
);

pub struct FlowManager {}

impl FlowManager {
    pub fn init() {
        let mut inner = INNER.lock();
        *inner = Some(FlowManagerInner {
            endpoints: HashTrieMapSync::new_sync()
        });
        kblog!("FlowManager", "FlowManager started");
    }

    pub fn subscribe<T: 'static + Message>(path: &str, consumer: Box<dyn Consumer<T>>) -> Result<Box<dyn Subscription>, FlowManagerError> {
        let mut inner = INNER.lock();
        match inner.as_mut().expect("FlowManager not initialized").endpoints.get(path) {
            Some(inner) => {
                match inner.write().downcast_mut::<Endpoint<T>>() {
                    Some(endpoint) => Ok(endpoint.provider.write().add_consumer(consumer)),
                    None => Err(FlowManagerError::WrongMessageType)
                }
            },
            None => Err(FlowManagerError::ProviderNotFound)
        }
    }

    pub async fn send<T: 'static + Message>(path: &str, message: T) -> Result<(), FlowManagerError> {
        let mut inner = INNER.lock();
        match inner.as_mut().expect("FlowManager not initialized").endpoints.get(path) {
            Some(inner) => {
                match inner.write().downcast_mut::<Endpoint<T>>() {
                    Some(endpoint) => match &mut endpoint.sender {
                        Some(sender) => {
                            sender.write().send(message).await;
                            Ok(())
                        },
                        None => Err(FlowManagerError::SendNotSupported)
                    },
                    None => Err(FlowManagerError::WrongMessageType)
                }
            },
            None => Err(FlowManagerError::ProviderNotFound)
        }
    }

    pub fn register_endpoint<T: 'static + Message>(path: &str, provider: Arc<RwLock<dyn Provider<T> + Send>>, sender: Option<Arc<RwLock<dyn Sender<T> + Send>>>) {
        let mut inner = INNER.lock();
        inner.as_mut().expect("FlowManager not initialized").endpoints.insert_mut(path.to_owned(), RwLock::new(Box::new(Endpoint {
            sender,
            provider
        })))
    }
}