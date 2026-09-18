# ProofChain: Project Proposal & System Design Document

## Phase 1: Project Planning and System Design

---

## 1. Problem Statement

### Current Challenge
Users frequently need to prove that a digital document, report, image, or file existed at a specific point in time. Traditional notarization systems face several critical issues:

- **Privacy Concerns**: Central databases store full file content, exposing sensitive information
- **Storage Overhead**: Maintaining complete copies of all documents requires significant storage infrastructure
- **Trust Issues**: Centralized systems create single points of failure and require blind trust in the operator
- **Tampering Risk**: Central databases can be modified without detection
- **Verification Complexity**: Third-party verification is often expensive or impossible

### Proposed Solution
ProofChain is a blockchain-based proof-of-existence system that:
- Stores only cryptographic hashes (SHA-256) instead of file content
- Creates a tamper-evident local ledger using hash chaining
- Batches multiple proofs using Merkle trees for efficiency
- Anchors Merkle roots to Ethereum Sepolia testnet for immutable timestamping
- Enables anyone to verify file existence without revealing content

---

## 2. System Workflow

### High-Level Workflow

```
┌─────────────┐     ┌──────────────┐     ┌─────────────┐     ┌──────────────┐
│   User      │────▶│  File Hash   │────▶│   Local     │────▶│   Merkle     │
│ Submits     │     │  Calculation │     │   Ledger    │     │   Tree       │
│   File      │     │  (SHA-256)   │     │   Entry     │     │   Batching   │
└─────────────┘     └──────────────┘     └─────────────┘     └──────────────┘
                                                                   │
                                                                   ▼
┌─────────────┐     ┌──────────────┐     ┌─────────────┐     ┌──────────────┐
│   User      │◀────│    Proof     │◀────│  Blockchain │◀────│   Anchor     │
│ Verifies    │     │ Verification │     │   Storage   │     │   Root to    │
│   File      │     │              │     │  (Sepolia)  │     │   Testnet    │
└─────────────┘     └──────────────┘     └─────────────┘     └──────────────┘
```

### Detailed Step-by-Step Process

#### Phase A: File Registration
1. **User submits file** via CLI command
2. **System calculates SHA-256 hash** of the file content
3. **Metadata extraction**: filename, size, timestamp (UTC)
4. **Create ledger entry** with hash and metadata
5. **Store in SQLite database** with block structure

#### Phase B: Batch Processing
1. **Collect multiple file hashes** over a time window or count threshold
2. **Build Merkle tree** from collected hashes
3. **Calculate Merkle root** (top hash of the tree)
4. **Create batch block** containing Merkle root

#### Phase C: Blockchain Anchoring
1. **Prepare transaction** with Merkle root as data
2. **Submit to Ethereum Sepolia testnet**
3. **Store transaction hash** in local ledger
4. **Wait for confirmation** (configurable number of blocks)

#### Phase D: Verification
1. **User provides file** for verification
2. **System recalculates SHA-256 hash**
3. **Search ledger** for matching hash
4. **Verify hash chain integrity** (detect tampering)
5. **Return proof**: timestamp, block height, transaction hash (if anchored)

---

## 3. System Architecture

### Architecture Diagram

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

### Component Descriptions

#### 1. CLI Module (`cli/`)
- **Purpose**: User interface for all operations
- **Commands**:
  - `register`: Register a file hash
  - `verify`: Verify a file's existence proof
  - `list`: List registered proofs
  - `merkle`: Build Merkle tree from pending hashes
  - `anchor`: Anchor Merkle root to blockchain
  - `status`: Check anchoring status
  - `export`: Export proof certificate

#### 2. File Processor (`file_processor/`)
- **Purpose**: Handle file operations and hashing
- **Responsibilities**:
  - Read file content
  - Calculate SHA-256 hash
  - Extract metadata (size, name, timestamp)
  - Validate file accessibility

#### 3. Block Manager (`blockchain/`)
- **Purpose**: Manage local blockchain ledger
- **Responsibilities**:
  - Create new blocks
  - Link blocks via previous hash
  - Validate hash chain integrity
  - Detect tampering attempts

#### 4. Merkle Builder (`merkle/`)
- **Purpose**: Construct and manage Merkle trees
- **Responsibilities**:
  - Build tree from leaf nodes (file hashes)
  - Calculate Merkle root
  - Generate Merkle proofs for individual leaves
  - Optimize batch processing

#### 5. Blockchain Anchor (`anchor/`)
- **Purpose**: Interface with Ethereum testnet
- **Responsibilities**:
  - Prepare smart contract calls
  - Submit transactions
  - Track confirmations
  - Store transaction receipts

#### 6. Storage Layer (`storage/`)
- **Purpose**: Persistent data management
- **Technology**: SQLite via rusqlite
- **Tables**:
  - `blocks`: Block headers and metadata
  - `proofs`: Individual file proofs
  - `merkle_trees`: Merkle tree data
  - `anchors`: Blockchain anchoring records

---

## 4. Data Model & Database Schema

### Entity Relationship Diagram

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
│ nonce               │       │ merkle_proof        │
│ created_at          │       │ status              │
└─────────────────────┘       └─────────────────────┘
         │                              │
         │ 1:1                          │
         ▼                              │
┌─────────────────────┐                 │
│      ANCHORS        │                 │
├─────────────────────┤                 │
│ anchor_id (PK)      │                 │
│ block_id (FK)       │                 │
│ merkle_root         │                 │
│ tx_hash             │                 │
│ network             │                 │
│ confirmations       │                 │
│ anchored_at         │─────────────────┘
│ status              │
└─────────────────────┘
```

### Table Schemas

#### Table: `blocks`
```sql
CREATE TABLE blocks (
    block_id INTEGER PRIMARY KEY AUTOINCREMENT,
    block_height INTEGER UNIQUE NOT NULL,
    timestamp TEXT NOT NULL,              -- ISO 8601 UTC
    previous_hash TEXT NOT NULL,          -- Hex-encoded SHA-256
    merkle_root TEXT NOT NULL,            -- Hex-encoded SHA-256
    block_hash TEXT UNIQUE NOT NULL,      -- Hex-encoded SHA-256
    nonce INTEGER DEFAULT 0,
    version INTEGER DEFAULT 1,
    created_at TEXT DEFAULT (datetime('now'))
);

CREATE INDEX idx_blocks_height ON blocks(block_height);
CREATE INDEX idx_blocks_hash ON blocks(block_hash);
```

#### Table: `proofs`
```sql
CREATE TABLE proofs (
    proof_id INTEGER PRIMARY KEY AUTOINCREMENT,
    block_id INTEGER NOT NULL,
    item_hash TEXT NOT NULL,              -- Hex-encoded SHA-256 of file
    file_name TEXT NOT NULL,
    file_size INTEGER NOT NULL,
    timestamp TEXT NOT NULL,              -- ISO 8601 UTC
    merkle_index INTEGER,                 -- Position in Merkle tree
    merkle_proof TEXT,                    -- JSON array of sibling hashes
    status TEXT DEFAULT 'pending',        -- pending, batched, anchored
    created_at TEXT DEFAULT (datetime('now')),
    FOREIGN KEY (block_id) REFERENCES blocks(block_id)
);

CREATE INDEX idx_proofs_hash ON proofs(item_hash);
CREATE INDEX idx_proofs_block ON proofs(block_id);
CREATE INDEX idx_proofs_status ON proofs(status);
```

#### Table: `merkle_trees`
```sql
CREATE TABLE merkle_trees (
    tree_id INTEGER PRIMARY KEY AUTOINCREMENT,
    root_hash TEXT UNIQUE NOT NULL,
    leaf_count INTEGER NOT NULL,
    tree_data TEXT NOT NULL,              -- JSON representation of tree
    created_at TEXT DEFAULT (datetime('now'))
);

CREATE INDEX idx_merkle_root ON merkle_trees(root_hash);
```

#### Table: `anchors`
```sql
CREATE TABLE anchors (
    anchor_id INTEGER PRIMARY KEY AUTOINCREMENT,
    block_id INTEGER UNIQUE NOT NULL,
    merkle_root TEXT NOT NULL,
    tx_hash TEXT UNIQUE,                  -- Ethereum transaction hash
    network TEXT DEFAULT 'sepolia',
    confirmations INTEGER DEFAULT 0,
    required_confirmations INTEGER DEFAULT 6,
    gas_used INTEGER,
    gas_price TEXT,
    status TEXT DEFAULT 'pending',        -- pending, submitted, confirmed, failed
    error_message TEXT,
    submitted_at TEXT,
    confirmed_at TEXT,
    FOREIGN KEY (block_id) REFERENCES blocks(block_id)
);

CREATE INDEX idx_anchors_tx ON anchors(tx_hash);
CREATE INDEX idx_anchors_status ON anchors(status);
```

#### Table: `config`
```sql
CREATE TABLE config (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,
    updated_at TEXT DEFAULT (datetime('now'))
);

-- Default configuration
INSERT INTO config (key, value) VALUES 
    ('network', 'sepolia'),
    ('required_confirmations', '6'),
    ('batch_size', '100'),
    ('auto_anchor', 'false');
```

---

## 5. Block Schema Design

### Block Structure (Rust)

```rust
/// Represents a block in the local ledger
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    /// Unique block identifier (auto-increment in DB)
    pub block_id: Option<i64>,
    
    /// Position in the blockchain (0 = genesis)
    pub block_height: u64,
    
    /// UTC timestamp of block creation
    pub timestamp: DateTime<Utc>,
    
    /// SHA-256 hash of previous block (all zeros for genesis)
    pub previous_hash: String,
    
    /// Merkle root of all transactions/hashes in this block
    pub merkle_root: String,
    
    /// SHA-256 hash of this block's header
    pub block_hash: String,
    
    /// Nonce for future proof-of-work (currently unused)
    pub nonce: u64,
    
    /// Schema version
    pub version: u32,
}

/// Block header used for hashing
#[derive(Debug, Clone)]
pub struct BlockHeader {
    pub block_height: u64,
    pub timestamp: i64,  // Unix timestamp
    pub previous_hash: String,
    pub merkle_root: String,
    pub nonce: u64,
    pub version: u32,
}
```

### Block Hash Calculation

The block hash is computed as:
```
block_hash = SHA256(
    block_height || 
    timestamp || 
    previous_hash || 
    merkle_root || 
    nonce || 
    version
)
```

Where `||` denotes concatenation of binary representations.

### Genesis Block

```rust
Block {
    block_id: Some(1),
    block_height: 0,
    timestamp: Unix epoch (1970-01-01T00:00:00Z),
    previous_hash: "0000000000000000000000000000000000000000000000000000000000000000",
    merkle_root: "0000000000000000000000000000000000000000000000000000000000000000",
    block_hash: calculated,
    nonce: 0,
    version: 1,
}
```

---

## 6. Merkle Tree Structure

### Design Overview

The Merkle tree implementation will:
- Use SHA-256 as the hash function
- Support arbitrary number of leaves (file hashes)
- Handle odd number of nodes by duplicating last node
- Provide efficient inclusion proofs

### Tree Structure

```
                    Root (Hash AB + CD)
                   /                     \
            Hash(A + B)              Hash(C + D)
            /         \              /         \
        Hash(A)    Hash(B)      Hash(C)    Hash(D)
          |          |            |          |
      Leaf A     Leaf B       Leaf C     Leaf D
    (File 1)   (File 2)     (File 3)   (File 4)
```

### Merkle Node Types

```rust
/// Node in a Merkle tree
#[derive(Debug, Clone)]
pub enum MerkleNode {
    /// Leaf node containing a file hash
    Leaf {
        hash: String,
        data: Option<String>,  // Optional metadata
    },
    /// Internal node combining two child hashes
    Branch {
        hash: String,
        left: Box<MerkleNode>,
        right: Box<MerkleNode>,
    },
}

/// Complete Merkle tree with utility methods
pub struct MerkleTree {
    root: MerkleNode,
    leaves: Vec<String>,
    layers: Vec<Vec<String>>,  // All levels for proof generation
}

/// Proof of inclusion for a leaf
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MerkleProof {
    /// The leaf hash being proved
    pub leaf: String,
    
    /// Index of the leaf in the tree
    pub index: usize,
    
    /// Ordered list of sibling hashes from leaf to root
    pub proof: Vec<MerkleProofStep>,
    
    /// The expected root hash
    pub root: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MerkleProofStep {
    /// Hash of the sibling node
    pub hash: String,
    
    /// Position relative to the path (left or right)
    pub position: Position,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Position {
    Left,
    Right,
}
```

### Merkle Proof Verification

To verify a file's inclusion:
1. Start with the file hash (leaf)
2. For each step in the proof:
   - If position is Left: hash = SHA256(sibling + current)
   - If position is Right: hash = SHA256(current + sibling)
3. Compare final hash with Merkle root

---

## 7. Blockchain Anchoring Strategy

### Approach: Ethereum Sepolia Testnet

We will anchor Merkle roots to Ethereum Sepolia testnet using a simple smart contract.

### Smart Contract Design

```solidity
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

contract ProofChainAnchor {
    event ProofAnchored(
        bytes32 indexed merkleRoot,
        uint256 timestamp,
        address anchoredBy,
        string metadata
    );
    
    mapping(bytes32 => uint256) public anchors;
    mapping(bytes32 => address) public anchoredBy;
    
    function anchor(bytes32 _merkleRoot, string memory _metadata) 
        external 
        returns (bool) 
    {
        require(anchors[_merkleRoot] == 0, "Already anchored");
        
        anchors[_merkleRoot] = block.timestamp;
        anchoredBy[_merkleRoot] = msg.sender;
        
        emit ProofAnchored(_merkleRoot, block.timestamp, msg.sender, _metadata);
        
        return true;
    }
    
    function verify(bytes32 _merkleRoot) 
        external 
        view 
        returns (uint256 timestamp, bool exists) 
    {
        return (anchors[_merkleRoot], anchors[_merkleRoot] > 0);
    }
}
```

### Anchoring Methods

#### Method 1: Direct Contract Call (Recommended)
- Deploy contract to Sepolia
- Use Rust library (alloy or ethers-rs) to call `anchor()`
- Include Merkle root as `bytes32`
- Store transaction hash locally

#### Method 2: Transaction Data Embedding
- Send ETH transaction with Merkle root in input data
- No smart contract required
- Less structured but simpler

#### Method 3: OP_RETURN-like Pattern (Not for Ethereum)
- Not applicable to Ethereum (use Method 1 or 2)

### Implementation Plan

**Phase 1**: Design and document (current phase)
**Phase 2**: Implement local ledger without anchoring
**Phase 3**: Add Sepolia testnet integration
  - Deploy smart contract
  - Implement Rust client using alloy
  - Add configuration for RPC endpoint and private key
  - Handle transaction submission and confirmation tracking

### Configuration Requirements

Users will need to provide:
- Sepolia RPC endpoint (e.g., Infura, Alchemy, or public RPC)
- Wallet private key (securely stored)
- Contract address (after deployment)
- Gas settings

Example configuration:
```toml
[blockchain]
network = "sepolia"
rpc_url = "https://sepolia.infura.io/v3/YOUR_KEY"
contract_address = "0x..."
private_key_env = "PROOFCHAIN_PRIVATE_KEY"
confirmations_required = 6
gas_limit = 100000
```

---

## 8. CLI Command Design

### Command Structure

Using `clap` with derive API for type-safe CLI parsing.

### Commands Overview

```bash
proofchain <COMMAND>

Commands:
  register    Register a file's hash in the ledger
  verify      Verify if a file was previously registered
  list        List registered proofs with optional filters
  merkle      Build Merkle tree from pending proofs
  anchor      Anchor Merkle root to blockchain
  status      Check anchoring status
  export      Export proof certificate
  init        Initialize the database and configuration
  config      Manage configuration settings
  help        Print help message
```

### Detailed Command Specifications

#### 1. `register` - Register a File

```bash
proofchain register [OPTIONS] <FILE_PATH>

Arguments:
  <FILE_PATH>    Path to the file to register

Options:
  -n, --name <NAME>      Custom name for the file (default: filename)
  -m, --metadata <JSON>  Additional metadata as JSON string
  -b, --batch-id <ID>    Assign to specific batch (optional)
  -q, --quiet            Suppress output, return only hash
  -h, --help             Print help
```

**Output Example:**
```
✓ File registered successfully
  File: document.pdf
  Size: 1,048,576 bytes
  Hash: a3f2b8c9d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8e9f0a1
  Timestamp: 2024-01-15T10:30:45Z
  Status: pending (waiting for batch)
  Block Height: 5
```

#### 2. `verify` - Verify a File

```bash
proofchain verify [OPTIONS] <FILE_PATH>

Arguments:
  <FILE_PATH>    Path to the file to verify

Options:
  -H, --hash <HASH>     Provide hash directly instead of file
  -f, --full            Show full verification details
  -j, --json            Output as JSON
  -h, --help            Print help
```

**Output Example:**
```
✓ Verification successful
  File: document.pdf
  Hash: a3f2b8c9d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8e9f0a1
  Registered: 2024-01-15T10:30:45Z
  Block Height: 5
  Merkle Root: b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8e9f0a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5
  Anchored: Yes
  Network: Sepolia
  TX Hash: 0x7f8a9b0c1d2e3f4a5b6c7d8e9f0a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8
  Confirmations: 12
```

#### 3. `list` - List Proofs

```bash
proofchain list [OPTIONS]

Options:
  -s, --status <STATUS>    Filter by status (pending, batched, anchored)
  -f, --from <DATE>        Filter from date (ISO 8601)
  -t, --to <DATE>          Filter to date (ISO 8601)
  -l, --limit <N>          Limit results (default: 50)
  -o, --offset <N>         Offset for pagination
  -j, --json               Output as JSON
  -h, --help               Print help
```

#### 4. `merkle` - Build Merkle Tree

```bash
proofchain merkle [OPTIONS]

Options:
  -i, --input <STATUS>     Input filter (default: pending)
  -o, --output-file <PATH> Save tree to file (JSON)
  -v, --verbose            Show detailed tree structure
  --dry-run                Simulate without committing
  -h, --help               Print help
```

**Output Example:**
```
✓ Merkle tree built successfully
  Leaves: 47
  Tree Height: 6
  Merkle Root: c5d6e7f8a9b0c1d2e3f4a5b6c7d8e9f0a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6
  Block Height: 6
  Ready for anchoring
```

#### 5. `anchor` - Anchor to Blockchain

```bash
proofchain anchor [OPTIONS]

Options:
  -r, --root <HASH>        Specific Merkle root to anchor
  -l, --latest             Anchor latest unanchored Merkle root
  -n, --network <NET>      Network override (default: sepolia)
  --gas-price <GWEI>       Custom gas price
  --gas-limit <LIMIT>      Custom gas limit
  --dry-run                Simulate transaction
  -h, --help               Print help
```

**Output Example:**
```
✓ Transaction submitted
  Merkle Root: c5d6e7f8a9b0c1d2e3f4a5b6c7d8e9f0a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6
  TX Hash: 0x7f8a9b0c1d2e3f4a5b6c7d8e9f0a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8
  Network: Sepolia
  Gas Used: 65,432
  Status: pending (waiting for confirmations)
  
  Monitor with: proofchain status --tx 0x7f8a...
```

#### 6. `status` - Check Status

```bash
proofchain status [OPTIONS]

Options:
  -t, --tx-hash <HASH>     Check specific transaction
  -r, --root <HASH>        Check by Merkle root
  -b, --block <HEIGHT>     Check by block height
  -w, --watch              Watch for updates
  -h, --help               Print help
```

#### 7. `export` - Export Proof Certificate

```bash
proofchain export [OPTIONS] <IDENTIFIER>

Arguments:
  <IDENTIFIER>    Hash, file path, or proof ID

Options:
  -o, --output <PATH>      Output file path (default: stdout)
  -f, --format <FORMAT>    Format: json, pdf, html (default: json)
  --include-chain          Include full hash chain
  --include-merkle         Include Merkle proof
  -h, --help               Print help
```

#### 8. `init` - Initialize System

```bash
proofchain init [OPTIONS]

Options:
  -d, --database <PATH>    Database path (default: ~/.proofchain/ledger.db)
  -c, --config <PATH>      Config file path
  --network <NETWORK>      Default network (sepolia, goerli, etc.)
  --force                  Overwrite existing database
  -h, --help               Print help
```

#### 9. `config` - Manage Configuration

```bash
proofchain config [OPTIONS] [KEY] [VALUE]

Arguments:
  <KEY>      Configuration key
  <VALUE>    Configuration value

Options:
  -l, --list                 List all configuration
  -d, --delete <KEY>         Delete configuration key
  --reset                    Reset to defaults
  -h, --help                 Print help
```

### Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | General error |
| 2 | File not found |
| 3 | Invalid hash or format |
| 4 | Database error |
| 5 | Verification failed |
| 6 | Blockchain error |
| 7 | Configuration error |

---

## 9. Phase-Wise Development Plan

### Phase 1: Project Planning and System Design ✅ (Current)
**Duration**: 1 week
**Deliverables**:
- [x] Project proposal (this document)
- [x] System architecture diagram
- [x] Database schema design
- [x] CLI command specifications
- [x] Phase-wise development plan
- [x] Technology stack finalized
- [x] Repository setup with Cargo.toml

**Completion Criteria**:
- Project scope clearly defined
- System design approved
- Required tools and libraries selected
- Team alignment on approach

---

### Phase 2: Core Infrastructure
**Duration**: 2 weeks
**Tasks**:
1. **Project Setup**
   - Initialize Rust project structure
   - Configure dependencies
   - Set up directory structure
   - Configure linting and formatting (rustfmt, clippy)

2. **Database Layer**
   - Implement SQLite connection management
   - Create database schema migrations
   - Implement CRUD operations for tables
   - Add connection pooling

3. **Cryptographic Utilities**
   - Implement SHA-256 file hashing
   - Create hex encoding/decoding utilities
   - Add hash validation functions

4. **Block Management**
   - Implement Block struct and serialization
   - Create genesis block
   - Implement block hash calculation
   - Build block creation logic
   - Add hash chain validation

5. **CLI Foundation**
   - Set up clap command structure
   - Implement `init` command
   - Implement `config` command
   - Add error handling framework

**Deliverables**:
- Working SQLite database with schema
- Functional block creation and validation
- Basic CLI with initialization commands
- Unit tests for core components

---

### Phase 3: File Registration and Verification
**Duration**: 2 weeks
**Tasks**:
1. **File Processing**
   - Implement file reading and streaming
   - Add SHA-256 hash computation
   - Extract file metadata
   - Handle large files efficiently

2. **Proof Registration**
   - Implement `register` command
   - Create proof records in database
   - Add duplicate detection
   - Implement batch assignment logic

3. **Verification System**
   - Implement `verify` command
   - Search proofs by hash
   - Verify hash chain integrity
   - Generate verification reports

4. **Listing and Querying**
   - Implement `list` command
   - Add filtering and pagination
   - Support JSON output
   - Implement search functionality

5. **Export Functionality**
   - Implement `export` command
   - Create JSON certificates
   - Add PDF generation (optional)
   - Include verification instructions

**Deliverables**:
- Complete file registration workflow
- Functional verification system
- CLI commands: register, verify, list, export
- Integration tests

---

### Phase 4: Merkle Tree Implementation
**Duration**: 2 weeks
**Tasks**:
1. **Merkle Tree Construction**
   - Implement MerkleNode enum
   - Build tree from leaves
   - Handle odd number of nodes
   - Calculate Merkle root

2. **Proof Generation**
   - Generate Merkle inclusion proofs
   - Serialize proofs to JSON
   - Optimize proof size

3. **Proof Verification**
   - Implement proof verification algorithm
   - Validate against stored root
   - Add unit tests for edge cases

4. **Batch Management**
   - Implement `merkle` command
   - Group pending proofs into batches
   - Update block with Merkle root
   - Link proofs to blocks

5. **Status Tracking**
   - Implement `status` command
   - Track batch status
   - Display tree statistics

**Deliverables**:
- Complete Merkle tree implementation
- Batch processing system
- CLI commands: merkle, status
- Comprehensive test suite

---

### Phase 5: Blockchain Anchoring
**Duration**: 3 weeks
**Tasks**:
1. **Smart Contract Development**
   - Write Solidity contract
   - Deploy to Sepolia testnet
   - Verify on Etherscan
   - Document contract ABI

2. **Blockchain Client**
   - Integrate alloy or ethers-rs
   - Implement contract interaction
   - Add transaction building
   - Handle gas estimation

3. **Transaction Management**
   - Implement `anchor` command
   - Submit transactions
   - Track confirmations
   - Handle failures and retries

4. **Wallet Management**
   - Secure private key handling
   - Environment variable support
   - Add wallet configuration
   - Implement security best practices

5. **Integration Testing**
   - Test on Sepolia testnet
   - Mock blockchain for unit tests
   - End-to-end testing
   - Performance optimization

**Deliverables**:
- Deployed smart contract on Sepolia
- Functional blockchain anchoring
- CLI command: anchor
- Transaction monitoring system
- Security documentation

---

### Phase 6: Testing and Quality Assurance
**Duration**: 2 weeks
**Tasks**:
1. **Unit Testing**
   - Achieve >80% code coverage
   - Test edge cases
   - Mock external dependencies
   - Property-based testing

2. **Integration Testing**
   - Test component interactions
   - Database integration tests
   - CLI command testing with assert_cmd
   - Blockchain integration tests

3. **End-to-End Testing**
   - Complete workflow testing
   - Multi-user scenarios
   - Performance under load
   - Recovery from failures

4. **Security Audit**
   - Review cryptographic implementations
   - Validate hash chain integrity
   - Check for common vulnerabilities
   - Secure key management audit

5. **Documentation**
   - API documentation
   - User guide
   - Developer guide
   - Troubleshooting guide

**Deliverables**:
- Comprehensive test suite
- Code coverage report
- Security audit findings
- Complete documentation

---

### Phase 7: Polish and Deployment
**Duration**: 1 week
**Tasks**:
1. **Performance Optimization**
   - Profile application
   - Optimize database queries
   - Improve Merkle tree construction
   - Reduce memory footprint

2. **User Experience**
   - Improve error messages
   - Add progress indicators
   - Enhance help documentation
   - Create examples and tutorials

3. **Packaging**
   - Create release builds
   - Package for multiple platforms
   - Set up CI/CD pipeline
   - Version management

4. **Final Review**
   - Code review
   - Documentation review
   - Demo preparation
   - Presentation materials

**Deliverables**:
- Production-ready binaries
- Installation guides
- CI/CD pipeline
- Final presentation

---

## 10. Risk Assessment and Mitigation

### Technical Risks

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| Blockchain transaction failures | Medium | High | Implement retry logic, gas estimation, fallback RPC endpoints |
| Database corruption | Low | High | Regular backups, WAL mode, integrity checks |
| Hash collisions (SHA-256) | Extremely Low | Critical | Use standard SHA-256, document theoretical risk |
| Private key compromise | Medium | Critical | Never store keys in code, use environment variables, consider HSM |
| Smart contract bugs | Medium | High | Thorough testing, audit, use simple contract design |
| Performance degradation with scale | Medium | Medium | Index optimization, pagination, batch processing |

### Project Risks

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| Scope creep | High | Medium | Strict adherence to phased approach, clear requirements |
| Timeline delays | Medium | Medium | Buffer time in each phase, prioritize MVP features |
| Learning curve for blockchain | Medium | Low | Allocate time for research, use well-documented libraries |
| Testnet instability | Medium | Low | Use multiple RPC providers, implement retry logic |

---

## 11. Success Metrics

### Functional Metrics
- ✓ File hashing works correctly (SHA-256)
- ✓ Blocks are properly chained via previous hash
- ✓ Tampering is detectable through hash chain verification
- ✓ Merkle trees are constructed correctly
- ✓ Merkle proofs can be generated and verified
- ✓ Transactions are successfully submitted to Sepolia
- ✓ Files can be verified against stored proofs

### Quality Metrics
- Code coverage > 80%
- All critical paths tested
- No high-severity security vulnerabilities
- CLI help is comprehensive and accurate
- Error messages are clear and actionable

### Performance Metrics
- File hashing: < 1 second for 100MB file
- Proof registration: < 500ms
- Verification: < 200ms
- Merkle tree construction: < 1 second for 1000 leaves
- Blockchain anchoring: dependent on network (target < 2 minutes for confirmation)

---

## 12. Future Enhancements (Out of Scope for MVP)

### Potential Extensions
1. **REST API**: Expose functionality via HTTP API
2. **Multiple Blockchain Support**: Polygon, Bitcoin, other chains
3. **Batch Operations**: Bulk registration and verification
4. **GUI Application**: Desktop or web interface
5. **Distributed Ledger**: Multi-node synchronization
6. **IPFS Integration**: Store encrypted files with hash-based retrieval
7. **Timestamp Authority**: Integrate with RFC 3161 TSA
8. **Multi-signature Anchoring**: Require multiple parties for anchoring
9. **Audit Logging**: Comprehensive activity logs
10. **Cloud Storage Backends**: PostgreSQL, MongoDB options

---

## 13. Conclusion

ProofChain demonstrates core blockchain concepts through a practical proof-of-existence system. By focusing on a minimal viable product that hashes files, stores them in a tamper-evident ledger, batches with Merkle trees, and anchors to Ethereum testnet, this project provides hands-on experience with:

- Cryptographic hashing (SHA-256)
- Blockchain data structures (hash chaining)
- Merkle trees and inclusion proofs
- Smart contract interaction
- Database design and management
- CLI application development in Rust
- Error handling and testing best practices

The phased approach ensures steady progress while managing complexity. Each phase builds upon the previous one, allowing for iterative development and testing.

---

## Appendix A: Glossary

| Term | Definition |
|------|------------|
| **Proof-of-Existence** | Cryptographic method to prove a file existed at a specific time |
| **SHA-256** | Secure Hash Algorithm producing 256-bit output |
| **Merkle Tree** | Binary tree where leaves are hashes and parent nodes are hashes of children |
| **Merkle Root** | Top hash of a Merkle tree, representing all leaves |
| **Hash Chain** | Sequence of blocks where each contains hash of previous |
| **Sepolia** | Ethereum proof-of-stake testnet |
| **Gas** | Unit measuring computational work on Ethereum |
| **Confirmation** | Number of blocks mined after a transaction's block |

---

## Appendix B: References

1. Bitcoin Whitepaper: https://bitcoin.org/bitcoin.pdf
2. Ethereum Documentation: https://ethereum.org/en/developers/docs/
3. Rust Book: https://doc.rust-lang.org/book/
4. SHA-256 Specification: FIPS PUB 180-4
5. Merkle Trees: https://en.wikipedia.org/wiki/Merkle_tree
6. Clap Crate: https://docs.rs/clap/latest/clap/
7. Rusqlite Crate: https://docs.rs/rusqlite/latest/rusqlite/
8. Alloy (Ethereum Rust): https://github.com/alloy-rs/alloy

---

**Document Version**: 1.0  
**Last Updated**: $(date)  
**Author**: ProofChain Development Team  
**Status**: Approved for Implementation
