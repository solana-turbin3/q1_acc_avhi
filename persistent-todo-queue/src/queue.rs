use std::collections::VecDeque;

use borsh::{BorshDeserialize, BorshSerialize};

#[derive(Debug, BorshSerialize, BorshDeserialize)]
pub struct Queue<T> {
    pub items: VecDeque<T>,
    pub next_id: u64,
}

impl<T> Queue<T> {
    pub fn new() -> Self {
        Queue {
            items: VecDeque::new(),
            next_id: 1,
        }
    }

    pub fn enqueue(&mut self, item: T) {
        self.items.push_back(item)
    }

    pub fn dequeue(&mut self) -> Option<T> {
        self.items.pop_front()
    }

    pub fn peek(&self) -> Option<&T> {
        self.items.front()
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}

impl<T> Queue<T>
where
    T: BorshSerialize,
    T: BorshDeserialize,
{
    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let bytes = borsh::to_vec(self)?;
        std::fs::write("todo.bin", &bytes)?;
        Ok(())
    }

    pub fn load() -> Result<Queue<T>, Box<dyn std::error::Error>> {
        let bytes = std::fs::read("todo.bin")?;
        let queue = borsh::from_slice(&bytes)?;
        Ok(queue)
    }
}
