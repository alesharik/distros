use crate::flow::{Consumer, Message, Provider, Sender, Subscription};
use alloc::borrow::ToOwned;
use alloc::boxed::Box;
use alloc::string::String;
use alloc::sync::Arc;
use core::fmt::{Debug, Formatter};
use spin::{Lazy, Mutex};
use crate::flow::tree::{FlowTree, FlowTreeError, FlowTreeEndpoint};
use alloc::vec::Vec;

pub type ElementInfo = super::tree::ElementInfo;

pub enum FlowManagerError {
    ProviderNotFound,
    WrongMessageType,
    SendNotSupported,
    AlreadyOccupied
}

impl Debug for FlowManagerError {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            FlowManagerError::WrongMessageType => writeln!(f, "Wrong message type"),
            FlowManagerError::ProviderNotFound => writeln!(f, "Provider not found"),
            FlowManagerError::SendNotSupported => writeln!(f, "Send not supported"),
            FlowManagerError::AlreadyOccupied => writeln!(f, "Already occupied")
        }
    }
}

struct FlowManagerInner {
    endpoints: FlowTree,
}

static INNER: Lazy<Mutex<FlowManagerInner>> = Lazy::new(|| {
    Mutex::new(FlowManagerInner {
        endpoints: FlowTree::new(),
    })
});

pub struct FlowManager {}

impl FlowManager {
    pub fn subscribe<T: 'static + Message>(
        path: &str,
        consumer: Box<dyn Consumer<T>>,
    ) -> Result<Box<dyn Subscription>, FlowManagerError> {
        let inner = INNER.lock();
        match inner.endpoints.get::<T>(path).map_err(|e| match e {
            FlowTreeError::WrongMessageType => FlowManagerError::WrongMessageType,
            FlowTreeError::AlreadyOccupied => unreachable!()
        })? {
            Some(inner) => Ok(inner.provider.lock().add_consumer(consumer)),
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
        match inner.endpoints.get::<T>(path).map_err(|e| match e {
            FlowTreeError::WrongMessageType => FlowManagerError::WrongMessageType,
            FlowTreeError::AlreadyOccupied => unreachable!()
        })? {
            Some(endpoint) => match &endpoint.sender {
                Some(sender) => {
                    sender.lock().send(message).await;
                    Ok(())
                }
                None => Err(FlowManagerError::SendNotSupported),
            },
            None => Err(FlowManagerError::ProviderNotFound),
        }
    }

    pub fn register_endpoint<T: 'static + Message>(
        path: &str,
        provider: Arc<Mutex<dyn Provider<T> + Send>>,
        sender: Option<Arc<Mutex<dyn Sender<T> + Send>>>,
    ) -> Result<(), FlowManagerError> {
        let mut endpoint = FlowTreeEndpoint::new(provider);
        if let Some(sender) = sender {
            endpoint.sender(sender);
        }
        let mut inner = INNER.lock();
        inner.endpoints.put(path, endpoint).map_err(|e| match e {
            FlowTreeError::AlreadyOccupied => FlowManagerError::AlreadyOccupied,
            FlowTreeError::WrongMessageType => unreachable!()
        })
    }

    pub fn list(path: &str) -> Vec<ElementInfo> {
        let inner = INNER.lock();
        inner.endpoints.list(path)
    }
}
