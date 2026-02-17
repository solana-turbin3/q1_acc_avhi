# Generic Storage with Multiple Serialization Formats

A generic storage system in Rust that can serialize and deserialize data using Borsh, Wincode, and JSON.

---

## Architecture

### Serializer Trait

A generic trait that defines how data moves to and from bytes.

```rust
pub trait Serializer<T> {
    fn to_bytes(&self, value: &T) -> Result<Vec<u8>, Box<dyn std::error::Error>>;
    fn from_bytes(&self, bytes: &[u8]) -> Result<T, Box<dyn std::error::Error>>;
}
```

### Storage Container

A generic struct that holds serialized bytes internally and uses `PhantomData<T>` to track the data type at compile time without storing it directly.

```rust
pub struct Storage<T, S> {
    data: Option<Vec<u8>>,
    serializer: S,
    _marker: PhantomData<T>,
}
```

- **T**: The data type being stored (e.g. `Person`)
- **S**: The serializer to use (e.g. `Borsh`, `Wincode`, `SerdeJson`)
- **data**: Raw bytes of the serialized value, `None` if nothing has been saved yet
- **_marker**: Zero-cost type marker, exists only at compile time

---

## Serializers

### Borsh

Uses the `borsh` crate. Requires `T: BorshSerialize + BorshDeserialize`.

```rust
impl<T: BorshSerialize + BorshDeserialize> Serializer<T> for Borsh {
    fn to_bytes(&self, value: &T) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        borsh::to_vec(value).map_err(|e| e.into())
    }

    fn from_bytes(&self, bytes: &[u8]) -> Result<T, Box<dyn std::error::Error>> {
        borsh::from_slice(bytes).map_err(|e| e.into())
    }
}
```

### Wincode

Uses the `wincode` crate. Requires `SchemaWrite<DefaultConfig, Src = T>` and `for<'a> SchemaRead<'a, DefaultConfig, Dst = T>`.

The `Src = T` and `Dst = T` constraints ensure the type serializes and deserializes into itself. The `for<'a>` higher-ranked trait bound means the type works with any input lifetime.

```rust
impl<T: SchemaWrite<DefaultConfig, Src = T> + for<'a> SchemaRead<'a, DefaultConfig, Dst = T>>
    Serializer<T> for Wincode
{
    fn to_bytes(&self, value: &T) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        wincode::serialize(value).map_err(|e| e.into())
    }

    fn from_bytes(&self, bytes: &[u8]) -> Result<T, Box<dyn std::error::Error>> {
        wincode::deserialize(bytes).map_err(|e| e.into())
    }
}
```

### SerdeJson

Uses the `serde_json` crate. Requires `T: Serialize + DeserializeOwned`.

`DeserializeOwned` is shorthand for `for<'de> Deserialize<'de>`, meaning the type owns all its data and does not borrow from the input bytes.

```rust
impl<T: Serialize + DeserializeOwned> Serializer<T> for SerdeJson {
    fn to_bytes(&self, value: &T) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        serde_json::to_vec(value).map_err(|e| e.into())
    }

    fn from_bytes(&self, bytes: &[u8]) -> Result<T, Box<dyn std::error::Error>> {
        serde_json::from_slice(bytes).map_err(|e| e.into())
    }
}
```

---

## Storage Methods

```rust
impl<T, S: Serializer<T>> Storage<T, S> {
    pub fn new(serializer: S) -> Self
    pub fn has_data(&self) -> bool
    pub fn save(&mut self, value: &T) -> Result<(), Box<dyn std::error::Error>>
    pub fn load(&self) -> Result<T, Box<dyn std::error::Error>>
}
```

- **new**: Creates an empty storage with a given serializer
- **has_data**: Returns `true` if data has been saved
- **save**: Serializes the value and stores the raw bytes
- **load**: Deserializes the stored bytes back into `T`

---

## Test Data Type

```rust
#[derive(
    Debug, Clone, PartialEq,
    SchemaRead, SchemaWrite,
    BorshSerialize, BorshDeserialize,
    serde::Serialize, serde::Deserialize,
)]
pub struct Person {
    pub name: String,
    pub age: u32,
}
```

---

## Usage

```rust
let person = Person { name: "avhi".to_string(), age: 21 };

// Borsh
let mut storage = Storage::new(Borsh);
storage.save(&person).unwrap();
let loaded = storage.load().unwrap();
assert_eq!(person, loaded);

// Wincode
let mut storage = Storage::new(Wincode);
storage.save(&person).unwrap();
let loaded = storage.load().unwrap();
assert_eq!(person, loaded);

// JSON
let mut storage = Storage::new(SerdeJson);
storage.save(&person).unwrap();
let loaded = storage.load().unwrap();
assert_eq!(person, loaded);
```

---

## Module Structure

```
src/
├── lib.rs                      -- exposes all modules
├── models.rs                   -- Person struct
├── storage.rs                  -- Storage<T, S> struct and methods
└── serializers/
    ├── mod.rs                  -- Serializer trait and re-exports
    ├── borsh_impl.rs           -- Borsh serializer
    ├── wincode_impl.rs         -- Wincode serializer
    └── json_impl.rs            -- SerdeJson serializer

tests/
└── integration.rs              -- integration tests for all three serializers
```

---

## Running Tests

```bash
cargo test
```

---

## Key Concepts

- **Generic traits**: `Serializer<T>` allows different implementations per type
- **PhantomData**: Zero-cost compile-time type marker for unused type parameters
- **Associated types**: `Src = T` and `Dst = T` constrain wincode to serialize a type into itself
- **HRTB**: `for<'a>` means a trait bound holds for any possible lifetime
- **DeserializeOwned**: Shorthand for `for<'de> Deserialize<'de>`, used when deserialized data owns all its fields
