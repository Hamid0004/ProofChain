use rusqlite::{Connection, params, OptionalExtension, Row};
use crate::error::Result;
use crate::ledger::models::Block;
use chrono::Utc;

/// Database manager for SQLite operations
pub struct Database {
    conn: Connection,
}

impl Database {
    /// Create a new database connection or open existing one
    pub fn new(db_path: &str) -> Result<Self> {
        let conn = Connection::open(db_path)?;
        
        // Enable WAL mode for better concurrency (PRAGMA returns results, so use execute_batch)
        conn.execute_batch("PRAGMA journal_mode=WAL;")?;
        
        Ok(Self { conn })
    }

    /// Initialize the database with required tables
    pub fn init(&self) -> Result<()> {
        // Create blocks table
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS blocks (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                height INTEGER UNIQUE NOT NULL,
                timestamp INTEGER NOT NULL,
                prev_hash TEXT NOT NULL,
                item_hash TEXT NOT NULL,
                file_name TEXT NOT NULL,
                file_size INTEGER NOT NULL,
                merkle_root TEXT,
                block_hash TEXT NOT NULL,
                anchor_tx_hash TEXT
            )",
            [],
        )?;

        // Create merkle_trees table
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS merkle_trees (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                created_at INTEGER NOT NULL,
                root_hash TEXT NOT NULL UNIQUE,
                leaf_count INTEGER NOT NULL,
                anchored INTEGER NOT NULL DEFAULT 0,
                anchor_tx_hash TEXT
            )",
            [],
        )?;

        // Create anchors table
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS anchors (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                merkle_tree_id INTEGER NOT NULL UNIQUE,
                transaction_hash TEXT NOT NULL,
                block_number INTEGER NOT NULL DEFAULT 0,
                timestamp INTEGER NOT NULL,
                network TEXT NOT NULL,
                status TEXT NOT NULL DEFAULT 'confirmed',
                FOREIGN KEY (merkle_tree_id) REFERENCES merkle_trees(id)
            )",
            [],
        )?;

        // Create indexes for performance
        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_blocks_height ON blocks(height)",
            [],
        )?;
        
        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_blocks_item_hash ON blocks(item_hash)",
            [],
        )?;
        
        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_blocks_file_name ON blocks(file_name)",
            [],
        )?;

        // Create genesis block if it doesn't exist
        let count: i64 = self.conn.query_row("SELECT COUNT(*) FROM blocks", [], |row| row.get(0))?;
        if count == 0 {
            self.create_genesis_block()?;
        }

        Ok(())
    }

    /// Create the genesis block (height = 1, prev_hash = 64 zeros)
    fn create_genesis_block(&self) -> Result<()> {
        let genesis = Block::genesis();
        
        self.conn.execute(
            "INSERT INTO blocks (height, timestamp, prev_hash, item_hash, file_name, file_size, merkle_root, block_hash, anchor_tx_hash)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                genesis.height,
                genesis.timestamp,
                genesis.prev_hash,
                genesis.item_hash,
                genesis.file_name,
                genesis.file_size,
                genesis.merkle_root,
                genesis.block_hash,
                genesis.anchor_tx_hash
            ],
        )?;

        Ok(())
    }

    /// Get the latest block
    pub fn get_latest_block(&self) -> Result<Option<Block>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, height, timestamp, prev_hash, item_hash, file_name, file_size, merkle_root, block_hash, anchor_tx_hash
             FROM blocks ORDER BY height DESC LIMIT 1"
        )?;

        let block = stmt.query_row([], |row| {
            Ok(Block {
                id: Some(row.get(0)?),
                height: row.get(1)?,
                timestamp: row.get(2)?,
                prev_hash: row.get(3)?,
                item_hash: row.get(4)?,
                file_name: row.get(5)?,
                file_size: row.get(6)?,
                merkle_root: row.get(7)?,
                block_hash: row.get(8)?,
                anchor_tx_hash: row.get(9)?,
            })
        }).optional()?;

        Ok(block)
    }

    /// Get the current block height
    pub fn get_current_height(&self) -> Result<u64> {
        let height: Option<i64> = self.conn.query_row(
            "SELECT MAX(height) FROM blocks",
            [],
            |row| row.get(0)
        ).optional()?;

        Ok(height.unwrap_or(0) as u64)
    }

    /// Insert a new block (file proof)
    pub fn insert_block(&self, block: &Block) -> Result<i64> {
        self.conn.execute(
            "INSERT INTO blocks (height, timestamp, prev_hash, item_hash, file_name, file_size, merkle_root, block_hash, anchor_tx_hash)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                block.height,
                block.timestamp,
                block.prev_hash,
                block.item_hash,
                block.file_name,
                block.file_size,
                block.merkle_root,
                block.block_hash,
                block.anchor_tx_hash
            ],
        )?;

        Ok(self.conn.last_insert_rowid())
    }

    /// Check if a file hash already exists
    pub fn block_exists(&self, item_hash: &str) -> Result<bool> {
        let count: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM blocks WHERE item_hash = ?1",
            [item_hash],
            |row| row.get(0)
        )?;

        Ok(count > 0)
    }

    /// Get block by item hash
    pub fn get_block_by_hash(&self, item_hash: &str) -> Result<Option<Block>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, height, timestamp, prev_hash, item_hash, file_name, file_size, merkle_root, block_hash, anchor_tx_hash
             FROM blocks WHERE item_hash = ?1"
        )?;

        let block = stmt.query_row([item_hash], |row| {
            Ok(Block {
                id: Some(row.get(0)?),
                height: row.get(1)?,
                timestamp: row.get(2)?,
                prev_hash: row.get(3)?,
                item_hash: row.get(4)?,
                file_name: row.get(5)?,
                file_size: row.get(6)?,
                merkle_root: row.get(7)?,
                block_hash: row.get(8)?,
                anchor_tx_hash: row.get(9)?,
            })
        }).optional()?;

        Ok(block)
    }

    /// List all blocks with pagination
    pub fn list_blocks(&self, limit: usize, offset: usize) -> Result<Vec<Block>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, height, timestamp, prev_hash, item_hash, file_name, file_size, merkle_root, block_hash, anchor_tx_hash
             FROM blocks ORDER BY height DESC LIMIT ?1 OFFSET ?2"
        )?;

        let blocks = stmt.query_map(params![limit, offset], |row| {
            Ok(Block {
                id: Some(row.get(0)?),
                height: row.get(1)?,
                timestamp: row.get(2)?,
                prev_hash: row.get(3)?,
                item_hash: row.get(4)?,
                file_name: row.get(5)?,
                file_size: row.get(6)?,
                merkle_root: row.get(7)?,
                block_hash: row.get(8)?,
                anchor_tx_hash: row.get(9)?,
            })
        })?;

        let mut result = Vec::new();
        for block in blocks {
            result.push(block?);
        }

        Ok(result)
    }

    /// Get total block count
    pub fn get_block_count(&self) -> Result<usize> {
        let count: i64 = self.conn.query_row("SELECT COUNT(*) FROM blocks", [], |row| row.get(0))?;
        Ok(count as usize)
    }

    /// Verify the integrity of the blockchain (hash chain verification)
    pub fn verify_chain(&self) -> Result<bool> {
        let mut stmt = self.conn.prepare(
            "SELECT id, height, timestamp, prev_hash, item_hash, file_name, file_size, merkle_root, block_hash, anchor_tx_hash
             FROM blocks ORDER BY height ASC"
        )?;

        let blocks = stmt.query_map([], |row| {
            Ok(Block {
                id: Some(row.get(0)?),
                height: row.get(1)?,
                timestamp: row.get(2)?,
                prev_hash: row.get(3)?,
                item_hash: row.get(4)?,
                file_name: row.get(5)?,
                file_size: row.get(6)?,
                merkle_root: row.get(7)?,
                block_hash: row.get(8)?,
                anchor_tx_hash: row.get(9)?,
            })
        })?;

        let mut expected_prev_hash = "0000000000000000000000000000000000000000000000000000000000000000".to_string();
        for block_result in blocks {
            let block = block_result?;
            
            // Verify previous hash linkage
            if block.height > 1 && block.prev_hash != expected_prev_hash {
                return Ok(false);
            }

            // Verify block's own hash
            if !block.verify_hash() {
                return Ok(false);
            }

            expected_prev_hash = block.block_hash.clone();
        }

        Ok(true)
    }

    /// Get statistics
    pub fn get_stats(&self) -> Result<(usize, bool)> {
        let block_count: i64 = self.conn.query_row("SELECT COUNT(*) FROM blocks", [], |row| row.get(0))?;
        let is_valid = self.verify_chain()?;

        Ok((block_count as usize, is_valid))
    }
    fn row_to_block(row: &Row<'_>) -> rusqlite::Result<Block> {
        Ok(Block {
            id: Some(row.get(0)?),
            height: row.get(1)?,
            timestamp: row.get(2)?,
            prev_hash: row.get(3)?,
            item_hash: row.get(4)?,
            file_name: row.get(5)?,
            file_size: row.get(6)?,
            merkle_root: row.get(7)?,
            block_hash: row.get(8)?,
            anchor_tx_hash: row.get(9)?,
        })
    }

    /// Get blocks not yet batched into a Merkle tree
    pub fn get_unbatched_blocks(&self, limit: usize) -> Result<Vec<Block>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, height, timestamp, prev_hash, item_hash, file_name, file_size, merkle_root, block_hash, anchor_tx_hash
             FROM blocks
             WHERE merkle_root IS NULL AND height > 1 AND item_hash != ''
             ORDER BY height ASC LIMIT ?1"
        )?;
        let mapped = stmt.query_map([limit], |row| Self::row_to_block(row))?;
        let mut out = Vec::new();
        for b in mapped { out.push(b?); }
        Ok(out)
    }

    /// Create a Merkle tree batch record
    pub fn create_merkle_batch(&self, root_hash: &str, leaf_count: usize) -> Result<i64> {
        let created_at = Utc::now().timestamp();
        self.conn.execute(
            "INSERT INTO merkle_trees (created_at, root_hash, leaf_count, anchored) VALUES (?1, ?2, ?3, 0)",
            params![created_at, root_hash, leaf_count as i64],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    /// Mark blocks with their Merkle root
    pub fn update_blocks_with_merkle_root(&self, block_ids: &[i64], merkle_root: &str) -> Result<()> {
        for id in block_ids {
            self.conn.execute("UPDATE blocks SET merkle_root = ?1 WHERE id = ?2", params![merkle_root, id])?;
        }
        Ok(())
    }

    /// Get latest unanchored Merkle tree (id, root_hash)
    pub fn get_unanchored_merkle_tree(&self) -> Result<Option<(i64, String)>> {
        let r = self.conn.query_row(
            "SELECT id, root_hash FROM merkle_trees WHERE anchored = 0 ORDER BY created_at DESC LIMIT 1",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        ).optional()?;
        Ok(r)
    }

    /// Save anchor and link it to tree + blocks
    pub fn create_anchor(&self, merkle_tree_id: i64, tx_hash: &str, network: &str) -> Result<()> {
        let ts = Utc::now().timestamp();
        self.conn.execute(
            "INSERT INTO anchors (merkle_tree_id, transaction_hash, block_number, timestamp, network, status) VALUES (?1, ?2, 0, ?3, ?4, 'confirmed')",
            params![merkle_tree_id, tx_hash, ts, network],
        )?;
        self.conn.execute(
            "UPDATE merkle_trees SET anchored = 1, anchor_tx_hash = ?1 WHERE id = ?2",
            params![tx_hash, merkle_tree_id],
        )?;
        self.conn.execute(
            "UPDATE blocks SET anchor_tx_hash = ?1 WHERE merkle_root = (SELECT root_hash FROM merkle_trees WHERE id = ?2)",
            params![tx_hash, merkle_tree_id],
        )?;
        Ok(())
    }

    /// Anchoring statistics: (total trees, anchored trees, latest tx)
    pub fn get_anchor_stats(&self) -> Result<(usize, usize, Option<String>)> {
        let total: i64 = self.conn.query_row("SELECT COUNT(*) FROM merkle_trees", [], |row| row.get(0))?;
        let anchored: i64 = self.conn.query_row("SELECT COUNT(*) FROM merkle_trees WHERE anchored = 1", [], |row| row.get(0))?;
        let latest: Option<String> = self.conn.query_row(
            "SELECT anchor_tx_hash FROM merkle_trees WHERE anchored = 1 ORDER BY created_at DESC LIMIT 1",
            [], |row| row.get(0)).optional()?;
        Ok((total as usize, anchored as usize, latest))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;
    use tempfile::NamedTempFile;
    use crate::ledger::hash;

    #[test]
    fn test_database_init() {
        let temp_file = NamedTempFile::new().unwrap();
        let db_path = temp_file.path().to_str().unwrap();
        
        let db = Database::new(db_path).unwrap();
        db.init().unwrap();
        
        assert!(Path::new(db_path).exists());
        
        // Verify genesis block was created
        let height = db.get_current_height().unwrap();
        assert_eq!(height, 1);
    }

    #[test]
    fn test_insert_block() {
        let temp_file = NamedTempFile::new().unwrap();
        let db_path = temp_file.path().to_str().unwrap();
        
        let db = Database::new(db_path).unwrap();
        db.init().unwrap();
        
        // Create a test file hash
        let test_data = b"test file content";
        let item_hash = hash::hash_bytes(&test_data[..]);
        
        // Create a new block
        let prev_block = db.get_latest_block().unwrap().unwrap();
        let timestamp = Utc::now().timestamp();
        let new_block = Block::new(
            prev_block.height + 1,
            timestamp,
            prev_block.block_hash.clone(),
            item_hash.clone(),
            "test.txt".to_string(),
            test_data.len() as u64,
        );
        
        let block_id = db.insert_block(&new_block).unwrap();
        assert!(block_id > 0);
        
        // Verify block was inserted
        let retrieved = db.get_block_by_hash(&item_hash).unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().file_name, "test.txt");
    }

    #[test]
    fn test_block_chain_integrity() {
        let temp_file = NamedTempFile::new().unwrap();
        let db_path = temp_file.path().to_str().unwrap();
        
        let db = Database::new(db_path).unwrap();
        db.init().unwrap();
        
        // Add multiple blocks
        for i in 0..3 {
            let prev_block = db.get_latest_block().unwrap().unwrap();
            let timestamp = Utc::now().timestamp();
            let item_hash = hash::hash_bytes(format!("test data {}", i).as_bytes());
            
            let new_block = Block::new(
                prev_block.height + 1,
                timestamp,
                prev_block.block_hash.clone(),
                item_hash,
                format!("file_{}.txt", i),
                100,
            );
            
            db.insert_block(&new_block).unwrap();
        }
        
        // Verify chain integrity
        let is_valid = db.verify_chain().unwrap();
        assert!(is_valid);
    }

    #[test]
    fn test_duplicate_hash_prevention() {
        let temp_file = NamedTempFile::new().unwrap();
        let db_path = temp_file.path().to_str().unwrap();
        
        let db = Database::new(db_path).unwrap();
        db.init().unwrap();
        
        let test_data = b"test file content";
        let item_hash = hash::hash_bytes(&test_data[..]);
        
        let prev_block = db.get_latest_block().unwrap().unwrap();
        let timestamp = Utc::now().timestamp();
        let block = Block::new(
            prev_block.height + 1,
            timestamp,
            prev_block.block_hash.clone(),
            item_hash.clone(),
            "test.txt".to_string(),
            test_data.len() as u64,
        );
        
        db.insert_block(&block).unwrap();
        
        // Check if hash exists
        let exists = db.block_exists(&item_hash).unwrap();
        assert!(exists);
    }
}
