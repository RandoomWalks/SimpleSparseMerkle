use digest::{Digest, Output};
use crate::Hash;
use digest::generic_array::GenericArray;


pub struct TreeHasher<D: Digest> {
    _marker: std::marker::PhantomData<D>,
}

impl<D: Digest> TreeHasher<D> {
    pub fn new() -> Self {
        Self { _marker: std::marker::PhantomData }
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