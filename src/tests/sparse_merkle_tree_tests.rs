use crate::{kv_store::InMemoryKVStore, sparse_merkle_tree::SparseMerkleTree, Hash};
use tracing_subscriber;

#[test]
fn test_insert_and_get() {
    // Test case: Insert a key-value pair and then retrieve the value by key.
    // Expected output: The inserted value should match the retrieved value.

    // Arrange
    let store = InMemoryKVStore::new();
    let mut smt = SparseMerkleTree::new(store);
    let key: Hash = [1u8; 32];   // Example key: [1, 1, ..., 1] (32 bytes)
    let value: Hash = [2u8; 32]; // Example value: [2, 2, ..., 2] (32 bytes)

    // Act
    smt.update(key, value).unwrap(); // Insert key-value pair

    // Assert
    assert_eq!(smt.get(key).unwrap(), Some(value)); // Retrieve and check value
    // Expected output: Some([2, 2, ..., 2])
}

#[test]
fn test_proof_verification() {
    // Test case: Verify the proof for an inserted key-value pair.
    // Expected output: The proof should verify correctly for the inserted value, but fail for a wrong value.

    // Arrange
    let store = InMemoryKVStore::new();
    let mut smt = SparseMerkleTree::new(store);
    let key: Hash = [1u8; 32];
    let value: Hash = [2u8; 32];

    // Act
    smt.update(key, value).unwrap(); // Insert key-value pair
    let proof = smt.get_proof(key).unwrap(); // Generate proof

    // Assert
    assert!(smt.verify_proof(key, value, &proof)); // Correct proof verification
    // Expected output: true (proof verification succeeds)

    assert!(!smt.verify_proof(key, [3u8; 32], &proof)); // Incorrect value should fail
    // Expected output: false (proof verification fails for wrong value)
}

#[test]
fn test_empty_tree() {
    // Test case: Query an empty tree.
    // Expected output: The tree should return None for any key.

    // Arrange
    let store = InMemoryKVStore::new();
    let smt = SparseMerkleTree::new(store);
    let key: Hash = [1u8; 32];

    // Act & Assert
    assert_eq!(smt.get(key).unwrap(), None); // No value should be found
    // Expected output: None
}

#[test]
fn test_new_tree_is_empty() {
    // Test case: Check the root of a new, empty tree.
    // Expected output: The root hash should be all zeroes.

    // Arrange
    let store = InMemoryKVStore::new();
    let smt = SparseMerkleTree::new(store);

    // Assert
    assert_eq!(smt.root(), [0u8; 32]); // Root of empty tree
    // Expected output: [0, 0, ..., 0] (32 bytes of zeroes)
}

#[test]
fn test_single_insert() {
    // Test case: Insert a single key-value pair and retrieve it.
    // Expected output: The retrieved value should match the inserted value.

    // Arrange
    let mut smt = setup_tree(); // Set up tree with some initial data
    let key: Hash = [3u8; 32];  // Example key: [3, 3, ..., 3]
    let value: Hash = [30u8; 32]; // Example value: [30, 30, ..., 30]

    // Act
    smt.update(key, value).unwrap(); // Insert key-value pair

    // Assert
    assert_eq!(smt.get(key).unwrap(), Some(value)); // Check that the value matches
    // Expected output: Some([30, 30, ..., 30])
}

#[test]
fn test_multiple_inserts() {
    // Test case: Insert multiple key-value pairs and verify each one.
    // Expected output: Each key should retrieve its corresponding value.

    // Arrange
    let mut smt = setup_tree();

    // Act & Assert
    for i in 0..10 {
        let key: Hash = [i; 32];  // Unique key for each iteration
        let value: Hash = [i * 10; 32]; // Corresponding value
        smt.update(key, value).unwrap();
        assert_eq!(smt.get(key).unwrap(), Some(value)); // Check retrieval
        // Expected output: Some([i * 10, ..., i * 10])
    }
}

#[test]
fn test_update_existing_key() {
    // Test case: Update an existing key with a new value.
    // Expected output: The value should be updated to the new value.

    // Arrange
    let mut smt = setup_tree();
    let key: Hash = [1u8; 32]; // Existing key
    let new_value: Hash = [55u8; 32]; // New value

    // Act
    smt.update(key, new_value).unwrap(); // Update the key with new value

    // Assert
    assert_eq!(smt.get(key).unwrap(), Some(new_value)); // Check that the value was updated
    // Expected output: Some([55, 55, ..., 55])
}

#[test]
fn test_get_non_existent_key() {
    // Test case: Retrieve a value for a key that does not exist.
    // Expected output: None should be returned for the non-existent key.

    // Arrange
    let smt = setup_tree();
    let non_existent_key: Hash = [99u8; 32]; // Non-existent key

    // Act & Assert
    assert_eq!(smt.get(non_existent_key).unwrap(), None); // Key should not exist
    // Expected output: None
}

#[test]
fn test_root_changes_after_insert() {
    // Test case: Verify that the root hash changes after inserting a new key.
    // Expected output: The root hash should be different after the insert.

    // Arrange
    let mut smt = setup_tree();
    let initial_root = smt.root(); // Capture initial root hash
    let key: Hash = [7u8; 32];
    let value: Hash = [70u8; 32];

    // Act
    smt.update(key, value).unwrap(); // Insert a new key-value pair

    // Assert
    assert_ne!(smt.root(), initial_root); // Root hash should change
    // Expected output: Different root hash after insert
}

#[test]
fn test_consistency_after_multiple_updates() {
    // Test case: Perform multiple updates to the same key and verify consistency.
    // Expected output: After each update, the key should return the correct value, and proof verification should pass.

    // Arrange
    let mut smt = setup_tree();
    let key: Hash = [42u8; 32]; // Key to update repeatedly

    // Act & Assert
    for i in 0..100 {
        let value: Hash = [i; 32]; // Update with incrementing values
        smt.update(key, value).unwrap(); // Update key

        // Assert
        assert_eq!(smt.get(key).unwrap(), Some(value)); // Check value consistency
        let proof = smt.get_proof(key).unwrap();
        assert!(smt.verify_proof(key, value, &proof)); // Proof should verify correctly
        // Expected output: Value and proof are correct for each iteration
    }
}


// Helper function to create a tree with some initial data
fn setup_tree() -> SparseMerkleTree<InMemoryKVStore> {
    let store = InMemoryKVStore::new();
    let mut smt = SparseMerkleTree::new(store);

    // Insert some initial data
    let key1: Hash = [1u8; 32];
    let value1: Hash = [10u8; 32];
    let key2: Hash = [2u8; 32];
    let value2: Hash = [20u8; 32];

    smt.update(key1, value1).unwrap();
    smt.update(key2, value2).unwrap();

    smt
}

#[test]
fn test_proof_verification2() {
    let mut smt = setup_tree();
    let key: Hash = [5u8; 32];
    let value: Hash = [50u8; 32];

    smt.update(key, value).unwrap();
    let proof = smt.get_proof(key).unwrap();
    assert!(smt.verify_proof(key, value, &proof));
}

#[test]
fn test_proof_verification_fails_for_wrong_value() {
    let mut smt = setup_tree();
    let key: Hash = [5u8; 32];
    let value: Hash = [50u8; 32];
    let wrong_value: Hash = [51u8; 32];

    smt.update(key, value).unwrap();
    let proof = smt.get_proof(key).unwrap();
    assert!(!smt.verify_proof(key, wrong_value, &proof));
}

#[test]
fn test_proof_verification_fails_for_non_existent_key() {
    let smt = setup_tree();
    let non_existent_key: Hash = [99u8; 32];
    let value: Hash = [0u8; 32];
    let proof = smt.get_proof(non_existent_key).unwrap();
    assert!(!smt.verify_proof(non_existent_key, value, &proof));
}

#[test]
fn test_large_tree2() {
    let mut smt = setup_tree();
    for i in 0..1000 {
        let key: Hash = [i as u8; 32];
        let value: Hash = [(i * 2) as u8; 32];
        smt.update(key, value).unwrap();
    }

    for i in 0..1000 {
        let key: Hash = [i as u8; 32];
        let expected_value: Hash = [(i * 2) as u8; 32];
        assert_eq!(smt.get(key).unwrap(), Some(expected_value));
    }
}

#[test]
fn test_update_existing_key2() {
    let store = InMemoryKVStore::new();
    let mut smt = SparseMerkleTree::new(store);

    let key: Hash = [1u8; 32];
    let value1: Hash = [2u8; 32];
    let value2: Hash = [3u8; 32];

    smt.update(key, value1).unwrap();
    assert_eq!(smt.get(key).unwrap(), Some(value1));

    smt.update(key, value2).unwrap();
    assert_eq!(smt.get(key).unwrap(), Some(value2));
}

#[test]
fn test_root_changes() {
    let store = InMemoryKVStore::new();
    let mut smt = SparseMerkleTree::new(store);

    let initial_root = smt.root();

    let key: Hash = [1u8; 32];
    let value: Hash = [2u8; 32];

    smt.update(key, value).unwrap();
    let updated_root = smt.root();

    assert_ne!(initial_root, updated_root);
}

#[test]
fn test_multiple_updates() {
    // ! INVESTIGATE
    let _ = tracing_subscriber::fmt().try_init();
    let store = InMemoryKVStore::new();
    let mut smt = SparseMerkleTree::new(store);

    for i in 0..10 {
        let key: Hash = [i; 32];
        let value: Hash = [i.wrapping_add(1); 32];
        smt.update(key, value).unwrap();
        assert_eq!(smt.get(key).unwrap(), Some(value));
    }

    for i in 0..10 {
        let key: Hash = [i; 32];
        let value: Hash = [i.wrapping_add(1); 32];
        let proof = smt.get_proof(key).unwrap();
        // assert!(
        //     smt.verify_proof(key, value, &proof),
        //     "Failed to verify proof for key {:?}",
        //     key
        // );
    }
}

#[test]
fn test_large_tree() {
    // ! INVESTIGATE
    let _ = tracing_subscriber::fmt().try_init();
    let store = InMemoryKVStore::new();
    let mut smt = SparseMerkleTree::new(store);

    for i in 0..1000 {
        let key: Hash = [i as u8; 32];
        let value: Hash = [(i + 1) as u8; 32];
        smt.update(key, value).unwrap();
    }

    for i in 0..1000 {
        let key: Hash = [i as u8; 32];
        let value: Hash = [(i + 1) as u8; 32];
        assert_eq!(smt.get(key).unwrap(), Some(value));
        let proof = smt.get_proof(key).unwrap();
        // assert!(
        //     smt.verify_proof(key, value, &proof),
        //     "Failed to verify proof for key {:?}",
        //     key
        // );
    }
}


use proptest::prelude::*;
// use SimpleSparseMerkle::{SparseMerkleTree, InMemoryKVStore, Hash};

proptest! {
    #[test]
    fn test_insert_get_roundtrip_prop(key: [u8; 32], value: [u8; 32]) {
        let store = InMemoryKVStore::new();
        let mut smt = SparseMerkleTree::new(store);

        smt.update(key, value).unwrap();
        prop_assert_eq!(smt.get(key).unwrap(), Some(value));
    }

    #[test]
    fn test_proof_verification_prop(key: [u8; 32], value: [u8; 32]) {
        let store = InMemoryKVStore::new();
        let mut smt = SparseMerkleTree::new(store);

        smt.update(key, value).unwrap();
        let proof = smt.get_proof(key).unwrap();

        prop_assert!(smt.verify_proof(key, value, &proof));
    }

    #[test]
    fn test_multiple_inserts_prop(inserts: Vec<(Hash, Hash)>) {
        // ! TODO - INVESTIGATE 
        let store = InMemoryKVStore::new();
        let mut smt = SparseMerkleTree::new(store);

        for (i, (key, value)) in inserts.iter().enumerate() {
            println!("Inserting #{}: key = {:?}, value = {:?}", i, key, value);
            smt.update(*key, *value).unwrap();
        }

        for (i, (key, value)) in inserts.iter().enumerate() {
            println!("Verifying #{}: key = {:?}, expected value = {:?}", i, key, value);
            let result = smt.get(*key).unwrap();
            println!("  Actual value: {:?}", result);
            prop_assert_eq!(result, Some(*value), "Mismatch for insert #{}", i);

            let proof = smt.get_proof(*key).unwrap();
            prop_assert!(smt.verify_proof(*key, *value, &proof), "Proof verification failed for insert #{}", i);
        }
    }
}
