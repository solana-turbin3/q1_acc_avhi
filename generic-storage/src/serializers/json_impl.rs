use serde::{Serialize, de::DeserializeOwned};

use super::Serializer;

pub struct SerdeJson;

impl<T: Serialize + DeserializeOwned> Serializer<T> for SerdeJson {
    fn to_bytes(&self, value: &T) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        serde_json::to_vec(value).map_err(|e| e.into())
    }

    fn from_bytes(&self, bytes: &[u8]) -> Result<T, Box<dyn std::error::Error>> {
        serde_json::from_slice(bytes).map_err(|e| e.into())
    }
}
