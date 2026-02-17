use generic_storage::models::Person;
use generic_storage::serializers::{Borsh, SerdeJson, Wincode};
use generic_storage::storage::Storage;

#[test]
fn test_borsh() {
    let person = Person { name: "avhi".to_string(), age: 21 };

    let mut storage = Storage::new(Borsh);
    assert_eq!(storage.has_data(), false);
    storage.save(&person).unwrap();
    assert_eq!(storage.load().unwrap(), person);
}

#[test]
fn test_wincode() {
    let person = Person { name: "avhi".to_string(), age: 21 };

    let mut storage = Storage::new(Wincode);
    assert_eq!(storage.has_data(), false);
    storage.save(&person).unwrap();
    assert_eq!(storage.load().unwrap(), person);
}

#[test]
fn test_serde_json() {
    let person = Person { name: "avhi".to_string(), age: 21 };

    let mut storage = Storage::new(SerdeJson);
    assert_eq!(storage.has_data(), false);
    storage.save(&person).unwrap();
    assert_eq!(storage.load().unwrap(), person);
}
