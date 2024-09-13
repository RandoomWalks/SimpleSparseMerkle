
use proptest::prelude::*;
use SimpleSparseMerkle::{SparseMerkleTree, InMemoryKVStore, Hash};

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
