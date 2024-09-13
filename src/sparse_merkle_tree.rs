use crate::{kv_store::KVStore, proof::MerkleProof, tree_hasher::TreeHasher, DefaultHasher, Hash};
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
            self.store.set(current, [left, right].concat())?;
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
            .map(|opt| opt.and_then(|v| v.try_into().ok()))
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

            let node_value = self.store.get(&current)?.unwrap_or_else(|| vec![0u8; 64]);
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
