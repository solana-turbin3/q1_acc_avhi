use borsh::{BorshDeserialize, BorshSerialize};
use wincode::{SchemaRead, SchemaWrite};

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
pub struct Person {
    pub name: String,
    pub age: u32,
}
