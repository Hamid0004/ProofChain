ProofChain 

A Rust-based Proof-of-Existence Notary System with Merkle Tree batching and blockchain anchoring.

ProofChain allows users to prove that a digital document existed at a specific time without storing the actual file content. It hashes files, stores them in a local tamper-evident ledger, batches multiple proofs into a Merkle tree, and anchors the root to a blockchain testnet.

---

✨ Features

· Tamper-Evident Local Ledger: Hash-chained blocks stored in SQLite. Any modification to past records breaks the chain.
· Merkle Tree Batching: Groups multiple file hashes into a Merkle tree for efficient verification and reduced blockchain footprint.
· Blockchain Anchoring: Anchors Merkle roots to the Ethereum Sepolia testnet (mock mode by default, real RPC support ready).
· JSON Proof Export: Export verifiable proofs as structured JSON containing block hash, Merkle root, and anchor transaction hash.
· CLI Interface: Fast, clean, and easy-to-use command-line tool.

---

🏗️ Architecture

```
┌─────────────────────────────────────────┐
│           ProofChain CLI                │
│  (add, verify, list, check, merkle,     │
│   anchor, export, status)               │
└──────────────┬──────────────────────────┘
               │
               ▼
┌─────────────────────────────────────────┐
│         Local SQLite Ledger             │
│  (Blocks + Hash Chaining + Indexes)     │
└──────────────┬──────────────────────────┘
               │
               ▼
┌─────────────────────────────────────────┐
│        Merkle Tree Engine               │
│  (Batching leaves, computing root)      │
└──────────────┬──────────────────────────┘
               │
               ▼
┌─────────────────────────────────────────┐
│   Blockchain Anchor (Sepolia/Mock)      │
│   (Stores Merkle Root on-chain)         │
└─────────────────────────────────────────┘
```

---

🚀 Quick Start

Prerequisites

· Rust (1.70 or later)

Build

```bash
cargo build --release
```

Usage Examples

1. Initialize the ledger

```bash
cargo run -- init
```

2. Register a file

```bash
echo "My secret contract" > contract.txt
cargo run -- add contract.txt
```

3. Verify a file

```bash
cargo run -- verify contract.txt
# Output: ✓ File verified!
```

4. Tamper Detection

```bash
echo "Hacker modified this" > contract.txt
cargo run -- verify contract.txt
# Output: ✗ File not found in registry (hash mismatch)
```

5. Check ledger integrity

```bash
cargo run -- check
# Output: Integrity: ✓ VALID
```

6. Merkle Batching & Anchoring

```bash
cargo run -- merkle    # Creates a Merkle root from unbatched blocks
cargo run -- anchor    # Anchors the root to the blockchain (Mock/Sepolia)
cargo run -- status    # Shows anchored trees and latest TX hash
```

7. Export Proof

```bash
cargo run -- export contract.txt
# Outputs JSON with block hash, merkle root, and anchor TX hash
```

---

🧪 Testing

Run the automated test suite:

```bash
cargo test
```

(All tests pass with 0 compiler warnings)

---

📊 Export Format

```json
{
  "file_name": "contract.txt",
  "file_size": 18,
  "item_hash": "a1b2c3d4...",
  "block_height": 2,
  "block_hash": "f8a3ddd1...",
  "timestamp": 1789805952,
  "merkle_root": "7191d7c5...",
  "anchor_tx_hash": "0xd1302f2b..."
}
```

---

🛠️ Tech Stack

· Language: Rust
· Database: SQLite (via rusqlite)
· Cryptography: SHA-256 (via sha2)
· CLI Framework: clap
· Serialization: serde + serde_json

---

🎓 Capstone Project Context

This project was developed as a final capstone for a Blockchain Engineering course. It demonstrates core blockchain concepts including:

· Cryptographic hashing and integrity verification
· Hash-chained data structures (local blockchain)
· Merkle trees for data aggregation
· Blockchain anchoring and public trust

---

📝 License

MIT License. See LICENSE for details.