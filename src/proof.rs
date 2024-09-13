use serde::{Serialize, Deserialize};
use crate::Hash;

#[derive(Clone, Serialize, Deserialize)]
pub struct MerkleProof {
    pub side_nodes: Vec<Hash>,
}