use borsh::{BorshDeserialize, BorshSerialize};
use serde::{Serialize, de::DeserializeOwned};
use wincode::{SchemaRead, SchemaWrite, config::DefaultConfig};

trait Serializer<T> {
    fn to_bytes(&self, value: &T) -> Result<Vec<u8>, Box<dyn std::error::Error>>;
    fn from_bytes(&self, bytes: &[u8]) -> Result<T, Box<dyn std::error::Error>>;
}

fn main() {
    println!("Hello, world!");
}
