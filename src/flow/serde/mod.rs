use crate::flow::serde::serializer::Serializer;
use serde::Serialize;

mod error;
mod serializer;

pub type FlowSerdeError = error::Error;

pub fn register_serialized<T>(path: &str, value: &T) -> core::result::Result<(), FlowSerdeError>
where
    T: Serialize,
{
    let mut serializer = Serializer::new(path);
    value.serialize(&mut serializer)?;
    Ok(())
}
