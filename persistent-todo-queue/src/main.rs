mod queue;
mod todo;

use std::time::SystemTime;

use queue::Queue;
use todo::Todo;

fn main() {
    let mut queue: Queue<Todo> = Queue::load().unwrap_or_else(|_| Queue::new());

    let args: Vec<String> = std::env::args().collect();

    match args.get(1).map(|s| s.as_str()) {
        Some("add") => {
            let todo = Todo {
                id: queue.next_id,
                created_at: SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
                description: args.get(2).expect("provide a description").clone(),
            };
            queue.next_id += 1;
            queue.enqueue(todo);
            queue.save().unwrap();
            println!("Task added!");
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
            println!("{} task(s) pending:", queue.len());
            for todo in queue.iter() {
                println!("[{}] {}", todo.id, todo.description);
            }
        }

        _ => println!("Usage: todo <add|list|done|next>"),
    }
}
