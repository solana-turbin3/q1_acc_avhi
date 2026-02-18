use borsh::{BorshDeserialize, BorshSerialize};

#[derive(Debug, BorshSerialize, BorshDeserialize)]
pub struct Queue<T> {
    inbox: Vec<T>,
    outbox: Vec<T>,
    pub next_id: u64,
}

impl<T> Queue<T> {
    pub fn new() -> Self {
        Queue {
            inbox: Vec::new(),
            outbox: Vec::new(),
            next_id: 1,
        }
    }

    pub fn enqueue(&mut self, item: T) {
        self.inbox.push(item);
    }

    pub fn dequeue(&mut self) -> Option<T> {
        if self.outbox.is_empty() {
            while let Some(item) = self.inbox.pop() {
                self.outbox.push(item);
            }
        }
        self.outbox.pop()
    }

    pub fn peek(&mut self) -> Option<&T> {
        if self.outbox.is_empty() {
            while let Some(item) = self.inbox.pop() {
                self.outbox.push(item);
            }
        }
        self.outbox.last()
    }

    pub fn len(&self) -> usize {
        self.inbox.len() + self.outbox.len()
    }

    pub fn is_empty(&self) -> bool {
        self.inbox.is_empty() && self.outbox.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.outbox.iter().rev().chain(self.inbox.iter())
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
