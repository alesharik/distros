macro_rules! register {
    (serial $path:expr => $var:expr) => {
        crate::flow::register_serialized(&$path, &$var)
    };
    (var $path:expr => $type:ident $handler:expr) => { {
        use crate::flow::VarProvider;
        VarProvider::new_var($handler).register(&$path)?;
    } };
    (val $path:expr => $type:ident ($($args:tt)*)) => { {
        use crate::flow::{ValHandler, VarProvider};

        struct Handler { data: $type }
        impl ValHandler<$type> for Handler {
            fn get(&self) -> $type {
                self.data.clone()
            }
        }

        VarProvider::new_val(Handler { data: $type::new($($args)*) }).register(&$path)?;
    } };
    (val $path:expr => $type:ident fun $fn:expr) => { {
        use crate::flow::{ValHandler, VarProvider};

        struct Handler<F: Fn() -> $type + Send + Sync> { fun: F }
        impl<F: Fn() -> $type + Send + Sync> ValHandler<$type> for Handler<F> {
            fn get(&self) -> $type {
                (self.fun)()
            }
        }

        VarProvider::new_val(Handler { fun: $fn }).register(&$path)?;
    } };
}

mod manager;
mod producer;
mod serde;
mod tree;
mod getter;
mod var;

pub use self::serde::{register_serialized, FlowSerdeError};
pub use manager::{FlowManager, FlowManagerError};
pub use producer::Producer;
pub use var::{VarProvider, VarHandler, ValHandler};