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

#[derive(
    Debug,
    Clone,
    PartialEq,
    SchemaRead,
    SchemaWrite,
    BorshSerialize,
    BorshDeserialize,
    serde::Serialize,
    serde::Deserialize,
)]
struct Person {
    name: String,
    age: u32,
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
    let person = Person {
        name: "avhi".to_string(),
        age: 21,
    };

    let mut borsh_storage = Storage::new(Borsh);
    let data = borsh_storage.has_data();
    assert_eq!(data, false);
    borsh_storage.save(&person).unwrap();

    let load = borsh_storage.load().unwrap();
    assert_eq!(person, load);

    let mut serde_storage = Storage::new(SerdeJson);
    let data = serde_storage.has_data();
    assert_eq!(data, false);
    serde_storage.save(&person).unwrap();
    assert_eq!(serde_storage.load().unwrap(), person);

    let mut wincode_storage = Storage::new(Wincode);
    let data = wincode_storage.has_data();
    assert_eq!(data, false);
    wincode_storage.save(&person).unwrap();
    assert_eq!(wincode_storage.load().unwrap(), person);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_borsh() {
        let person = Person {
            name: "avhi".to_string(),
            age: 21,
        };

        let mut storage = Storage::new(Borsh);
        assert_eq!(storage.has_data(), false);
        storage.save(&person).unwrap();
        assert_eq!(storage.load().unwrap(), person);
    }

    #[test]
    fn test_wincode() {
        let person = Person {
            name: "avhi".to_string(),
            age: 21,
        };

        let mut storage = Storage::new(Borsh);
        assert_eq!(storage.has_data(), false);
        storage.save(&person).unwrap();
        assert_eq!(storage.load().unwrap(), person);
    }

    #[test]
    fn test_serde_json() {
        let person = Person {
            name: "avhi".to_string(),
            age: 21,
        };

        let mut storage = Storage::new(Borsh);
        assert_eq!(storage.has_data(), false);
        storage.save(&person).unwrap();
        assert_eq!(storage.load().unwrap(), person);
    }
}
