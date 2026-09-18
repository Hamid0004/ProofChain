-- ProofChain Database Schema
-- Version: 1.0
-- Description: SQLite database schema for the ProofChain notarization system

-- Enable foreign keys
PRAGMA foreign_keys = ON;

-- ============================================================================
-- TABLE: blocks
-- Description: Stores blockchain block headers
-- ============================================================================
CREATE TABLE IF NOT EXISTS blocks (
    block_id INTEGER PRIMARY KEY AUTOINCREMENT,
    block_height INTEGER UNIQUE NOT NULL,
    timestamp TEXT NOT NULL,              -- ISO 8601 UTC format
    previous_hash TEXT NOT NULL,          -- Hex-encoded SHA-256 (64 chars)
    merkle_root TEXT NOT NULL,            -- Hex-encoded SHA-256 (64 chars)
    block_hash TEXT UNIQUE NOT NULL,      -- Hex-encoded SHA-256 (64 chars)
    nonce INTEGER DEFAULT 0,
    version INTEGER DEFAULT 1,
    created_at TEXT DEFAULT (datetime('now'))
);

-- Indexes for blocks table
CREATE INDEX IF NOT EXISTS idx_blocks_height ON blocks(block_height);
CREATE INDEX IF NOT EXISTS idx_blocks_hash ON blocks(block_hash);
CREATE INDEX IF NOT EXISTS idx_blocks_timestamp ON blocks(timestamp);

-- ============================================================================
-- TABLE: proofs
-- Description: Stores individual file proof records
-- ============================================================================
CREATE TABLE IF NOT EXISTS proofs (
    proof_id INTEGER PRIMARY KEY AUTOINCREMENT,
    block_id INTEGER NOT NULL,
    item_hash TEXT NOT NULL,              -- Hex-encoded SHA-256 of file content
    file_name TEXT NOT NULL,              -- Original filename
    file_size INTEGER NOT NULL,           -- File size in bytes
    timestamp TEXT NOT NULL,              -- ISO 8601 UTC format
    merkle_index INTEGER,                 -- Position in Merkle tree (0-based)
    merkle_proof TEXT,                    -- JSON array of sibling hashes
    status TEXT DEFAULT 'pending' CHECK(status IN ('pending', 'batched', 'anchored')),
    metadata TEXT,                        -- Optional JSON metadata
    created_at TEXT DEFAULT (datetime('now')),
    FOREIGN KEY (block_id) REFERENCES blocks(block_id) ON DELETE CASCADE
);

-- Indexes for proofs table
CREATE INDEX IF NOT EXISTS idx_proofs_hash ON proofs(item_hash);
CREATE INDEX IF NOT EXISTS idx_proofs_block ON proofs(block_id);
CREATE INDEX IF NOT EXISTS idx_proofs_status ON proofs(status);
CREATE INDEX IF NOT EXISTS idx_proofs_timestamp ON proofs(timestamp);
CREATE INDEX IF NOT EXISTS idx_proofs_filename ON proofs(file_name);

-- ============================================================================
-- TABLE: merkle_trees
-- Description: Stores Merkle tree data for batch processing
-- ============================================================================
CREATE TABLE IF NOT EXISTS merkle_trees (
    tree_id INTEGER PRIMARY KEY AUTOINCREMENT,
    root_hash TEXT UNIQUE NOT NULL,       -- Hex-encoded Merkle root
    leaf_count INTEGER NOT NULL,          -- Number of leaves in the tree
    tree_data TEXT NOT NULL,              -- JSON representation of complete tree
    block_id INTEGER,                     -- Reference to block containing this tree
    created_at TEXT DEFAULT (datetime('now')),
    FOREIGN KEY (block_id) REFERENCES blocks(block_id) ON DELETE SET NULL
);

-- Indexes for merkle_trees table
CREATE INDEX IF NOT EXISTS idx_merkle_root ON merkle_trees(root_hash);
CREATE INDEX IF NOT EXISTS idx_merkle_block ON merkle_trees(block_id);

-- ============================================================================
-- TABLE: anchors
-- Description: Stores blockchain anchoring records
-- ============================================================================
CREATE TABLE IF NOT EXISTS anchors (
    anchor_id INTEGER PRIMARY KEY AUTOINCREMENT,
    block_id INTEGER UNIQUE NOT NULL,     -- One anchor per block
    merkle_root TEXT NOT NULL,            -- Hex-encoded Merkle root that was anchored
    tx_hash TEXT UNIQUE,                  -- Ethereum transaction hash (66 chars with 0x)
    network TEXT DEFAULT 'sepolia',       -- Blockchain network name
    confirmations INTEGER DEFAULT 0,      -- Number of block confirmations
    required_confirmations INTEGER DEFAULT 6,
    gas_used INTEGER,                     -- Actual gas used in transaction
    gas_price TEXT,                       -- Gas price in wei
    total_cost TEXT,                      -- Total transaction cost in wei
    status TEXT DEFAULT 'pending' CHECK(status IN ('pending', 'submitted', 'confirmed', 'failed')),
    error_message TEXT,                   -- Error details if failed
    contract_address TEXT,                -- Smart contract address used
    submitted_at TEXT,                    -- When transaction was submitted
    confirmed_at TEXT,                    -- When transaction was confirmed
    created_at TEXT DEFAULT (datetime('now')),
    FOREIGN KEY (block_id) REFERENCES blocks(block_id) ON DELETE CASCADE
);

-- Indexes for anchors table
CREATE INDEX IF NOT EXISTS idx_anchors_tx ON anchors(tx_hash);
CREATE INDEX IF NOT EXISTS idx_anchors_status ON anchors(status);
CREATE INDEX IF NOT EXISTS idx_anchors_network ON anchors(network);
CREATE INDEX IF NOT EXISTS idx_anchors_merkle ON anchors(merkle_root);

-- ============================================================================
-- TABLE: config
-- Description: Stores application configuration
-- ============================================================================
CREATE TABLE IF NOT EXISTS config (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,
    description TEXT,
    updated_at TEXT DEFAULT (datetime('now'))
);

-- Default configuration values
INSERT OR IGNORE INTO config (key, value, description) VALUES 
    ('network', 'sepolia', 'Default blockchain network for anchoring'),
    ('required_confirmations', '6', 'Number of confirmations before considering anchored'),
    ('batch_size', '100', 'Default number of proofs per Merkle tree batch'),
    ('auto_anchor', 'false', 'Automatically anchor when batch is full'),
    ('database_path', '~/.proofchain/ledger.db', 'Default database location'),
    ('gas_limit', '100000', 'Default gas limit for anchoring transactions'),
    ('rpc_timeout', '30', 'RPC request timeout in seconds');

-- ============================================================================
-- VIEW: proof_summary
-- Description: Provides a summary view of proofs with block information
-- ============================================================================
CREATE VIEW IF NOT EXISTS proof_summary AS
SELECT 
    p.proof_id,
    p.item_hash,
    p.file_name,
    p.file_size,
    p.timestamp as proof_timestamp,
    p.status,
    b.block_height,
    b.timestamp as block_timestamp,
    b.merkle_root,
    a.tx_hash,
    a.network,
    a.confirmations,
    a.status as anchor_status
FROM proofs p
LEFT JOIN blocks b ON p.block_id = b.block_id
LEFT JOIN anchors a ON b.block_id = a.block_id;

-- ============================================================================
-- VIEW: chain_status
-- Description: Provides overview of the blockchain ledger status
-- ============================================================================
CREATE VIEW IF NOT EXISTS chain_status AS
SELECT 
    COUNT(DISTINCT b.block_id) as total_blocks,
    MAX(b.block_height) as latest_height,
    MAX(b.timestamp) as latest_block_time,
    COUNT(DISTINCT p.proof_id) as total_proofs,
    COUNT(CASE WHEN p.status = 'pending' THEN 1 END) as pending_proofs,
    COUNT(CASE WHEN p.status = 'batched' THEN 1 END) as batched_proofs,
    COUNT(CASE WHEN p.status = 'anchored' THEN 1 END) as anchored_proofs,
    COUNT(DISTINCT a.anchor_id) as total_anchors,
    COUNT(CASE WHEN a.status = 'confirmed' THEN 1 END) as confirmed_anchors
FROM blocks b
LEFT JOIN proofs p ON 1=1
LEFT JOIN anchors a ON 1=1;

-- ============================================================================
-- TRIGGER: update_config_timestamp
-- Description: Automatically updates timestamp when config changes
-- ============================================================================
CREATE TRIGGER IF NOT EXISTS update_config_timestamp
AFTER UPDATE ON config
BEGIN
    UPDATE config SET updated_at = datetime('now') WHERE key = NEW.key;
END;

-- ============================================================================
-- TRIGGER: ensure_genesis_block
-- Description: Prevents deletion of genesis block
-- ============================================================================
CREATE TRIGGER IF NOT EXISTS prevent_genesis_deletion
BEFORE DELETE ON blocks
WHEN OLD.block_height = 0
BEGIN
    SELECT RAISE(ABORT, 'Cannot delete genesis block');
END;

-- ============================================================================
-- TRIGGER: validate_block_chain
-- Description: Ensures block height continuity
-- ============================================================================
CREATE TRIGGER IF NOT EXISTS validate_block_height
BEFORE INSERT ON blocks
WHEN NEW.block_height > 0
BEGIN
    SELECT CASE 
        WHEN NOT EXISTS (
            SELECT 1 FROM blocks WHERE block_height = NEW.block_height - 1
        )
        THEN RAISE(ABORT, 'Block height must be sequential')
    END;
END;

-- ============================================================================
-- SEED DATA: Genesis Block
-- Description: Initialize the blockchain with a genesis block
-- ============================================================================
INSERT OR IGNORE INTO blocks (
    block_height, 
    timestamp, 
    previous_hash, 
    merkle_root, 
    block_hash, 
    nonce, 
    version
) VALUES (
    0,
    '1970-01-01T00:00:00Z',
    '0000000000000000000000000000000000000000000000000000000000000000',
    '0000000000000000000000000000000000000000000000000000000000000000',
    'genesis_block_hash_placeholder',  -- Will be calculated by application
    0,
    1
);

-- Note: The genesis block hash should be calculated and updated by the application
-- using the proper block hash algorithm

-- ============================================================================
-- COMMENTS / DOCUMENTATION
-- ============================================================================

-- Hash Format:
-- All hashes are stored as lowercase hexadecimal strings
-- SHA-256 produces 64 character hex strings (256 bits = 32 bytes = 64 hex chars)
-- Ethereum transaction hashes include '0x' prefix (66 characters total)

-- Timestamp Format:
-- All timestamps use ISO 8601 format with UTC timezone
-- Example: 2024-01-15T10:30:45Z

-- Status Values:
-- proofs.status: 'pending' | 'batched' | 'anchored'
-- anchors.status: 'pending' | 'submitted' | 'confirmed' | 'failed'

-- Merkle Proof Format (JSON):
-- [
--   {"hash": "abc123...", "position": "left"},
--   {"hash": "def456...", "position": "right"},
--   ...
-- ]

-- Performance Considerations:
-- - All frequently queried columns are indexed
-- - Foreign keys enforce referential integrity
-- - Views provide convenient query interfaces
-- - Triggers maintain data consistency

-- Security Notes:
-- - Enable WAL mode for better concurrency: PRAGMA journal_mode = WAL;
-- - Set synchronous mode: PRAGMA synchronous = NORMAL;
-- - Regular integrity checks recommended: PRAGMA integrity_check;

-- ============================================================================
-- END OF SCHEMA
-- ============================================================================
