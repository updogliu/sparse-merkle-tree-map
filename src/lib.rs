#![cfg_attr(not(feature = "std"), no_std)]

#![feature(alloc)]

extern crate alloc;

use alloc::collections::btree_map::BTreeMap;
use alloc::vec::Vec;
use uint::U256;

#[cfg(test)]
mod tests;
mod u256_utils;

pub type Hash256 = [u8; 32];

lazy_static::lazy_static! {
    static ref U256_ZERO: U256 = U256::zero();
    static ref DEFAULT_HASHES: [Hash256; 257] = {
        // The element at index `i` is the hash of a subtree with `2^i` default nodes.
        let mut hashes: [Hash256; 257] = [[0; 32]; 257];
        for i in 1..=256 {
            hashes[i] = merge_hashes(&hashes[i-1], &hashes[i-1]);
        }
        hashes
    };
}

/// Index of a node in a Sparse Merkle Tree.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
struct TreeNodeIndex {
    // The path is defined as from the most significant bit to the `depth`-th significant bit. An 0
    // bit means left, and 1 bit means right. More less significant bits are irrelevant and set to
    // zeros.
    bit_path: U256,

    // The root has depth of 0, and the leaves have depth of 256.
    depth: usize,
}

impl TreeNodeIndex {
    /// Get a new TreeNodeIndex to the leaf corresponding to `key`.
    fn leaf(key: U256) -> Self {
        Self {
            bit_path: key,
            depth: 256,
        }
    }

    /// Index of the root.
    fn root() -> Self {
        Self {
            bit_path: U256::zero(),
            depth: 0,
        }
    }

    /// Whether this is the root.
    fn is_root(&self) -> bool {
        self.depth == 0
    }

    /// Whether this is a left subnode.
    fn is_left(&self) -> bool {
        self.depth > 0 && !self.bit_path.bit(256 - self.depth)
    }

    /// Returns the index of the sibling of this node. Returns `None` if `self` is the root.
    fn sibling(&self) -> Option<TreeNodeIndex> {
        if self.is_root() {
            return None;
        }

        let mut result = self.clone();
        u256_utils::flip_bit(&mut result.bit_path, 256 - result.depth);
        Some(result)
    }

    /// Change `self` to the index of its parent node. Panics if `self` is the root.
    fn move_up(&mut self) {
        assert!(self.depth > 0, "Cannot move up from the root of the tree!");
        u256_utils::clear_bit(&mut self.bit_path, 256 - self.depth);
        self.depth -= 1;
    }
}

/// Merkle proof of a certain triple (SMT-merkle-root, key, value).
#[derive(PartialEq, Eq, Debug)]
pub struct MerkleProof {
    pub bitmap: U256,
    pub hashes: Vec<Hash256>,
}

/// SmtMap256 is Sparse Merkle Tree Map from uint256 keys to uint256 values, and supports
/// generating 256-bit merkle proofs. Initially every of the 2**256 possible keys has a default
/// value of zero.
///
/// Each leaf corresponds to a key-value pair. The key is the bit-path from the root to the leaf
/// (starting from the most-significant-bit to the least-significant-bit; 0 is left, 1 is right).
///
/// The hash of the leaf node is just the value (in big-endian) of the corresponding key. The hash
/// of an non-leaf node is calculated by hashing (using keccak-256) the concatenation of the hashes
/// of its two sub-nodes.
#[derive(Default)]
pub struct SmtMap256 {
    kvs: BTreeMap<U256, U256>,

    // Hash values of both leaf and inner nodes.
    hashes: BTreeMap<TreeNodeIndex, Hash256>,
}

impl SmtMap256 {
    /// Returns a new SMT-Map of uint256 where all keys have the default value (zero).
    pub fn new() -> Self {
        Self {
            kvs: BTreeMap::new(),
            hashes: BTreeMap::new(),
        }
    }

    /// Sets the value of a key. Returns the old value of the key.
    pub fn set(&mut self, key: U256, value: U256) -> U256 {
        // Update the hash of the leaf.
        let mut index = TreeNodeIndex::leaf(key);
        let mut hash: Hash256 = u256_to_hash(&value);
        self.update_hash(&index, &hash);

        // Update the hashes of the inner nodes along the path.
        while !index.is_root() {
            let sibling_hash = self.get_hash(&index.sibling().unwrap());

            hash = if index.is_left() {
                merge_hashes(&hash, &sibling_hash)
            } else {
                merge_hashes(&sibling_hash, &hash)
            };
            index.move_up();
            self.update_hash(&index, &hash);
        }

        self.kvs.insert(key, value).unwrap_or(*U256_ZERO)
    }

    /// Returns a reference to the value of a key.
    pub fn get(&self, key: &U256) -> &U256 {
        self.kvs.get(key).unwrap_or(&U256_ZERO)
    }

    /// Returns a reference to the value of the key with merkle proof.
    pub fn get_with_proof(&self, key: &U256) -> (&U256, MerkleProof) {
        let mut bitmap = U256::zero();
        let mut sibling_hashes = Vec::new();
        let mut index = TreeNodeIndex::leaf(*key);
        for i in 0..256 {
            if let Some(sibling_hash) = self.hashes.get(&index.sibling().unwrap()) {
                u256_utils::set_bit(&mut bitmap, i);
                sibling_hashes.push(*sibling_hash);
            }
            index.move_up();
        }
        (
            self.get(key),
            MerkleProof {
                bitmap,
                hashes: sibling_hashes,
            },
        )
    }

    /// Returns the merkle root of this Sparse Merkle Tree.
    pub fn merkle_root(&self) -> &Hash256 {
        self.get_hash(&TreeNodeIndex::root())
    }

    /// Verifies the value of a key using the merkle proof. Returns whether the verification passed.
    pub fn verify_merkle_proof(&self, key: &U256, value: &U256, proof: &MerkleProof) -> bool {
        verify_merkle_proof(self.merkle_root(), key, value, proof)
    }

    fn get_hash(&self, index: &TreeNodeIndex) -> &Hash256 {
        self.hashes
            .get(index)
            .unwrap_or(&(*DEFAULT_HASHES)[256 - index.depth])
    }

    fn update_hash(&mut self, index: &TreeNodeIndex, hash: &Hash256) {
        if (*DEFAULT_HASHES)[256 - index.depth] == *hash {
            self.hashes.remove(index);
        } else {
            self.hashes.insert(index.clone(), *hash);
        }
    }
}

/// Verifies the value of a key in a SMT-Map (specified by its merkle root). Returns whether the
/// verification has passed.
pub fn verify_merkle_proof(
    merkle_root: &Hash256,
    key: &U256,
    value: &U256,
    proof: &MerkleProof,
) -> bool {
    let mut hash = u256_to_hash(value);
    let mut iter = proof.hashes.iter();
    for i in 0..256 {
        let sibling_hash = if !proof.bitmap.bit(i) {
            &(*DEFAULT_HASHES)[i]
        } else {
            if let Some(h) = iter.next() {
                h
            } else {
                return false;
            }
        };

        hash = if key.bit(i) {
            // sibling is at left
            merge_hashes(sibling_hash, &hash)
        } else {
            // sibling is at right
            merge_hashes(&hash, sibling_hash)
        };
    }

    iter.next() == None && hash == *merkle_root
}

fn u256_to_hash(value: &U256) -> Hash256 {
    let mut hash = [0; 32];
    value.to_big_endian(&mut hash);
    hash
}

fn merge_hashes(left: &Hash256, right: &Hash256) -> Hash256 {
    use tiny_keccak::Keccak;
    let mut hasher = Keccak::new_keccak256();
    hasher.update(&*left);
    hasher.update(&*right);
    let mut result: Hash256 = [0; 32];
    hasher.finalize(&mut result);
    result
}
