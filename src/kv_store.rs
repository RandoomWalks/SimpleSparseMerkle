use std::collections::HashMap;
use crate::Hash;

pub trait KVStore {
    type Error;

    fn get(&self, key: &Hash) -> Result<Option<Vec<u8>>, Self::Error>;
    fn set(&mut self, key: Hash, value: Vec<u8>) -> Result<(), Self::Error>;
}

pub struct InMemoryKVStore {
    store: HashMap<Hash, Vec<u8>>,
}

impl InMemoryKVStore {
    pub fn new() -> Self {
        Self { store: HashMap::new() }
    }
}

impl KVStore for InMemoryKVStore {
    type Error = std::io::Error;

    fn get(&self, key: &Hash) -> Result<Option<Vec<u8>>, Self::Error> {
        Ok(self.store.get(key).cloned())
    }

    fn set(&mut self, key: Hash, value: Vec<u8>) -> Result<(), Self::Error> {
        self.store.insert(key, value);
        Ok(())
    }
}