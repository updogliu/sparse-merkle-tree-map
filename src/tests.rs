use super::*;

use hex::FromHex;
use alloc::string::ToString;

#[test]
fn test_tree_node_index() {
    let mut index = TreeNodeIndex::leaf(r256("1234567890abcdef1234567890abcdef"));
    assert!(!index.is_left());
    for _ in 0..3 {
        index.move_up();
    }
    assert!(index.is_left());
    assert_eq!(
        index,
        TreeNodeIndex {
            bit_path: r256("1234567890abcdef1234567890abcd0f"),
            depth: 256 - 3,
        }
    );
    assert_eq!(
        index.sibling().unwrap(),
        TreeNodeIndex {
            bit_path: r256("1234567890abcdef1234567890abcd1f"),
            depth: 256 - 3,
        }
    );

    // Climb up from the left-most leaf.
    let mut index = TreeNodeIndex::leaf([0; 32]);
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
    assert_eq!(index.bit_path, [0; 32]);
    assert_eq!(index.sibling(), None);

    // Climb up from the right-most leaf.
    let mut index = TreeNodeIndex::leaf(max256());
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
    assert_eq!(index.bit_path, [0; 32]);
    assert_eq!(index.sibling(), None);
}

#[test]
fn test_smt_map_256_kv() {
    let mut smt = SmtMap256::new();
    assert_eq!(*smt.get(&[0; 32]), [0; 32]);

    let key = b256("1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef");
    assert_eq!(*smt.get(&key), [0; 32]);

    let value1 = r256("ffeebbaa99887766554433221100");
    let value2 = r256("ffeebbaa99887766554433221199");

    assert_eq!(smt.set(&key, value1), [0; 32]);
    assert_eq!(*smt.get(&key), value1);
    assert_eq!(smt.set(&key, value2), value1);
    assert_eq!(*smt.get(&key), value2);
}

#[test]
fn test_smt_map_256_merkle_proof() {
    assert_eq!((*DEFAULT_HASHES)[0], [0; 32]);

    let expected_default_root_hash =
        b256("a7ff9e28ffd3def443d324547688c2c4eb98edf7da757d6bfa22bff55b9ce24a");
    assert_eq!((*DEFAULT_HASHES)[256], expected_default_root_hash);

    let mut smt = SmtMap256::new();

    // Verify proof of `key` when the values of all keys are default.
    let key = r256("C0");
    let (value, proof) = smt.get_with_proof(&key);
    assert_eq!(*value, [0; 32]);
    assert_eq!(
        proof,
        MerkleProof {
            bitmap: [0; 32],
            hashes: Vec::new(),
        }
    );
    assert!(smt.check_merkle_proof(&key, value, &proof));
    assert!(check_merkle_proof(smt.merkle_root(), &key, value, &proof));

    // Verify the merkle proof of `key` when key 0x00 has a non-default value.
    smt.set(&[0; 32], r256("AA"));
    let (value, proof) = smt.get_with_proof(&key);
    assert_eq!(*value, [0; 32]);
    assert_eq!(
        proof,
        MerkleProof {
            bitmap: l256("02"),
            hashes: vec![b256(
                "d6f751104ddfead9549c96fabdbd4d2fc6876c8cd9a49ea4a821de938f71a011"
            )],
        },
    );
    assert_eq!(
        *smt.merkle_root(),
        b256("c2850844249b78ca4b416d5d8430c48a89b76e808648d4630275feadab00d0cd")
    );
    assert!(smt.check_merkle_proof(&key, value, &proof));
    assert!(check_merkle_proof(smt.merkle_root(), &key, value, &proof));

    // Verify the merkle proof of `key` again after setting a value at the max key (0xFF..FF).
    smt.set(&max256(), r256("1234"));
    let (value, proof) = smt.get_with_proof(&key);
    assert_eq!(*value, [0; 32]);
    assert_eq!(
        proof,
        MerkleProof {
            bitmap: b256("0200000000000000000000000000000000000000000000000000000000000080"),
            hashes: vec![
                b256("d6f751104ddfead9549c96fabdbd4d2fc6876c8cd9a49ea4a821de938f71a011"),
                b256("5a7ef746ad33334b4fbd7406a1a4ffa5c5f959199448d5ae6ed39b4a9d6ebe5a"),
            ],
        },
    );
    assert_eq!(
        *smt.merkle_root(),
        b256("514f973cd76a4e5430119524ae291a3227f1e81f69f5bf2c61a36d2a6c3e239e")
    );
    assert!(smt.check_merkle_proof(&key, value, &proof));
    assert!(check_merkle_proof(smt.merkle_root(), &key, value, &proof));

    // Verify the merkle proof of `key` again after setting a value at `key` itself.
    let value2 = r256("0100000000000000000000000000000000");
    smt.set(&key, value2);
    let (value, proof) = smt.get_with_proof(&key);
    assert_eq!(*value, value2);
    assert_eq!(
        proof,
        MerkleProof {
            bitmap: b256("0200000000000000000000000000000000000000000000000000000000000080"),
            hashes: vec![
                b256("d6f751104ddfead9549c96fabdbd4d2fc6876c8cd9a49ea4a821de938f71a011"),
                b256("5a7ef746ad33334b4fbd7406a1a4ffa5c5f959199448d5ae6ed39b4a9d6ebe5a"),
            ],
        },
    );
    assert_eq!(
        *smt.merkle_root(),
        b256("1f744be63eb3f347f491d7561926d80a1bee8f025f15725bd7171a32bbeefbb9")
    );

    // Reset the value of key 0x00..00 to the default, and verify the merkle proof of `key`.
    smt.set(&[0; 32], [0; 32]);
    let (value, proof) = smt.get_with_proof(&key);
    assert_eq!(*value, value2);
    assert_eq!(
        proof,
        MerkleProof {
            bitmap: b256("0000000000000000000000000000000000000000000000000000000000000080"),
            hashes: vec![b256(
                "5a7ef746ad33334b4fbd7406a1a4ffa5c5f959199448d5ae6ed39b4a9d6ebe5a"
            ),],
        },
    );
    assert_eq!(
        *smt.merkle_root(),
        b256("8de84b42df91b9bb7a8be19646a92d31891368ec215e1f75a71c5d5022996c1d")
    );

    // Reset the value of the max key to the default, and verify the merkle proof of `key`.
    smt.set(&max256(), [0; 32]);
    let (value, proof) = smt.get_with_proof(&key);
    assert_eq!(*value, value2);
    assert_eq!(
        proof,
        MerkleProof {
            bitmap: [0; 32],
            hashes: vec![]
        },
    );
    assert_eq!(
        *smt.merkle_root(),
        b256("48da89f9e50a0add3b33d90f59ecd6c828d4f324926677e6cdca7a198e2573f1")
    );

    // Reset the value of `key`, and verify that the merkle tree has been reset to the init state.
    smt.set(&key, [0; 32]);
    let (value, proof) = smt.get_with_proof(&key);
    assert_eq!(*value, [0; 32]);
    assert_eq!(
        proof,
        MerkleProof {
            bitmap: [0; 32],
            hashes: vec![]
        },
    );
    assert_eq!(smt.merkle_root(), &expected_default_root_hash);
}

#[test]
fn test_smt_map_256_merkle_proof_negative_cases() {
    let mut smt = SmtMap256::new();
    let (key, value) = (r256("C0"), r256("0100000000000000000000000000000000"));
    smt.set(&key, value);
    smt.set(&[0; 32], r256("AA"));
    smt.set(&max256(), r256("1234"));

    // The correct merkle proof:
    assert!(smt.check_merkle_proof(
        &key,
        &value,
        &MerkleProof {
            bitmap: b256("0200000000000000000000000000000000000000000000000000000000000080"),
            hashes: vec![
                b256("d6f751104ddfead9549c96fabdbd4d2fc6876c8cd9a49ea4a821de938f71a011"),
                b256("5a7ef746ad33334b4fbd7406a1a4ffa5c5f959199448d5ae6ed39b4a9d6ebe5a"),
            ],
        },
    ));

    // Negative cases of merkle proof verification:
    assert!(!smt.check_merkle_proof(
        &key,
        &value,
        &MerkleProof {
            bitmap: b256("0200000000000000000000000000000000000000000000000000000000000080"),
            hashes: vec![
                b256("d6f751104ddfead9549c96fabdbd4d2fc6876c8cd9a49ea4a821de938f71a011"),
                b256("5a7ef746ad33334b4fbd7406a1a4ffa5c5f959199448d5ae6ed39b4a9d6ebe5a"),
                [0; 32], // extra hash
            ],
        }
    ));
    assert!(!smt.check_merkle_proof(
        &key,
        &value,
        &MerkleProof {
            bitmap: b256("0200000000000000000000000000000000000000000000000000000000000080"),
            hashes: vec![
                b256("d6f751104ddfead9549c96fabdbd4d2fc6876c8cd9a49ea4a821de938f71a011"),
                // missing hash
            ],
        }
    ));
    assert!(!smt.check_merkle_proof(
        &key,
        &value,
        &MerkleProof {
            // wrong bitmap - missing bit
            bitmap: b256("0200000000000000000000000000000000000000000000000000000000000000"),
            hashes: vec![
                b256("d6f751104ddfead9549c96fabdbd4d2fc6876c8cd9a49ea4a821de938f71a011"),
                b256("5a7ef746ad33334b4fbd7406a1a4ffa5c5f959199448d5ae6ed39b4a9d6ebe5a"),
            ],
        }
    ));
    assert!(!smt.check_merkle_proof(
        &key,
        &value,
        &MerkleProof {
            // wrong bitmap - extra bit
            bitmap: b256("0200010000000000000000000000000000000000000000000000000000000080"),
            hashes: vec![
                b256("d6f751104ddfead9549c96fabdbd4d2fc6876c8cd9a49ea4a821de938f71a011"),
                b256("5a7ef746ad33334b4fbd7406a1a4ffa5c5f959199448d5ae6ed39b4a9d6ebe5a"),
            ],
        }
    ));
    assert!(!smt.check_merkle_proof(
        &key,
        &value,
        &MerkleProof {
            // wrong bitmap - wrong bit
            bitmap: b256("0400000000000000000000000000000000000000000000000000000000000080"),
            hashes: vec![
                b256("d6f751104ddfead9549c96fabdbd4d2fc6876c8cd9a49ea4a821de938f71a011"),
                b256("5a7ef746ad33334b4fbd7406a1a4ffa5c5f959199448d5ae6ed39b4a9d6ebe5a"),
            ],
        }
    ));
}

// `hex` is the first a few bytes of the desired 32 bytes (the rest bytes are zeros).
fn l256(hex: &str) -> [u8; 32] {
    assert!(hex.len() % 2 == 0 && hex.len() <= 64);
    let hex = hex.to_string() + &"0".repeat(64 - hex.len());
    <[u8; 32]>::from_hex(&hex).unwrap()
}


// `hex` is the last a few bytes of the desired 32 bytes (the rest bytes are zeros).
fn r256(hex: &str) -> [u8; 32] {
    assert!(hex.len() % 2 == 0 && hex.len() <= 64);
    let hex = "0".repeat(64 - hex.len()) + hex;
    <[u8; 32]>::from_hex(&hex).unwrap()
}


fn b256(s: &str) -> Hash256 {
    <[u8; 32]>::from_hex(s).unwrap()
}

fn max256() -> [u8; 32] {
    b256("ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff")
}
