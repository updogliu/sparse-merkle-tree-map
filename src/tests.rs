use super::*;

use std::str::FromStr;

#[test]
fn test_tree_node_index() {
    let mut index = TreeNodeIndex::leaf(hex_u256("1234567890abcdef1234567890abcdef"));
    assert!(!index.is_left());
    for _ in 0..4 {
        index.move_up();
    }
    assert!(index.is_left());
    assert_eq!(
        index,
        TreeNodeIndex {
            bit_path: hex_u256("1234567890abcdef1234567890abcde0"),
            depth: 256 - 4,
        }
    );
    assert_eq!(
        index.sibling().unwrap(),
        TreeNodeIndex {
            bit_path: hex_u256("1234567890abcdef1234567890abcdf0"),
            depth: 256 - 4,
        }
    );

    let mut index = TreeNodeIndex::leaf(U256::zero());
    for depth in (1..=256).rev() {
        assert_eq!(index.depth, depth);
        assert!(index.is_left());
        let mut sibling = index.sibling().unwrap();
        assert_eq!(sibling.depth, depth);
        assert!(!sibling.is_left());

        index.move_up();
        sibling.move_up();
        assert_eq!(index, sibling);
    }
    assert!(index.is_root());
    assert_eq!(index.bit_path, U256::zero());
    assert_eq!(index.sibling(), None);

    let mut index = TreeNodeIndex::leaf(U256::max_value());
    for depth in (1..=256).rev() {
        assert_eq!(index.depth, depth);
        assert!(!index.is_left());
        let mut sibling = index.sibling().unwrap();
        assert_eq!(sibling.depth, depth);
        assert!(sibling.is_left());

        index.move_up();
        sibling.move_up();
        assert_eq!(index, sibling);
    }
    assert!(index.is_root());
    assert_eq!(index.bit_path, U256::zero());
    assert_eq!(index.sibling(), None);
}

#[test]
fn test_smt_map_256_kv() {
    let mut smt = SmtMap256::new();
    assert_eq!(*smt.get(&U256::zero()), U256::zero());

    let key = hex_u256("1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef");
    assert_eq!(*smt.get(&key), U256::zero());

    let value = hex_u256("ffeebbaa99887766554433221100");

    assert_eq!(smt.set(key, value), U256::zero());
    assert_eq!(*smt.get(&key), value);
    assert_eq!(smt.set(key, value + 99), value);
    assert_eq!(*smt.get(&key), value + 99);
}

#[test]
fn test_smt_map_256_merkle_proof() {
    assert_eq!((*DEFAULT_HASHES)[0], [0; 32]);

    let expected_default_root_hash =
        hex_hash("a7ff9e28ffd3def443d324547688c2c4eb98edf7da757d6bfa22bff55b9ce24a");
    assert_eq!((*DEFAULT_HASHES)[256], expected_default_root_hash);

    let mut smt = SmtMap256::new();

    // Verify proof of key 0x03 when the values of all keys are default.
    let key3 = U256::from(0x03);
    let (value, proof) = smt.get_with_proof(&key3);
    assert_eq!(*value, U256::zero());
    assert_eq!(
        proof,
        MerkleProof {
            bitmap: U256::zero(),
            hashes: vec![],
        }
    );
    assert!(smt.check_merkle_proof(&key3, value, &proof));
    assert!(check_merkle_proof(smt.merkle_root(), &key3, value, &proof));

    // Verify the merkle proof of key 0x03 when key 0x00 has a non-default value.
    smt.set(U256::zero(), U256::from(0xAA));
    let (value, proof) = smt.get_with_proof(&key3);
    assert_eq!(*value, U256::zero());
    assert_eq!(
        proof,
        MerkleProof {
            bitmap: hex_u256("02"),
            hashes: vec![hex_hash(
                "d6f751104ddfead9549c96fabdbd4d2fc6876c8cd9a49ea4a821de938f71a011"
            )],
        },
    );
    assert_eq!(
        *smt.merkle_root(),
        hex_hash("c2850844249b78ca4b416d5d8430c48a89b76e808648d4630275feadab00d0cd")
    );
    assert!(smt.check_merkle_proof(&key3, value, &proof));
    assert!(check_merkle_proof(smt.merkle_root(), &key3, value, &proof));

    // Verify the merkle proof of key 0x03 again after setting a value at the max key (0xFF..FF).
    smt.set(U256::max_value(), U256::from(0x1234));
    let (value, proof) = smt.get_with_proof(&key3);
    assert_eq!(*value, U256::zero());
    assert_eq!(
        proof,
        MerkleProof {
            bitmap: hex_u256("8000000000000000000000000000000000000000000000000000000000000002"),
            hashes: vec![
                hex_hash("d6f751104ddfead9549c96fabdbd4d2fc6876c8cd9a49ea4a821de938f71a011"),
                hex_hash("5a7ef746ad33334b4fbd7406a1a4ffa5c5f959199448d5ae6ed39b4a9d6ebe5a"),
            ],
        },
    );
    assert_eq!(
        *smt.merkle_root(),
        hex_hash("514f973cd76a4e5430119524ae291a3227f1e81f69f5bf2c61a36d2a6c3e239e")
    );
    assert!(smt.check_merkle_proof(&key3, value, &proof));
    assert!(check_merkle_proof(smt.merkle_root(), &key3, value, &proof));

    // Verify the merkle proof of key 0x03 again after setting a value at key 0x03 itself.
    let value3 = U256::from(0x01) << 128;
    smt.set(key3, value3);
    let (value, proof) = smt.get_with_proof(&key3);
    assert_eq!(*value, value3);
    assert_eq!(
        proof,
        MerkleProof {
            bitmap: hex_u256("8000000000000000000000000000000000000000000000000000000000000002"),
            hashes: vec![
                hex_hash("d6f751104ddfead9549c96fabdbd4d2fc6876c8cd9a49ea4a821de938f71a011"),
                hex_hash("5a7ef746ad33334b4fbd7406a1a4ffa5c5f959199448d5ae6ed39b4a9d6ebe5a"),
            ],
        },
    );
    assert_eq!(
        *smt.merkle_root(),
        hex_hash("1f744be63eb3f347f491d7561926d80a1bee8f025f15725bd7171a32bbeefbb9")
    );

    // Reset the value of key 0x00 to the default, and verify the merkle proof of key 0x03.
    smt.set(U256::zero(), U256::zero());
    let (value, proof) = smt.get_with_proof(&key3);
    assert_eq!(*value, value3);
    assert_eq!(
        proof,
        MerkleProof {
            bitmap: hex_u256("8000000000000000000000000000000000000000000000000000000000000000"),
            hashes: vec![hex_hash(
                "5a7ef746ad33334b4fbd7406a1a4ffa5c5f959199448d5ae6ed39b4a9d6ebe5a"
            ),],
        },
    );
    assert_eq!(
        *smt.merkle_root(),
        hex_hash("8de84b42df91b9bb7a8be19646a92d31891368ec215e1f75a71c5d5022996c1d")
    );

    // Reset the value of the max key to the default, and verify the merkle proof of key 0x03.
    smt.set(U256::max_value(), U256::zero());
    let (value, proof) = smt.get_with_proof(&key3);
    assert_eq!(*value, value3);
    assert_eq!(
        proof,
        MerkleProof {
            bitmap: U256::zero(),
            hashes: vec![]
        },
    );
    assert_eq!(
        *smt.merkle_root(),
        hex_hash("48da89f9e50a0add3b33d90f59ecd6c828d4f324926677e6cdca7a198e2573f1")
    );

    // Reset the value of `key3`, and verify that the merkle tree has turned back to the initial
    // state.
    smt.set(key3, U256::zero());
    let (value, proof) = smt.get_with_proof(&key3);
    assert_eq!(*value, U256::zero());
    assert_eq!(
        proof,
        MerkleProof {
            bitmap: U256::zero(),
            hashes: vec![]
        },
    );
    assert_eq!(smt.merkle_root(), &expected_default_root_hash);
}

#[test]
fn test_smt_map_256_merkle_proof_negative_cases() {
    let mut smt = SmtMap256::new();
    let (key, value) = (U256::from(0x03), U256::from(0x01) << 128);
    smt.set(key, value);
    smt.set(U256::zero(), U256::from(0xAA));
    smt.set(U256::max_value(), U256::from(0x1234));

    // The correct merkle proof:
    assert!(smt.check_merkle_proof(
        &key,
        &value,
        &MerkleProof {
            bitmap: hex_u256("8000000000000000000000000000000000000000000000000000000000000002"),
            hashes: vec![
                hex_hash("d6f751104ddfead9549c96fabdbd4d2fc6876c8cd9a49ea4a821de938f71a011"),
                hex_hash("5a7ef746ad33334b4fbd7406a1a4ffa5c5f959199448d5ae6ed39b4a9d6ebe5a"),
            ],
        },
    ));

    // Negative cases of merkle proof verification:
    assert!(!smt.check_merkle_proof(
        &key,
        &value,
        &MerkleProof {
            bitmap: hex_u256("8000000000000000000000000000000000000000000000000000000000000002"),
            hashes: vec![
                hex_hash("d6f751104ddfead9549c96fabdbd4d2fc6876c8cd9a49ea4a821de938f71a011"),
                hex_hash("5a7ef746ad33334b4fbd7406a1a4ffa5c5f959199448d5ae6ed39b4a9d6ebe5a"),
                hex_hash("00"), // extra hash
            ],
        }
    ));
    assert!(!smt.check_merkle_proof(
        &key,
        &value,
        &MerkleProof {
            bitmap: hex_u256("8000000000000000000000000000000000000000000000000000000000000002"),
            hashes: vec![
                hex_hash("d6f751104ddfead9549c96fabdbd4d2fc6876c8cd9a49ea4a821de938f71a011"),
                // missing hash
            ],
        }
    ));
    assert!(!smt.check_merkle_proof(
        &key,
        &value,
        &MerkleProof {
            // wrong bitmap
            bitmap: hex_u256("0000000000000000000000000000000000000000000000000000000000000002"),
            hashes: vec![
                hex_hash("d6f751104ddfead9549c96fabdbd4d2fc6876c8cd9a49ea4a821de938f71a011"),
                hex_hash("5a7ef746ad33334b4fbd7406a1a4ffa5c5f959199448d5ae6ed39b4a9d6ebe5a"),
            ],
        }
    ));
    assert!(!smt.check_merkle_proof(
        &key,
        &value,
        &MerkleProof {
            // wrong bitmap
            bitmap: hex_u256("8000000000000000000000000000000000000000000000000000000000000004"),
            hashes: vec![
                hex_hash("d6f751104ddfead9549c96fabdbd4d2fc6876c8cd9a49ea4a821de938f71a011"),
                hex_hash("5a7ef746ad33334b4fbd7406a1a4ffa5c5f959199448d5ae6ed39b4a9d6ebe5a"),
            ],
        }
    ));
}

fn hex_u256(s: &str) -> U256 {
    U256::from_str(s).unwrap()
}

fn hex_hash(s: &str) -> Hash256 {
    u256_to_hash(&hex_u256(s))
}
