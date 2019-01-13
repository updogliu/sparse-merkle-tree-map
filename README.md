# Spare Merkle Tree Map (SMT-Map)

A Spare Merkle Tree Map (SMT-Map) is a map from uint to uint backed by Sparse Merkle Tree (SMT),
which supports proving the value of a key with Merkle Proof. In particular it can prove that
a key is unset (has never been set to a non-zero value or has been reset to zero).


SMT has a form of full binary tree. Therefore an SMT of, for example, uint256 always has 2**256
leaf node. Each leaf node corresponds to a key-value pair of the SMT-Map: the value is stored on
the node, and the key is the uint representation of the path from the root to the leaf.

See the [documentation](https://docs.rs/smt_map/0.0.1/smt_map) and references for more details.

## References:
[1] [Revocation Transparency](
    https://www.links.org/files/RevocationTransparency.pdf)

[2] [Data availability proof-friendly state tree transitions](
    https://ethresear.ch/t/data-availability-proof-friendly-state-tree-transitions/1453)

# Library Status

Pre-alpha. Basically tested and documented. The APIs are subject to change.

Pull requests and feature requests are welcome.
