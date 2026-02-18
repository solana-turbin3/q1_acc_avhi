use std::marker::PhantomData;

use crate::serializers::Serializer;

pub struct Storage<T, S> {
    data: Option<Vec<u8>>,
    serializer: S,
    _marker: PhantomData<T>,
}

impl<T, S> Storage<T, S>
where
    S: Serializer<T>,
{
    pub fn new(serializer: S) -> Self {
        Storage {
            data: None,
            serializer,
            _marker: PhantomData,
        }
    }

    pub fn has_data(&self) -> bool {
        self.data.is_some()
    }

    pub fn save(&mut self, value: &T) -> Result<(), Box<dyn std::error::Error>> {
        let bytes = self.serializer.to_bytes(value)?;
        self.data = Some(bytes);
        Ok(())
    }

    pub fn load(&self) -> Result<T, Box<dyn std::error::Error>> {
        match &self.data {
            Some(bytes) => self.serializer.from_bytes(bytes),
            None => Err("no data stored".into()),
        }
    }
}
