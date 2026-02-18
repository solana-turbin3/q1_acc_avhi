# Persistent Todo Queue

A CLI-based todo application in Rust that stores tasks in a FIFO queue and persists them to disk using Borsh serialization. Tasks survive program restarts and are processed in the order they were added.

---

## Architecture

### Todo

Represents a single task.

```rust
#[derive(Debug, BorshSerialize, BorshDeserialize)]
pub struct Todo {
    pub id: u64,
    pub description: String,
    pub created_at: u64,
}
```

- **id**: Auto-incrementing unique identifier, never reused even after tasks are completed
- **description**: The task text
- **created_at**: Unix timestamp when the task was added

---

### Queue

A generic FIFO queue implemented using two stacks instead of `VecDeque`.

```rust
pub struct Queue<T> {
    inbox: Vec<T>,
    outbox: Vec<T>,
    pub next_id: u64,
}
```

**Why two stacks?**

A regular `Vec` is fast at pushing/popping from the back but slow at removing from the front (O(n) shift). Two stacks simulate a queue with amortized O(1) for both enqueue and dequeue:

- `enqueue` pushes to `inbox`
- `dequeue`/`peek` transfers all items from `inbox` to `outbox` (reversing order) when `outbox` is empty, then pops from `outbox`

```
enqueue "A", "B", "C":
  inbox:  [A, B, C]
  outbox: []

dequeue (outbox empty, drain inbox):
  inbox:  []
  outbox: [C, B, A]   <- reversed
  pop outbox -> A     <- correct FIFO order
```

**Methods:**

| Method | Description |
|---|---|
| `new()` | Creates an empty queue |
| `enqueue(item)` | Adds item to the back |
| `dequeue()` | Removes and returns the front item |
| `peek()` | Returns a reference to the front item without removing it |
| `len()` | Returns total number of items |
| `is_empty()` | Returns true if no items |
| `iter()` | Iterates all items in FIFO order |

**Persistence:**

| Method | Description |
|---|---|
| `save()` | Serializes queue to `todo.bin` using Borsh |
| `load()` | Deserializes queue from `todo.bin` |

---

## CLI Commands

```bash
# Add a task
todo add "Buy groceries"

# List all pending tasks
todo list

# Preview the next task without completing it
todo next

# Complete the oldest task
todo done
```

---

## Usage Example

```bash
$ todo add "Buy groceries"
Task added!

$ todo add "Do laundry"
Task added!

$ todo add "Cook dinner"
Task added!

$ todo list
3 task(s) pending:
[1] Buy groceries
[2] Do laundry
[3] Cook dinner

$ todo next
Next up: [1] Buy groceries

$ todo done
About to complete: Buy groceries
Completed: [1] Buy groceries

$ todo list
2 task(s) pending:
[2] Do laundry
[3] Cook dinner
```

---

## Persistence

Tasks are serialized to `todo.bin` using Borsh after every `add` and `done` command. On startup the app loads `todo.bin` automatically. If the file does not exist a fresh empty queue is created.

---

## Module Structure

```
src/
├── main.rs       -- CLI args and command matching
├── todo.rs       -- Todo struct
└── queue.rs      -- Queue<T> with two-stack impl and save/load
```

---

## Build and Run

```bash
cargo build
./target/debug/persistent-todo-queue add "Buy groceries"
./target/debug/persistent-todo-queue list
./target/debug/persistent-todo-queue done
```
