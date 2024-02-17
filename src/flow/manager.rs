use crate::flow::getter::getter;
use crate::flow::tree::{FlowTree, FlowTreeEndpoint, FlowTreeError};
use alloc::borrow::ToOwned;
use alloc::boxed::Box;
use alloc::string::String;
use alloc::sync::Arc;
use alloc::vec::Vec;
use core::any::TypeId;
use core::fmt::{Debug, Formatter};
use core::ops::DerefMut;
use libkernel::flow::{AnyConsumer, Message, Provider, Sender, Subscription};
use spin::{Lazy, Mutex, RwLock};

pub type ElementInfo = super::tree::ElementInfo;

#[derive(Clone, Eq, PartialEq)]
pub enum FlowManagerError {
    ProviderNotFound,
    WrongMessageType,
    SendNotSupported,
    AlreadyOccupied,
}

impl Debug for FlowManagerError {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            FlowManagerError::WrongMessageType => writeln!(f, "Wrong message type"),
            FlowManagerError::ProviderNotFound => writeln!(f, "Provider not found"),
            FlowManagerError::SendNotSupported => writeln!(f, "Send not supported"),
            FlowManagerError::AlreadyOccupied => writeln!(f, "Already occupied"),
        }
    }
}

struct FlowManagerInner {
    endpoints: FlowTree,
}

static INNER: Lazy<RwLock<FlowManagerInner>> = Lazy::new(|| {
    RwLock::new(FlowManagerInner {
        endpoints: FlowTree::new(),
    })
});

pub struct FlowManager {}

impl FlowManager {
    pub fn subscribe(
        path: &str,
        consumer: Box<dyn AnyConsumer>,
    ) -> Result<Box<dyn Subscription>, FlowManagerError> {
        let inner = INNER.read();
        match inner.endpoints.get(path) {
            Some(inner) => {
                if !consumer.check_type(&inner.message_type) {
                    Err(FlowManagerError::WrongMessageType)
                } else {
                    Ok(inner.provider.lock().add_consumer(consumer))
                }
            }
            None => Err(FlowManagerError::ProviderNotFound),
        }
    }

    pub async fn get<T: Message + Clone + 'static>(path: &str) -> Result<T, FlowManagerError> {
        let (tx, rx) = getter::<T>();
        let sub = FlowManager::subscribe(path, Box::new(tx))?;
        let result = rx
            .receive()
            .await
            .map_err(|_| FlowManagerError::WrongMessageType);
        drop(sub);
        result
    }

    pub fn send_async<T: 'static + Message>(path: &str, message: T) {
        spawn!("send" => FlowManager::send_async_inner(path.to_owned(), message));
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
        let inner = INNER.read();
        match inner.endpoints.get(path) {
            Some(endpoint) => match &endpoint.sender {
                Some(sender) => {
                    if endpoint.message_type != TypeId::of::<T>() {
                        Err(FlowManagerError::WrongMessageType)
                    } else {
                        let mut sender = sender.lock();
                        let sender: &mut dyn Sender<Msg = T> =
                            unsafe { core::mem::transmute(sender.deref_mut()) };
                        sender.send(message).await;
                        Ok(())
                    }
                }
                None => Err(FlowManagerError::SendNotSupported),
            },
            None => Err(FlowManagerError::ProviderNotFound),
        }
    }

    pub fn register_endpoint<T: 'static + Message>(
        path: &str,
        provider: Arc<Mutex<dyn Provider + Send>>,
        sender: Option<Arc<Mutex<dyn Sender<Msg = T> + Send>>>,
    ) -> Result<(), FlowManagerError> {
        let mut endpoint = FlowTreeEndpoint::new::<T>(provider);
        if let Some(sender) = sender {
            unsafe {
                endpoint.sender(core::mem::transmute(sender));
            }
        }
        let mut inner = INNER.write();
        inner
            .endpoints
            .put::<T>(path, endpoint)
            .map_err(|e| match e {
                FlowTreeError::AlreadyOccupied => FlowManagerError::AlreadyOccupied,
                FlowTreeError::WrongMessageType => unreachable!(),
            })
    }

    pub fn list(path: &str) -> Vec<ElementInfo> {
        let inner = INNER.read();
        inner.endpoints.list(path)
    }
}
