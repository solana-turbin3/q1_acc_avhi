use std::marker::PhantomData;

use borsh::{BorshDeserialize, BorshSerialize};
use serde::{Serialize, de::DeserializeOwned};
use wincode::{SchemaRead, SchemaWrite, config::DefaultConfig};

trait Serializer<T> {
    fn to_bytes(&self, value: &T) -> Result<Vec<u8>, Box<dyn std::error::Error>>;
    fn from_bytes(&self, bytes: &[u8]) -> Result<T, Box<dyn std::error::Error>>;
}

struct Borsh;
struct Wincode;
struct SerdeJson;

struct Storage<T, S> {
    data: Option<Vec<u8>>,
    serializer: S,
    _marker: PhantomData<T>,
}

impl<T: BorshSerialize + BorshDeserialize> Serializer<T> for Borsh {
    fn to_bytes(&self, value: &T) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        borsh::to_vec(value).map_err(|e| e.into())
    }

    fn from_bytes(&self, bytes: &[u8]) -> Result<T, Box<dyn std::error::Error>> {
        borsh::from_slice(bytes).map_err(|e| e.into())
    }
}

impl<T: SchemaWrite<DefaultConfig, Src = T> + for<'a> SchemaRead<'a, DefaultConfig, Dst = T>>
    Serializer<T> for Wincode
{
    fn to_bytes(&self, value: &T) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        wincode::serialize(value).map_err(|e| e.into())
    }
    fn from_bytes(&self, bytes: &[u8]) -> Result<T, Box<dyn std::error::Error>> {
        wincode::deserialize(bytes).map_err(|e| e.into())
    }
}

impl<T: Serialize + DeserializeOwned> Serializer<T> for SerdeJson {
    fn to_bytes(&self, value: &T) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        serde_json::to_vec(value).map_err(|e| e.into())
    }
    fn from_bytes(&self, bytes: &[u8]) -> Result<T, Box<dyn std::error::Error>> {
        serde_json::from_slice(bytes).map_err(|e| e.into())
    }
}

impl<T, S: Serializer<T>> Storage<T, S> {
    fn new(serializer: S) -> Self {
        Storage {
            data: None,
            serializer,
            _marker: PhantomData,
        }
    }

    fn has_data(&self) -> bool {
        self.data.is_some()
    }

    fn save(&mut self, value: &T) -> Result<(), Box<dyn std::error::Error>> {
        let bytes = self.serializer.to_bytes(value)?;
        self.data = Some(bytes);
        Ok(())
    }

    fn load(&self) -> Result<T, Box<dyn std::error::Error>> {
        match &self.data {
            Some(bytes) => self.serializer.from_bytes(bytes),
            None => Err("no data stored".into()),
        }
    }
}

fn main() {
    todo!();
}
