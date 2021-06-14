use crate::flow::{Consumer, Message, Provider, Sender, Subscription};
use alloc::borrow::ToOwned;
use alloc::boxed::Box;
use alloc::string::String;
use alloc::sync::Arc;
use core::any::Any;
use core::fmt::{Debug, Formatter};
use rpds::HashTrieMapSync;
use spin::{Lazy, Mutex};

pub enum FlowManagerError {
    ProviderNotFound,
    WrongMessageType,
    SendNotSupported,
}

impl Debug for FlowManagerError {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            FlowManagerError::WrongMessageType => writeln!(f, "Wrong message type"),
            FlowManagerError::ProviderNotFound => writeln!(f, "Provider not found"),
            FlowManagerError::SendNotSupported => writeln!(f, "Send not supported"),
        }
    }
}

struct Endpoint<T: Message> {
    provider: Arc<Mutex<dyn Provider<T> + Send>>,
    sender: Option<Arc<Mutex<dyn Sender<T> + Send>>>,
}

struct FlowManagerInner {
    endpoints: HashTrieMapSync<String, Box<dyn Any + Send>>,
}

static INNER: Lazy<Mutex<FlowManagerInner>> = Lazy::new(|| {
    Mutex::new(FlowManagerInner {
        endpoints: HashTrieMapSync::new_sync(),
    })
});

pub struct FlowManager {}

impl FlowManager {
    pub fn subscribe<T: 'static + Message>(
        path: &str,
        consumer: Box<dyn Consumer<T>>,
    ) -> Result<Box<dyn Subscription>, FlowManagerError> {
        let inner = INNER.lock();
        match inner.endpoints.get(path) {
            Some(inner) => match inner.downcast_ref::<Endpoint<T>>() {
                Some(endpoint) => Ok(endpoint.provider.lock().add_consumer(consumer)),
                None => Err(FlowManagerError::WrongMessageType),
            },
            None => Err(FlowManagerError::ProviderNotFound),
        }
    }

    pub fn send_async<T: 'static + Message>(path: &str, message: T) {
        spawn!(FlowManager::send_async_inner(path.to_owned(), message));
    }

    async fn send_async_inner<T: 'static + Message>(path: String, message: T) {
        if let Err(e) = FlowManager::send(&path, message).await {
            error!("Error while sending message {:?}", e);
        }
    }

    pub async fn send<T: 'static + Message>(
        path: &str,
        message: T,
    ) -> Result<(), FlowManagerError> {
        let inner = INNER.lock();
        match inner.endpoints.get(path) {
            Some(inner) => match inner.downcast_ref::<Endpoint<T>>().as_ref() {
                Some(endpoint) => match &endpoint.sender {
                    Some(sender) => {
                        sender.lock().send(message).await;
                        Ok(())
                    }
                    None => Err(FlowManagerError::SendNotSupported),
                },
                None => Err(FlowManagerError::WrongMessageType),
            },
            None => Err(FlowManagerError::ProviderNotFound),
        }
    }

    pub fn register_endpoint<T: 'static + Message>(
        path: &str,
        provider: Arc<Mutex<dyn Provider<T> + Send>>,
        sender: Option<Arc<Mutex<dyn Sender<T> + Send>>>,
    ) {
        let mut inner = INNER.lock();
        inner
            .endpoints
            .insert_mut(path.to_owned(), Box::new(Endpoint { sender, provider }))
    }
}
