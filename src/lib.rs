pub mod kv_store;
pub mod proof;
pub mod sparse_merkle_tree;
pub mod tree_hasher;
pub mod error;

#[cfg(test)]
mod tests;

use sha2::Sha256;

pub type Hash = [u8; 32];
pub type DefaultHasher = Sha256;
