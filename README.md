# Spare Merkle Tree

Sparse Merkle Tree (SMT) is a data structure storing a uint-to-uint key-value map. It supports
proving the value of a key with Merkle Proof.

SMT has a form of full binary tree. Therefore an SMT of uint256 always has 2**256 leaf node.

See the references for more details:
[1] [Revocation Transparency] (https://www.links.org/files/RevocationTransparency.pdf)
[2] [Data availability proof-friendly state tree transitions] (https://ethresear.ch/t/data-availability-proof-friendly-state-tree-transitions/1453)

# Library Status

Pre-alpha. Basically tested. The APIs are subject to change. Pull requests and feature requests
are welcome.
