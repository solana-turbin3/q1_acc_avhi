use borsh::{BorshDeserialize, BorshSerialize};

#[derive(Debug, BorshSerialize, BorshDeserialize)]
struct Todo {
    id: u64,
    description: String,
    created_at: u64,
}

fn main() {
    println!("Hello, world!");
}
