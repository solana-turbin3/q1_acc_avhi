use std::{collections::VecDeque, time::SystemTime};

use borsh::{BorshDeserialize, BorshSerialize};

#[derive(Debug, BorshSerialize, BorshDeserialize)]
struct Todo {
    id: u64,
    description: String,
    created_at: u64,
}

#[derive(Debug, BorshSerialize, BorshDeserialize)]
struct Queue<T> {
    items: VecDeque<T>,
}

impl<T> Queue<T> {
    fn new() -> Self {
        Queue {
            items: VecDeque::new(),
        }
    }
    fn enqueue(&mut self, items: T) {
        self.items.push_back(items)
    }
    fn dequeue(&mut self) -> Option<T> {
        self.items.pop_front()
    }
    fn peek(&self) -> Option<&T> {
        self.items.front()
    }
    fn len(&self) -> usize {
        self.items.len()
    }
    fn is_empty(&self) -> bool {
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
        std::fs::write("todo.bin", &bytes).unwrap();
        Ok(())
    }

    pub fn load() -> Result<Queue<T>, Box<dyn std::error::Error>> {
        let bytes = std::fs::read("todo.bin")?;
        let queue = borsh::from_slice(&bytes)?;
        Ok(queue)
    }
}

fn main() {
    let mut queue: Queue<Todo> = Queue::load().unwrap_or_else(|_| Queue::new());

    queue.enqueue(Todo {
        id: 1,
        description: "test".to_string(),
        created_at: 0,
    });

    queue.save().unwrap();

    println!("saved! check if todos.bin exists");
}
