pub mod borsh_impl;
pub mod json_impl;
pub mod wincode_impl;

pub use borsh_impl::Borsh;
pub use json_impl::SerdeJson;
pub use wincode_impl::Wincode;

pub trait Serializer<T> {
    fn to_bytes(&self, value: &T) -> Result<Vec<u8>, Box<dyn std::error::Error>>;
    fn from_bytes(&self, bytes: &[u8]) -> Result<T, Box<dyn std::error::Error>>;
}
