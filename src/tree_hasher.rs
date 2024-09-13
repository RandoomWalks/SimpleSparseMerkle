use bytes::Bytes;
use digest::{generic_array::GenericArray, Digest};
use digest::OutputSizeUser;

/// Simple hasher struct for computing hashes
pub struct TreeHasher<H> {
    zero_value: Bytes,
    _marker: core::marker::PhantomData<H>,
}

impl<H: Digest + OutputSizeUser> TreeHasher<H> {
    pub fn new() -> Self {
        Self {
            zero_value: vec![0; <H as OutputSizeUser>::output_size()].into(), // Specify `OutputSizeUser`
            _marker: Default::default(),
        }
    }

    pub fn digest(&self, data: impl AsRef<[u8]>) -> Vec<u8> {
        H::digest(data).to_vec() // Convert to Vec<u8>
    }

    pub fn digest_leaf(&self, path: &[u8], value: &[u8]) -> Bytes {
        let mut data = Vec::with_capacity(1 + path.len() + value.len());
        data.push(0); // LEAF_PREFIX
        data.extend_from_slice(path);
        data.extend_from_slice(value);
        Bytes::from(H::digest(&data).to_vec()) // Convert GenericArray to Vec<u8>, then to Bytes
    }

    pub fn digest_node(&self, left: &[u8], right: &[u8]) -> Bytes {
        let mut data = Vec::with_capacity(1 + left.len() + right.len());
        data.push(1); // NODE_PREFIX
        data.extend_from_slice(left);
        data.extend_from_slice(right);
        Bytes::from(H::digest(&data).to_vec()) // Convert GenericArray to Vec<u8>, then to Bytes
    }
    pub fn zero_value(&self) -> Bytes {
        self.zero_value.clone()
    }
}
