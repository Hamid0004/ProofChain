# ProofChain Architecture Diagram

## System Architecture Overview

This document contains the architecture diagrams for the ProofChain system.

## 1. High-Level System Architecture

```
┌─────────────────────────────────────────────────────────────────────────┐
│                         PROOFCHAIN SYSTEM ARCHITECTURE                   │
└─────────────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────────────┐
│                            PRESENTATION LAYER                            │
│  ┌─────────────────────────────────────────────────────────────────┐    │
│  │                    Command Line Interface (CLI)                  │    │
│  │  Commands: register, verify, list, anchor, merkle, export       │    │
│  └─────────────────────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────────────────┘
                                   │
                                   ▼
┌─────────────────────────────────────────────────────────────────────────┐
│                           APPLICATION LAYER                              │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐  ┌────────────┐  │
│  │   File       │  │   Block      │  │   Merkle     │  │  Blockchain│  │
│  │   Processor  │  │   Manager    │  │   Builder    │  │  Anchor    │  │
│  └──────────────┘  └──────────────┘  └──────────────┘  └────────────┘  │
│         │                │                │                │            │
│         ▼                ▼                ▼                ▼            │
│  ┌──────────────────────────────────────────────────────────────────┐   │
│  │                    Core Business Logic Layer                     │   │
│  │  • Hash computation (SHA-256)                                    │   │
│  │  • Block creation and validation                                 │   │
│  │  • Hash chain verification                                       │   │
│  │  • Merkle tree construction                                      │   │
│  │  • Transaction preparation                                       │   │
│  └──────────────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────────────┘
                                   │
                                   ▼
┌─────────────────────────────────────────────────────────────────────────┐
│                            DATA LAYER                                    │
│  ┌──────────────────────────────────────────────────────────────────┐   │
│  │                    SQLite Local Ledger                           │   │
│  │  Tables: blocks, proofs, merkle_roots, anchors                   │   │
│  └──────────────────────────────────────────────────────────────────┘   │
│                                                                          │
│  ┌──────────────────────────────────────────────────────────────────┐   │
│  │                 External: Ethereum Sepolia Testnet               │   │
│  │  Smart Contract: ProofChainAnchor                                │   │
│  └──────────────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────────────┐
│                          SUPPORTING MODULES                              │
│  ┌────────────┐  ┌────────────┐  ┌────────────┐  ┌────────────────┐    │
│  │   Crypto   │  │   Error    │  │   Config   │  │     Logger     │    │
│  │  Utilities │  │  Handling  │  │  Manager   │  │     Module     │    │
│  └────────────┘  └────────────┘  └────────────┘  └────────────────┘    │
└─────────────────────────────────────────────────────────────────────────┘
```

## 2. Data Flow Diagram

### File Registration Flow

```
User → CLI → File Processor → SHA-256 Hash → Block Manager → SQLite
                                         ↓
                                    Merkle Builder → Batch Queue
```

### Verification Flow

```
User → CLI → File Processor → SHA-256 Hash → Storage Layer → Verify Chain
                                              ↓
                                        Return Proof Details
```

### Blockchain Anchoring Flow

```
Merkle Root → Blockchain Anchor → Smart Contract → Sepolia Network
                                              ↓
                                        TX Hash → Storage Layer
```

## 3. Component Interaction Diagram

```
┌──────────────────────────────────────────────────────────────────────────┐
│                            CLI Commands                                   │
│  init | register | verify | list | merkle | anchor | status | export     │
└──────────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
┌──────────────────────────────────────────────────────────────────────────┐
│                         Application Services                              │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐  ┌─────────────┐ │
│  │FileService   │  │BlockService  │  │MerkleService │  │AnchorService│ │
│  └──────────────┘  └──────────────┘  └──────────────┘  └─────────────┘ │
└──────────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
┌──────────────────────────────────────────────────────────────────────────┐
│                          Domain Layer                                     │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐  │
│  │  Block   │  │  Proof   │  │  Merkle  │  │Transaction│  │  Config  │  │
│  │          │  │          │  │   Tree   │  │          │  │          │  │
│  └──────────┘  └──────────┘  └──────────┘  └──────────┘  └──────────┘  │
└──────────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
┌──────────────────────────────────────────────────────────────────────────┐
│                        Infrastructure Layer                               │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐  ┌─────────────┐ │
│  │SQLiteRepo    │  │CryptoUtils   │  │Web3Client    │  │FileSystem   │ │
│  └──────────────┘  └──────────────┘  └──────────────┘  └─────────────┘ │
└──────────────────────────────────────────────────────────────────────────┘
```

## 4. Database Entity Relationship

```
┌─────────────────────┐       ┌─────────────────────┐
│       BLOCKS        │       │       PROOFS        │
├─────────────────────┤       ├─────────────────────┤
│ block_id (PK)       │◀──────│ block_id (FK)       │
│ block_height        │   1:N │ proof_id (PK)       │
│ timestamp           │       │ item_hash           │
│ previous_hash       │       │ file_name           │
│ merkle_root         │       │ file_size           │
│ block_hash          │       │ timestamp           │
│ nonce               │       │ merkle_index        │
│ created_at          │       │ merkle_proof        │
└─────────────────────┘       │ status              │
         │                    └─────────────────────┘
         │ 1:1
         ▼
┌─────────────────────┐
│      ANCHORS        │
├─────────────────────┤
│ anchor_id (PK)      │
│ block_id (FK)       │
│ merkle_root         │
│ tx_hash             │
│ network             │
│ confirmations       │
│ status              │
│ submitted_at        │
│ confirmed_at        │
└─────────────────────┘
```

## 5. Merkle Tree Structure

```
                         Root Hash
                      (Merkle Root)
                       /         \
                      /           \
                 Hash AB         Hash CD
                /      \         /      \
               /        \       /        \
           Hash A     Hash B  Hash C    Hash D
             |          |        |         |
          Leaf A     Leaf B   Leaf C    Leaf D
        (File 1)   (File 2) (File 3)  (File 4)
```

## 6. Block Chain Structure

```
┌─────────────┐     ┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│   Genesis   │────▶│   Block 1   │────▶│   Block 2   │────▶│   Block 3   │
│   Block 0   │     │             │     │             │     │             │
├─────────────┤     ├─────────────┤     ├─────────────┤     ├─────────────┤
│ Height: 0   │     │ Height: 1   │     │ Height: 2   │     │ Height: 3   │
│ Prev: 0x00  │     │ Prev: Hash0 │     │ Prev: Hash1 │     │ Prev: Hash2 │
│ Root: 0x00  │     │ Root: MR1   │     │ Root: MR2   │     │ Root: MR3   │
│ Hash: H0    │     │ Hash: H1    │     │ Hash: H2    │     │ Hash: H3    │
└─────────────┘     └─────────────┘     └─────────────┘     └─────────────┘
                                               │
                                               ▼
                                      ┌─────────────────┐
                                      │  Sepolia TX     │
                                      │  0xabc...123    │
                                      └─────────────────┘
```

## 7. Project Directory Structure

```
proofchain/
├── Cargo.toml
├── PROJECT_PROPOSAL.md
├── ARCHITECTURE.md
├── README.md
├── src/
│   ├── main.rs              # CLI entry point
│   ├── lib.rs               # Library root
│   ├── cli/                 # CLI commands
│   │   ├── mod.rs
│   │   ├── init.rs
│   │   ├── register.rs
│   │   ├── verify.rs
│   │   ├── list.rs
│   │   ├── merkle.rs
│   │   ├── anchor.rs
│   │   ├── status.rs
│   │   └── export.rs
│   ├── blockchain/          # Blockchain components
│   │   ├── mod.rs
│   │   ├── block.rs
│   │   ├── chain.rs
│   │   └── validator.rs
│   ├── merkle/              # Merkle tree implementation
│   │   ├── mod.rs
│   │   ├── tree.rs
│   │   ├── proof.rs
│   │   └── builder.rs
│   ├── storage/             # Database layer
│   │   ├── mod.rs
│   │   ├── database.rs
│   │   ├── schema.rs
│   │   ├── repositories/
│   │   │   ├── mod.rs
│   │   │   ├── block_repo.rs
│   │   │   ├── proof_repo.rs
│   │   │   └── anchor_repo.rs
│   │   └── migrations/
│   ├── crypto/              # Cryptographic utilities
│   │   ├── mod.rs
│   │   ├── hasher.rs
│   │   └── utils.rs
│   ├── file_processor/      # File handling
│   │   ├── mod.rs
│   │   └── processor.rs
│   ├── anchor/              # Blockchain anchoring
│   │   ├── mod.rs
│   │   ├── client.rs
│   │   ├── contract.rs
│   │   └── transaction.rs
│   ├── config/              # Configuration management
│   │   ├── mod.rs
│   │   └── settings.rs
│   ├── error/               # Error types
│   │   ├── mod.rs
│   │   └── types.rs
│   └── models/              # Data models
│       ├── mod.rs
│       ├── block.rs
│       ├── proof.rs
│       ├── merkle.rs
│       └── anchor.rs
├── tests/                   # Integration tests
│   ├── cli_tests.rs
│   ├── blockchain_tests.rs
│   └── merkle_tests.rs
├── contracts/               # Solidity smart contracts
│   └── ProofChainAnchor.sol
└── docs/                    # Documentation
    ├── user_guide.md
    ├── api_reference.md
    └── deployment.md
```

## 8. Technology Stack Summary

| Layer | Technology | Purpose |
|-------|-----------|---------|
| Language | Rust 1.98+ | Core implementation |
| CLI Framework | clap 4.4 | Command-line interface |
| Hashing | sha2 0.10 | SHA-256 computation |
| Encoding | hex 0.4 | Hexadecimal encoding |
| Serialization | serde, serde_json | JSON serialization |
| Error Handling | anyhow, thiserror | Error management |
| DateTime | chrono 0.4 | Timestamp handling |
| Database | rusqlite 0.31 | SQLite storage |
| Blockchain | alloy (future) | Ethereum interaction |
| Testing | tempfile, assert_cmd, predicates | Test framework |
| Network | Ethereum Sepolia | Testnet anchoring |

## 9. Security Considerations

```
┌──────────────────────────────────────────────────────────────────────────┐
│                         Security Layers                                   │
├──────────────────────────────────────────────────────────────────────────┤
│ 1. Input Validation                                                       │
│    • File path sanitization                                              │
│    • Hash format validation                                              │
│    • Parameter bounds checking                                           │
├──────────────────────────────────────────────────────────────────────────┤
│ 2. Cryptographic Security                                                 │
│    • SHA-256 for all hashing                                             │
│    • Constant-time comparisons                                           │
│    • Secure random number generation                                     │
├──────────────────────────────────────────────────────────────────────────┤
│ 3. Key Management                                                         │
│    • Private keys via environment variables only                         │
│    • Never stored in code or database                                    │
│    • Consider hardware wallet integration                                │
├──────────────────────────────────────────────────────────────────────────┤
│ 4. Database Security                                                      │
│    • WAL mode for integrity                                              │
│    • Regular backups                                                     │
│    • Integrity checks on startup                                         │
├──────────────────────────────────────────────────────────────────────────┤
│ 5. Network Security                                                       │
│    • HTTPS for RPC connections                                           │
│    • Multiple RPC endpoint fallback                                      │
│    • Transaction replay protection                                       │
└──────────────────────────────────────────────────────────────────────────┘
```

---

**Document Version**: 1.0  
**Last Updated**: 2024  
**Author**: ProofChain Development Team
