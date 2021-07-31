use crate::flow::FlowManagerError;
use serde::Serialize;
use crate::flow::serde::serializer::Serializer;
use alloc::borrow::ToOwned;

mod error;
mod serializer;

pub type FlowSerdeError = error::Error;

pub fn register_serialized<T>(path: &str, value: &T) -> core::result::Result<(), FlowSerdeError>
where
    T: Serialize {
    let mut serializer = Serializer::new(path);
    value.serialize(&mut serializer)?;
    Ok(())
}