use crate::tree_hasher::TreeHasher;
use bytes::Bytes;
use digest::Digest;

pub struct MerkleProof {
    pub side_nodes: Vec<Bytes>,
}

impl MerkleProof {
    pub fn verify(
        &self,
        root: &[u8],
        key: &[u8],
        value: &[u8],
        hasher: &TreeHasher<impl Digest>,
    ) -> bool {
        let path = hasher.digest(key);
        let leaf_hash = hasher.digest_leaf(&path, value);
        let mut current_hash = leaf_hash;

        for (i, sibling) in self.side_nodes.iter().enumerate().rev() {
            let bit = (path[i / 8] >> (7 - (i % 8))) & 1;
            current_hash = if bit == 0 {
                hasher.digest_node(&current_hash, sibling)
            } else {
                hasher.digest_node(sibling, &current_hash)
            };
        }

        current_hash.as_ref() == root
    }
}
