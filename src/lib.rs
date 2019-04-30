#![no_std]
#![feature(alloc)]

#[cfg(not(test))]
extern crate alloc;

#[cfg(test)]
#[macro_use]
extern crate alloc;

use alloc::collections::btree_map::BTreeMap;
use alloc::vec::Vec;

mod bit_op;

#[cfg(test)]
mod tests;

pub type Key = [u8; 32];
pub type Value = [u8; 32];
pub type Hash256 = [u8; 32];

lazy_static::lazy_static! {
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
    // The path starts from the first bit (the least significant bit of the first byte), and ends at
    // the `depth`-th bit. Bit 0 means left, and bit 1 means right. Bits beyond the `depth`-th bit
    // are irrelevant, and are always zeros.
    bit_path: [u8; 32],

    // The root has depth of 0, and the leaves have depth of 256.
    depth: usize,
}

impl TreeNodeIndex {
    /// Get a new TreeNodeIndex of the leaf corresponding to the given key.
    fn leaf(key: Key) -> Self {
        Self {
            bit_path: key,
            depth: 256,
        }
    }

    /// Index of the root.
    fn root() -> Self {
        Self {
            bit_path: [0; 32],
            depth: 0,
        }
    }

    /// Whether this is the root.
    fn is_root(&self) -> bool {
        self.depth == 0
    }

    /// Whether this is a left subnode.
    fn is_left(&self) -> bool {
        self.depth > 0 && !bit_op::get_bit(&self.bit_path, self.depth - 1)
    }

    /// Returns the index of the sibling of this node. Returns `None` if `self` is the root.
    fn sibling(&self) -> Option<TreeNodeIndex> {
        if self.is_root() {
            return None;
        }

        let mut result = self.clone();
        bit_op::flip_bit(&mut result.bit_path, result.depth - 1);
        Some(result)
    }

    /// Change `self` to the index of its parent node. Panics if `self` is the root.
    fn move_up(&mut self) {
        assert!(self.depth > 0, "Cannot move up from the root");
        bit_op::clear_bit(&mut self.bit_path, self.depth - 1);
        self.depth -= 1;
    }
}

/// Merkle proof of a certain triple (SMT-merkle-root, key, value).
#[derive(PartialEq, Eq, Debug)]
pub struct MerkleProof {
    /// Whether the siblings along the path to the root are non-default hashes.
    pub bitmap: [u8; 32],

    pub hashes: Vec<Hash256>,
}

/// SmtMap256 is Sparse Merkle Tree Map from 256-bit keys to 256-bit values, and supports
/// generating 256-bit merkle proofs. Initially every of the 2**256 possible keys has a default
/// value of zero.
///
/// Each leaf corresponds to a key-value pair. The key is the bit-path from the root to the leaf
/// (see the documentation for TreeNodeIndex).
///
/// The hash of the leaf node is just the value of the corresponding key. The hash of an non-leaf
/// node is calculated by hashing (using keccak-256) the concatenation of the hashes of its two
/// sub-nodes.
#[derive(Clone, Default)]
pub struct SmtMap256 {
    kvs: BTreeMap<Key, Value>,

    // Hash values of both leaf and inner nodes.
    hashes: BTreeMap<TreeNodeIndex, Hash256>,
}

impl SmtMap256 {
    /// Returns a new SMT-Map where all keys have the default value (zero).
    pub fn new() -> Self {
        Self {
            kvs: BTreeMap::new(),
            hashes: BTreeMap::new(),
        }
    }

    /// Sets the value of a key. Returns the old value of the key.
    pub fn set(&mut self, key: &Key, value: Value) -> Value {
        // Update the hash of the leaf.
        let mut index = TreeNodeIndex::leaf(*key);
        let mut hash: Hash256 = value;
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

        self.kvs.insert(*key, value).unwrap_or([0; 32])
    }

    /// Returns a reference to the value of a key.
    pub fn get(&self, key: &Key) -> &Value {
        self.kvs.get(key).unwrap_or(&[0; 32])
    }

    /// Returns a reference to the value of the key with merkle proof.
    pub fn get_with_proof(&self, key: &Key) -> (&Value, MerkleProof) {
        let mut bitmap = [0_u8; 32];
        let mut sibling_hashes = Vec::new();
        let mut index = TreeNodeIndex::leaf(*key);
        for i in 0..256 {
            if let Some(sibling_hash) = self.hashes.get(&index.sibling().unwrap()) {
                bit_op::set_bit(&mut bitmap, i);
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

    /// Check the merkle proof of a key-value pair in this SMT-Map. Returns whether the proof is
    /// valid.
    pub fn check_merkle_proof(&self, key: &Key, value: &Value, proof: &MerkleProof) -> bool {
        check_merkle_proof(self.merkle_root(), key, value, proof)
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

/// Check the merkle proof of a key-value pair in a SMT-Map (specified by its merkle root). Returns
/// whether the proof is valid.
pub fn check_merkle_proof(
    merkle_root: &Hash256,
    key: &Key,
    value: &Value,
    proof: &MerkleProof,
) -> bool {
    let mut hash = *value;
    let mut iter = proof.hashes.iter();
    for i in 0..256 {
        let sibling_hash = if !bit_op::get_bit(&proof.bitmap, i) {
            &(*DEFAULT_HASHES)[i]
        } else {
            if let Some(h) = iter.next() {
                h
            } else {
                return false;
            }
        };

        let depth = 256 - i;
        hash = if bit_op::get_bit(key, depth - 1) {
            // sibling is at left
            merge_hashes(sibling_hash, &hash)
        } else {
            // sibling is at right
            merge_hashes(&hash, sibling_hash)
        };
    }

    iter.next() == None && hash == *merkle_root
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
