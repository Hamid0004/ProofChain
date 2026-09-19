#![allow(dead_code)]
use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};
use hex;
use chrono::{DateTime, Utc};

/// Represents a single block in the local ledger.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    pub id: Option<i64>,
    pub height: u64,
    pub timestamp: i64, // Unix timestamp (seconds since epoch)
    pub prev_hash: String,
    pub item_hash: String,
    pub file_name: String,
    pub file_size: u64,
    pub merkle_root: Option<String>, // Nullable for single items until batched
    pub block_hash: String,
    pub anchor_tx_hash: Option<String>,
}

impl Block {
    /// Creates a new block. The block_hash is computed immediately.
    pub fn new(
        height: u64,
        timestamp: i64,
        prev_hash: String,
        item_hash: String,
        file_name: String,
        file_size: u64,
    ) -> Self {
        let block_hash = Self::compute_block_hash(
            height,
            &prev_hash,
            timestamp,
            &item_hash,
            &file_name,
            file_size,
        );

        Block {
            id: None,
            height,
            timestamp,
            prev_hash,
            item_hash,
            file_name,
            file_size,
            merkle_root: None,
            block_hash,
            anchor_tx_hash: None,
        }
    }

    /// Creates the genesis block (height = 1, prev_hash = 64 zeros)
    pub fn genesis() -> Self {
        let timestamp = Utc::now().timestamp();
        let prev_hash = genesis::PREV_HASH.to_string();
        
        // Genesis block has empty item data but still needs a valid hash
        let block_hash = Self::compute_block_hash(
            genesis::HEIGHT,
            &prev_hash,
            timestamp,
            "",
            "genesis",
            0,
        );

        Block {
            id: None,
            height: genesis::HEIGHT,
            timestamp,
            prev_hash,
            item_hash: String::new(),
            file_name: "genesis".to_string(),
            file_size: 0,
            merkle_root: None,
            block_hash,
            anchor_tx_hash: None,
        }
    }

    /// Computes the deterministic block hash.
    /// Format: SHA256(JSON({height, prev_hash, timestamp, item_hash, file_name, file_size}))
    fn compute_block_hash(
        height: u64,
        prev_hash: &str,
        timestamp: i64,
        item_hash: &str,
        file_name: &str,
        file_size: u64,
    ) -> String {
        // Create a deterministic JSON string manually to avoid whitespace variations
        let data = format!(
            r#"{{"height":{},"prev_hash":"{}","timestamp":{},"item_hash":"{}","file_name":"{}","file_size":{}}}"#,
            height, prev_hash, timestamp, item_hash, file_name, file_size
        );

        let mut hasher = Sha256::new();
        hasher.update(data.as_bytes());
        let result = hasher.finalize();
        hex::encode(result)
    }

    /// Verifies the integrity of the block's own hash.
    pub fn verify_hash(&self) -> bool {
        let expected = Self::compute_block_hash(
            self.height,
            &self.prev_hash,
            self.timestamp,
            &self.item_hash,
            &self.file_name,
            self.file_size,
        );
        self.block_hash == expected
    }
}

/// Represents the Genesis Block constants.
pub mod genesis {
    pub const HEIGHT: u64 = 1;
    pub const PREV_HASH: &str = "0000000000000000000000000000000000000000000000000000000000000000";
}

/// Represents a proof entry (file hash record) - optional helper for UI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Proof {
    pub id: Option<i64>,
    pub block_id: i64,
    pub file_hash: String,
    pub file_name: String,
    pub file_size: u64,
    pub timestamp: DateTime<Utc>,
}

/// Represents a Merkle tree batch
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MerkleTree {
    pub id: Option<i64>,
    pub created_at: DateTime<Utc>,
    pub root_hash: String,
    pub leaf_count: usize,
    pub anchored: bool,
    pub anchor_tx_hash: Option<String>,
}

/// Represents a Merkle tree node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MerkleNode {
    pub hash: String,
    pub left: Option<Box<MerkleNode>>,
    pub right: Option<Box<MerkleNode>>,
    pub is_leaf: bool,
    pub leaf_index: Option<usize>,
}

impl MerkleNode {
    pub fn new_leaf(hash: String, index: usize) -> Self {
        Self {
            hash,
            left: None,
            right: None,
            is_leaf: true,
            leaf_index: Some(index),
        }
    }

    pub fn new_internal(hash: String, left: MerkleNode, right: MerkleNode) -> Self {
        Self {
            hash,
            left: Some(Box::new(left)),
            right: Some(Box::new(right)),
            is_leaf: false,
            leaf_index: None,
        }
    }
}

/// Represents blockchain anchor information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Anchor {
    pub id: Option<i64>,
    pub merkle_tree_id: i64,
    pub transaction_hash: String,
    pub block_number: u64,
    pub timestamp: DateTime<Utc>,
    pub network: String,
    pub status: String,
}
