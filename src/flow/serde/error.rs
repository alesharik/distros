use crate::flow::FlowManagerError;
use alloc::fmt;
use alloc::prelude::v1::{String, ToString};
use core::fmt::Display;
use serde::ser;

#[derive(Clone, Debug, PartialEq)]
pub enum Error {
    Message(String),
    Unsupported,
    FlowManagerError(FlowManagerError),
}

impl ser::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Error::Message(msg.to_string())
    }
}

impl Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Message(msg) => formatter.write_str(msg),
            Error::Unsupported => formatter.write_str("unsupported"),
            Error::FlowManagerError(e) => write!(formatter, "{:?}", e),
        }
    }
}

impl From<FlowManagerError> for Error {
    fn from(err: FlowManagerError) -> Self {
        Error::FlowManagerError(err)
    }
}

pub type Result<T> = core::result::Result<T, Error>;
