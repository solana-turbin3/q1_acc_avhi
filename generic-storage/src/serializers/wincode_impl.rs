use wincode::{SchemaRead, SchemaWrite, config::DefaultConfig};

use super::Serializer;

pub struct Wincode;

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
