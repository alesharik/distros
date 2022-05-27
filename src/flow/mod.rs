macro_rules! register {
    (content $path:expr => $type:ident ($($args:tt)*)) => { {
        use alloc::sync::Arc;
        use spin::Mutex;
        use crate::flow::{FlowManager, ContentProvider};
        FlowManager::register_endpoint::<$type>(&$path, Arc::new(Mutex::new(ContentProvider::new($type::new($($args)*)))), None)?;
    } };
    (dynacontent $path:expr => $type:ident $fn:expr) => { {
        use alloc::sync::Arc;
        use spin::Mutex;
        use crate::flow::{FlowManager, DynamicContentProvider};
        FlowManager::register_endpoint::<$type>(&$path, Arc::new(Mutex::new(DynamicContentProvider::new($fn))), None)?;
    } };
    (var $path:expr => $type:ident $handler:expr) => { {
        use alloc::sync::Arc;
        use spin::Mutex;
        use crate::flow::{FlowManager, VarProvider};
        let provider = Arc::new(Mutex::new(VarProvider::new($handler)));
        FlowManager::register_endpoint::<$type>(&$path, provider.clone(), Some(provider))?;
    } };
    (serial $path:expr => $var:expr) => {
        crate::flow::register_serialized(&$path, &$var)
    }
}

mod content;
mod manager;
mod producer;
mod serde;
mod tree;
mod getter;
mod dynacontent;
mod var;

pub use self::serde::{register_serialized, FlowSerdeError};
pub use content::ContentProvider;
pub use dynacontent::DynamicContentProvider;
pub use manager::{FlowManager, FlowManagerError};
pub use producer::Producer;
pub use var::{VarProvider, VarHandler};