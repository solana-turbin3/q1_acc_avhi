use borsh::{BorshDeserialize, BorshSerialize};

use super::Serializer;

pub struct Borsh;

impl<T: BorshSerialize + BorshDeserialize> Serializer<T> for Borsh {
    fn to_bytes(&self, value: &T) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        borsh::to_vec(value).map_err(|e| e.into())
    }

    fn from_bytes(&self, bytes: &[u8]) -> Result<T, Box<dyn std::error::Error>> {
        borsh::from_slice(bytes).map_err(|e| e.into())
    }
}
