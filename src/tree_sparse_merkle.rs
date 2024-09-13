// error.rs
use jemallocator::Jemalloc;
use rand::Rng;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SMTError {
    #[error("Key-value store error: {0}")]
    KVStoreError(#[from] std::io::Error),

    #[error("Invalid proof")]
    InvalidProof,

    #[error("Unsupported operation")]
    UnsupportedOperation,
}

// kv_store.rs
use std::collections::BTreeMap;
// use crate::Hash;

pub trait KVStore {
    type Error;

    fn get(&self, key: &Hash) -> Result<Option<&Vec<u8>>, Self::Error>;
    fn set(&mut self, key: Hash, value: Vec<u8>) -> Result<(), Self::Error>;
}

pub struct InMemoryKVStore {
    store: BTreeMap<Hash, Vec<u8>>,
}

impl InMemoryKVStore {
    pub fn new() -> Self {
        Self {
            store: BTreeMap::new(),
        }
    }
}

impl KVStore for InMemoryKVStore {
    type Error = std::io::Error;

    fn get(&self, key: &Hash) -> Result<Option<&Vec<u8>>, Self::Error> {
        Ok(self.store.get(key))
    }

    fn set(&mut self, key: Hash, value: Vec<u8>) -> Result<(), Self::Error> {
        self.store.insert(key, value);
        Ok(())
    }
}

// sparse_merkle_tree.rs
// use crate::{kv_store::KVStore, proof::MerkleProof, tree_hasher::TreeHasher, DefaultHasher, Hash};
use tracing::{debug, error, info, warn};

pub struct SparseMerkleTree<S: KVStore> {
    pub(crate) hasher: TreeHasher<DefaultHasher>,
    pub(crate) store: S,
    pub(crate) root: Hash,
}

impl<S: KVStore> SparseMerkleTree<S> {
    pub fn new(store: S) -> Self {
        let hasher = TreeHasher::<DefaultHasher>::new();
        let root = [0u8; 32];
        info!("Created new Sparse Merkle Tree");
        Self {
            hasher,
            store,
            root,
        }
    }

    pub fn update(&mut self, key: Hash, value: Hash) -> Result<(), S::Error> {
        info!("Updating tree with key {:?}, value {:?}", key, value);
        let leaf_hash = self.hasher.digest_leaf(&key, &value);
        self.store.set(key, value.to_vec())?;
        debug!("Set key-value pair in store");

        let mut current = leaf_hash;
        for i in (0..256).rev() {
            let bit = (key[i / 8] >> (7 - (i % 8))) & 1;
            let sibling = self.hasher.zero_hash();
            let (left, right) = if bit == 0 {
                (current, sibling)
            } else {
                (sibling, current)
            };
            current = self.hasher.digest_node(&left, &right);
            let mut combined = Vec::with_capacity(left.len() + right.len());
            combined.extend_from_slice(&left);
            combined.extend_from_slice(&right);
            self.store.set(current, combined)?;
            debug!("Updated node at depth {}, current hash: {:?}", i, current);
        }

        self.root = current;
        info!("Updated tree with key {:?}, new root: {:?}", key, self.root);
        Ok(())
    }

    pub fn get(&self, key: Hash) -> Result<Option<Hash>, S::Error> {
        if self.root == [0u8; 32] {
            return Ok(None);
        }
        self.store
            .get(&key)
            .map(|opt| opt.and_then(|v| v.as_slice().try_into().ok()))
    }

    pub fn get_proof(&self, key: Hash) -> Result<MerkleProof, S::Error> {
        let mut current = self.root;
        let mut side_nodes = Vec::new();

        debug!("Generating proof for key {:?}", key);
        debug!("Starting from root {:?}", current);

        for i in 0..256 {
            if current == self.hasher.zero_hash() {
                debug!("Reached zero hash at depth {}", i);
                break;
            }

            // First, match on the Result to handle any potential errors
            let node_value = match self.store.get(&current)? {
                Some(v) => v,
                None => &vec![0u8; 64], // Handle the None case with a default value
            };

            let (left, right) = node_value.split_at(32);
            let bit = (key[i / 8] >> (7 - (i % 8))) & 1;

            debug!(
                "At depth {}, bit {}, left: {:?}, right: {:?}",
                i, bit, left, right
            );

            if bit == 0 {
                side_nodes.push(right.try_into().unwrap());
                current = left.try_into().unwrap();
            } else {
                side_nodes.push(left.try_into().unwrap());
                current = right.try_into().unwrap();
            }
        }

        debug!("Generated proof with {} side nodes", side_nodes.len());
        Ok(MerkleProof { side_nodes })
    }

    pub fn verify_proof(&self, key: Hash, value: Hash, proof: &MerkleProof) -> bool {
        let leaf_hash = self.hasher.digest_leaf(&key, &value);
        let mut current = leaf_hash;

        debug!("Verifying proof for key {:?}, value {:?}", key, value);
        debug!("Starting from leaf hash {:?}", current);

        for (i, sibling) in proof.side_nodes.iter().enumerate().rev() {
            let bit = (key[i / 8] >> (7 - (i % 8))) & 1;
            let (left, right) = if bit == 0 {
                (current, *sibling)
            } else {
                (*sibling, current)
            };
            current = self.hasher.digest_node(&left, &right);

            debug!(
                "At depth {}, bit {}, left: {:?}, right: {:?}, current: {:?}",
                255 - i,
                bit,
                left,
                right,
                current
            );
        }

        debug!("Final hash: {:?}", current);
        debug!("Root hash:  {:?}", self.root);

        current == self.root
    }

    pub fn root(&self) -> Hash {
        self.root
    }
}
use sha2::Sha256;

pub type Hash = [u8; 32];
pub type DefaultHasher = Sha256;

// tree_hasher.rs
use digest::{Digest, Output};
// use crate::Hash;
// 183 | use std::hash::Hash;

pub struct TreeHasher<D: Digest> {
    _marker: std::marker::PhantomData<D>,
}

impl<D: Digest> TreeHasher<D> {
    pub fn new() -> Self {
        Self {
            _marker: std::marker::PhantomData,
        }
    }

    pub fn digest_leaf(&self, key: &Hash, value: &Hash) -> Hash {
        let mut hasher = D::new();
        hasher.update([0u8]); // Leaf prefix
        hasher.update(key);
        hasher.update(value);
        self.finalize_to_array(hasher)
    }

    pub fn digest_node(&self, left: &Hash, right: &Hash) -> Hash {
        let mut hasher = D::new();
        hasher.update([1u8]); // Node prefix
        hasher.update(left);
        hasher.update(right);
        self.finalize_to_array(hasher)
    }

    pub fn zero_hash(&self) -> Hash {
        [0u8; 32]
    }

    fn finalize_to_array(&self, hasher: D) -> Hash {
        let result = hasher.finalize();
        let mut hash = [0u8; 32];
        hash.copy_from_slice(&result);
        hash
    }
}

// proof.rs
use serde::{Deserialize, Serialize};
// use crate::Hash;

#[derive(Clone, Serialize, Deserialize)]
pub struct MerkleProof {
    pub side_nodes: Vec<Hash>,
}

// main.rs (test cases)
#[cfg(test)]
mod tests {
    use super::*;
    // use proptest::prelude::*;

    #[test]
    fn test_insert_get_roundtrip() {
        let store = InMemoryKVStore::new();
        let mut smt = SparseMerkleTree::new(store);

        let key: Hash = [1u8; 32];
        let value: Hash = [2u8; 32];

        smt.update(key, value).unwrap();
        assert_eq!(smt.get(key).unwrap(), Some(value));
    }

    #[test]
    fn test_insert_and_get() {
        let store = InMemoryKVStore::new();
        let mut smt = SparseMerkleTree::new(store);
        let key: Hash = [1u8; 32];
        let value: Hash = [2u8; 32];

        smt.update(key, value).unwrap();
        assert_eq!(smt.get(key).unwrap(), Some(value));
    }

    #[test]
    fn test_proof_verification() {
        let store = InMemoryKVStore::new();
        let mut smt = SparseMerkleTree::new(store);
        let key: Hash = [1u8; 32];
        let value: Hash = [2u8; 32];

        smt.update(key, value).unwrap();
        let proof = smt.get_proof(key).unwrap();
        assert!(smt.verify_proof(key, value, &proof));

        assert!(!smt.verify_proof(key, [3u8; 32], &proof)); // Incorrect value should fail
    }

    #[test]
    fn test_empty_tree() {
        let store = InMemoryKVStore::new();
        let smt = SparseMerkleTree::new(store);
        let key: Hash = [1u8; 32];

        assert_eq!(smt.get(key).unwrap(), None); // No value should be found
    }

    #[test]
    fn test_new_tree_is_empty() {
        let store = InMemoryKVStore::new();
        let smt = SparseMerkleTree::new(store);

        assert_eq!(smt.root(), [0u8; 32]); // Root of empty tree
    }

    #[test]
    fn test_update_existing_key() {
        let store = InMemoryKVStore::new();
        let mut smt = SparseMerkleTree::new(store);
        let key: Hash = [1u8; 32];
        let initial_value: Hash = [2u8; 32];
        let updated_value: Hash = [3u8; 32];

        smt.update(key, initial_value).unwrap();
        assert_eq!(smt.get(key).unwrap(), Some(initial_value));

        smt.update(key, updated_value).unwrap();
        assert_eq!(smt.get(key).unwrap(), Some(updated_value));
    }

    #[test]
    fn test_get_non_existent_key() {
        let store = InMemoryKVStore::new();
        let smt = SparseMerkleTree::new(store);
        let non_existent_key: Hash = [99u8; 32];

        assert_eq!(smt.get(non_existent_key).unwrap(), None); // Key should not exist
    }

    #[test]
    fn test_root_changes_after_insert() {
        let store = InMemoryKVStore::new();
        let mut smt = SparseMerkleTree::new(store);
        let initial_root = smt.root();
        let key: Hash = [7u8; 32];
        let value: Hash = [70u8; 32];

        smt.update(key, value).unwrap();
        assert_ne!(smt.root(), initial_root); // Root hash should change
    }

    use super::*;
    use std::time::Instant;

    #[test]
    fn test_large_tree_inserts() {
        let store = InMemoryKVStore::new();
        let mut smt = SparseMerkleTree::new(store);
        let start = Instant::now();

        for i in 0..1000 {
            let key: Hash = [i as u8; 32];
            let value: Hash = [(i * 2) as u8; 32];
            smt.update(key, value).unwrap();
        }

        println!("Time to insert 1000 items: {:?}", start.elapsed());

        for i in 0..1000 {
            let key: Hash = [i as u8; 32];
            let expected_value: Hash = [(i * 2) as u8; 32];
            assert_eq!(smt.get(key).unwrap(), Some(expected_value));
        }
    }

    #[test]
    fn test_memory_usage() {
        let store = InMemoryKVStore::new();
        let mut smt = SparseMerkleTree::new(store);
        // let initial_memory = get_current_memory_usage();

        for i in 0..1000 {
            let key: Hash = [i as u8; 32];
            let value: Hash = [(i * 2) as u8; 32];
            smt.update(key, value).unwrap();
        }

        // let final_memory = get_current_memory_usage();
        // assert!(final_memory - initial_memory < 50_000_000); // Ensure less than 50MB increase
    }
    // Additional test cases go here...
}

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn test_insert_get_roundtrip(key: [u8; 32], value: [u8; 32]) {
            let store = InMemoryKVStore::new();
            let mut smt = SparseMerkleTree::new(store);

            smt.update(key, value).unwrap();
            prop_assert_eq!(smt.get(key).unwrap(), Some(value));
        }

        #[test]
        fn test_proof_verification(key: [u8; 32], value: [u8; 32]) {
            let store = InMemoryKVStore::new();
            let mut smt = SparseMerkleTree::new(store);

            smt.update(key, value).unwrap();
            let proof = smt.get_proof(key).unwrap();

            prop_assert!(smt.verify_proof(key, value, &proof));
        }

        #[test]
        fn test_multiple_inserts(inserts: Vec<(Hash, Hash)>) {
            let store = InMemoryKVStore::new();
            let mut smt = SparseMerkleTree::new(store);

            for (key, value) in &inserts {
                smt.update(*key, *value).unwrap();
            }

            for (key, value) in &inserts {
                prop_assert_eq!(smt.get(*key).unwrap(), Some(*value));
                let proof = smt.get_proof(*key).unwrap();
                prop_assert!(smt.verify_proof(*key, *value, &proof));
            }
        }
    }
}
