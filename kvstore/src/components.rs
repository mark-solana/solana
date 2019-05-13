use std::fs::File;
use std::ops::Deref;
use std::path::PathBuf;

use bytes::{Bytes, BytesMut};

use evmap::shallow_copy::ShallowCopy;

use specs::{Component, VecStorage};

#[cfg(test)]
pub use self::arb::*;

#[derive(Component, Debug)]
#[storage(VecStorage)]
pub struct Location {
    pub parent: PathBuf,
    pub path: PathBuf,
}

#[derive(Component, Debug)]
#[storage(VecStorage)]
pub struct Cache {
    pub key: Bytes,
    pub value: Option<Bytes>,
}

#[derive(
    Component, Clone, Default, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize,
)]
#[storage(VecStorage)]
pub struct Key {
    pub bytes: BytesMut,
}

#[derive(Component, Default, Debug, Serialize, Deserialize)]
#[storage(VecStorage)]
pub struct Index {
    pub offset: u64,
    pub size: u64,
}

#[derive(Component, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[storage(VecStorage)]
pub struct Value {
    pub bytes: Bytes,
}

#[derive(Component, Debug)]
#[storage(VecStorage)]
pub struct LogFile {
    pub file: File,
    pub path: PathBuf,
}

impl Value {
    pub fn new<T>(bytes: Option<T>) -> Self
    where
        T: Into<Bytes>,
    {
        Value {
            bytes: bytes.map(Into::into),
        }
    }
}

impl Deref for Key {
    type Target = BytesMut;

    fn deref(&self) -> &BytesMut {
        &self.bytes
    }
}

impl<T> From<T> for Key
where
    T: Into<BytesMut>,
{
    fn from(buf: T) -> Self {
        Key { bytes: buf.into() }
    }
}

impl<T> From<T> for Value
where
    T: Into<Bytes>,
{
    fn from(buf: T) -> Self {
        Value {
            bytes: Some(buf.into()),
        }
    }
}

impl ShallowCopy for Value {
    unsafe fn shallow_copy(&mut self) -> Self {
        self.clone()
    }
}

#[cfg(test)]
mod arb {
    use super::*;
    use proptest::prelude::*;

    pub fn arb_bytes(size: usize) -> impl Strategy<Value = Bytes> {
        prop::collection::vec(0u8..255, size).prop_map(Bytes::from)
    }

    pub fn arb_key(size: usize) -> impl Strategy<Value = Key> {
        prop::collection::vec(0u8..255, size).prop_map(Key::from)
    }

    pub fn arb_value(size: usize) -> impl Strategy<Value = Value> {
        arb_bytes(size).prop_map(Value::new)
    }
}
