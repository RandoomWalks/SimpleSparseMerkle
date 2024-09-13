use bytes::Bytes;
use digest::Digest;
use std::collections::HashMap;

use crate::{kv_store::{KVStore,SimpleKVStore}, proof::MerkleProof, tree_hasher::TreeHasher};

pub struct SparseMerkleTree<S: KVStore> {
    pub(crate) hasher: TreeHasher<S::Hasher>,
    pub(crate) store: S,
    pub(crate) root: Bytes,
}

impl<S: KVStore> SparseMerkleTree<S> {
    pub fn new(store: S) -> Self {
        let hasher = TreeHasher::<S::Hasher>::new();
        let root = hasher.zero_value().clone();
        Self {
            hasher,
            store,
            root,
        }
    }

    pub fn get(&self, key: &[u8]) -> Result<Option<Bytes>, S::Error> {
        if self.root == self.hasher.zero_value() {
            return Ok(None);
        }
        let path = self.hasher.digest(key);
        self.store.get(&path)
    }

    pub fn update(&mut self, key: &[u8], value: Bytes) -> Result<(), S::Error> {
        let path = self.hasher.digest(key);
        let leaf_hash = self.hasher.digest_leaf(&path, &value);

        let mut current = leaf_hash.clone();
        self.store.set(path.clone().into(), value.clone())?;

        let zero_value = self.hasher.zero_value();
        let combined = [zero_value.as_ref(), zero_value.as_ref()].concat();
        self.store.set(current.clone(), Bytes::from(combined))?;

        for i in (0..256).rev() {
            let bit = (path[i / 8] >> (7 - (i % 8))) & 1;
            let sibling = self.hasher.zero_value();
            let (left, right) = if bit == 0 {
                (current, sibling)
            } else {
                (sibling, current)
            };
            current = self.hasher.digest_node(&left, &right);
            let combined = [left.as_ref(), right.as_ref()].concat();
            self.store.set(current.clone(), Bytes::from(combined))?;
        }

        self.root = current;
        Ok(())
    }

    pub fn remove(&mut self, key: &[u8]) -> Result<(), S::Error> {
        let path = self.hasher.digest(key);
        self.store.remove(&path)?;
        self.root = self.hasher.zero_value().clone();
        Ok(())
    }

    pub fn generate_proof(&self, key: &[u8]) -> Result<MerkleProof, S::Error> {
        let path = self.hasher.digest(key);
        let mut current = self.root.clone();
        let mut side_nodes = Vec::new();

        for i in 0..256 {
            if current == self.hasher.zero_value() {
                break;
            }

            let zero_value = self.hasher.zero_value();
            let default_combined = [zero_value.as_ref(), zero_value.as_ref()].concat();
            let node_value = self
                .store
                .get(&current)?
                .unwrap_or_else(|| Bytes::from(default_combined));
            let (left, right) = node_value.split_at(node_value.len() / 2);
            let bit = (path[i / 8] >> (7 - (i % 8))) & 1;

            if bit == 0 {
                side_nodes.push(Bytes::copy_from_slice(right));
                current = Bytes::copy_from_slice(left);
            } else {
                side_nodes.push(Bytes::copy_from_slice(left));
                current = Bytes::copy_from_slice(right);
            }
        }

        Ok(MerkleProof { side_nodes })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sha2::Sha256; // Use Sha256 as our hashing function

    #[test]
    fn test_insert_and_get() {
        let mut store = SimpleKVStore::<Sha256>::new();
        let mut smt = SparseMerkleTree::new(store);

        let key = b"key1";
        let value = Bytes::from("value1");

        // Insert value
        smt.update(key, value.clone()).unwrap();

        // Retrieve value
        let retrieved = smt.get(key).unwrap();
        assert_eq!(retrieved, Some(value));
    }

    #[test]
    fn test_update() {
        let mut store = SimpleKVStore::<Sha256>::new();
        let mut smt = SparseMerkleTree::new(store);

        let key = b"key1";
        let initial_value = Bytes::from("value1");
        let updated_value = Bytes::from("value2");

        // Insert initial value
        smt.update(key, initial_value.clone()).unwrap();
        assert_eq!(smt.get(key).unwrap(), Some(initial_value));

        // Update the value
        smt.update(key, updated_value.clone()).unwrap();
        assert_eq!(smt.get(key).unwrap(), Some(updated_value));
    }

    #[test]
    fn test_remove() {
        let mut store = SimpleKVStore::<Sha256>::new();
        let mut smt = SparseMerkleTree::new(store);

        let key = b"key1";
        let value = Bytes::from("value1");

        // Insert value
        smt.update(key, value.clone()).unwrap();
        assert_eq!(smt.get(key).unwrap(), Some(value));

        // Remove the value
        smt.remove(key).unwrap();
        assert_eq!(smt.get(key).unwrap(), None);
    }

    #[test]
    fn test_proof_verification() {
        let store = SimpleKVStore::<Sha256>::new();
        let mut smt = SparseMerkleTree::new(store);

        let key = b"key1";
        let value = Bytes::from("value1");

        // Insert value
        smt.update(key, value.clone()).unwrap();

        // Generate proof
        let proof = smt.generate_proof(key).unwrap();

        // Verify the proof against the current root
        assert!(proof.verify(smt.root.as_ref(), key, &value, &smt.hasher));

        // Test with an incorrect value
        let incorrect_value = Bytes::from("incorrect_value");
        assert!(!proof.verify(smt.root.as_ref(), key, &incorrect_value, &smt.hasher));
    }
}
