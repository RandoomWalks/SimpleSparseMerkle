
use bytes::Bytes;
use digest::Digest;
use std::collections::HashMap;

/// Trait for a simple key-value store abstraction
pub trait KVStore {
    type Hasher: Digest;
    type Error;

    fn get(&self, key: &[u8]) -> Result<Option<Bytes>, Self::Error>;
    fn set(&mut self, key: Bytes, value: Bytes) -> Result<(), Self::Error>;
    fn remove(&mut self, key: &[u8]) -> Result<(), Self::Error>;
}



/// Simple in-memory key-value store for testing purposes
pub struct SimpleKVStore<H: Digest> {
    map: HashMap<Vec<u8>, Bytes>, // Use Vec<u8> as the key type for the HashMap
    _marker: core::marker::PhantomData<H>,
}

impl<H: Digest> SimpleKVStore<H> {
    /// Creates a new SimpleKVStore instance
    pub fn new() -> Self {
        SimpleKVStore {
            map: HashMap::new(),
            _marker: core::marker::PhantomData,
        }
    }
}

// Implement the KVStore trait for SimpleKVStore
impl<H: Digest> KVStore for SimpleKVStore<H> {
    type Hasher = H;
    type Error = String; // Using String as a simplified error type for demonstration

    /// Retrieves a value by key
    fn get(&self, key: &[u8]) -> Result<Option<Bytes>, Self::Error> {
        Ok(self.map.get(key).cloned()) // Return a clone of the value if it exists
    }

    /// Inserts or updates a value by key
    fn set(&mut self, key: Bytes, value: Bytes) -> Result<(), Self::Error> {
        self.map.insert(key.to_vec(), value); // Insert or update the key-value pair
        Ok(())
    }

    /// Removes a value by key
    fn remove(&mut self, key: &[u8]) -> Result<(), Self::Error> {
        self.map.remove(key); // Remove the key-value pair if it exists
        Ok(())
    }
}
