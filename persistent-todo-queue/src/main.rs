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

    let args: Vec<String> = std::env::args().collect();

    match args.get(1).map(|s| s.as_str()) {
        Some("add") => {
            let todo = Todo {
                id: (queue.len() + 1) as u64,
                created_at: SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
                description: args.get(2).expect("provide a description").clone(),
            };
            queue.enqueue(todo);
            queue.save().unwrap();
        }

        Some("next") => match queue.peek() {
            Some(todo) => println!("Next up: [{}] {}", todo.id, todo.description),
            None => println!("No tasks!"),
        },

        Some("done") => {
            if queue.is_empty() {
                println!("No tasks to complete!");
                return;
            }
            if let Some(todo) = queue.peek() {
                println!("About to complete: {}", todo.description);
            }
            match queue.dequeue() {
                Some(todo) => println!("Completed: [{}] {}", todo.id, todo.description),
                None => println!("No tasks!"),
            }
            queue.save().unwrap();
        }

        Some("list") => {
            if queue.is_empty() {
                println!("No tasks!");
                return;
            }
            for todo in &queue.items {
                println!("[{}] {}", todo.id, todo.description);
            }
        }

        _ => println!("Usage: Todo add | list | done"),
    }
}
