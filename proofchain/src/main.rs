mod cli;
mod error;
mod ledger;

use clap::Parser;
use cli::{Cli, Commands};
use error::Result;
use ledger::{Database, hash_file, hash_combine, hash_string, Block};
use std::path::Path;
use std::fs;
use chrono::Utc;

fn main() {
    let cli = Cli::parse();

    if let Err(e) = run(cli) {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

fn run(cli: Cli) -> Result<()> {
    match cli.command {
        Commands::Add { file, description } => cmd_add(&cli.db, &file, description),
        Commands::Verify { file } => cmd_verify(&cli.db, &file),
        Commands::List { limit, offset } => cmd_list(&cli.db, limit, offset),
        Commands::Check => cmd_check(&cli.db),
        Commands::Merkle { batch_size } => cmd_merkle(&cli.db, batch_size),
        Commands::Anchor { tree_id, rpc_url } => cmd_anchor(&cli.db, tree_id, rpc_url),
        Commands::Status => cmd_status(&cli.db),
        Commands::Export { file, output } => cmd_export(&cli.db, Some(file.as_str()), output),
        Commands::Init => cmd_init(&cli.db),
        Commands::Config { key, value, show_all } => cmd_config(&cli.db, key, value, show_all),
    }
}

/// Register a new file by computing its hash and storing it in the ledger
fn cmd_add(db_path: &str, file_path: &str, _description: Option<String>) -> Result<()> {
    // Check if file exists
    if !Path::new(file_path).exists() {
        return Err(crate::error::ProofChainError::FileNotFound(file_path.to_string()));
    }

    // Get file metadata
    let metadata = fs::metadata(file_path)?;
    let file_size = metadata.len();
    let file_name = Path::new(file_path)
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();

    // Compute file hash
    let item_hash = hash_file(file_path)?;

    // Open database
    let db = Database::new(db_path)?;
    db.init()?;

    // Check if proof already exists
    if db.block_exists(&item_hash)? {
        println!("✓ File already registered!");
        println!("  Hash: {}", item_hash);
        println!("  Name: {}", file_name);
        println!("  Size: {} bytes", file_size);
        return Ok(());
    }

    // Get current block height and previous block
    let prev_block = db.get_latest_block()?.expect("Genesis block should exist");
    let timestamp = Utc::now().timestamp();
    
    // Create new block
    let new_block = Block::new(
        prev_block.height + 1,
        timestamp,
        prev_block.block_hash.clone(),
        item_hash.clone(),
        file_name.clone(),
        file_size,
    );
    
    db.insert_block(&new_block)?;

    println!("✓ File registered successfully!");
    println!("  Hash: {}", item_hash);
    println!("  Name: {}", file_name);
    println!("  Size: {} bytes", file_size);
    println!("  Timestamp: {}", timestamp);
    println!("  Block Height: {}", new_block.height);

    Ok(())
}

/// Verify if a file was previously registered
fn cmd_verify(db_path: &str, file_path: &str) -> Result<()> {
    // Check if file exists
    if !Path::new(file_path).exists() {
        return Err(crate::error::ProofChainError::FileNotFound(file_path.to_string()));
    }

    // Compute file hash
    let item_hash = hash_file(file_path)?;

    // Open database
    let db = Database::new(db_path)?;
    db.init()?;

    // Check if proof exists
    match db.get_block_by_hash(&item_hash)? {
        Some(block) => {
            println!("✓ File verified! This file was registered at:");
            println!("  Hash: {}", block.item_hash);
            println!("  Name: {}", block.file_name);
            println!("  Size: {} bytes", block.file_size);
            println!("  Timestamp: {}", block.timestamp);
            println!("  Block Height: {}", block.height);
            if let Some(tx) = &block.anchor_tx_hash {
                println!("  Anchored TX: {}", tx);
            }
        }
        None => {
            println!("✗ File not found in registry.");
            println!("  Hash: {}", item_hash);
            println!("This file has not been registered before.");
        }
    }

    Ok(())
}

/// List all registered proofs
fn cmd_list(db_path: &str, limit: usize, offset: usize) -> Result<()> {
    let db = Database::new(db_path)?;
    db.init()?;

    let blocks = db.list_blocks(limit, offset)?;
    let total = db.get_block_count()?;

    if blocks.is_empty() {
        println!("No files registered yet.");
        return Ok(());
    }

    println!("Registered Files (showing {} of {}):", blocks.len(), total);
    println!("{:<6} {:<20} {:<15} {:<12} {}", "Height", "Name", "Size", "Timestamp", "Hash");
    println!("{}", "-".repeat(80));

    for block in blocks {
        let ts = chrono::DateTime::from_timestamp(block.timestamp, 0)
            .map(|dt| dt.format("%Y-%m-%d %H:%M").to_string())
            .unwrap_or_else(|| "Unknown".to_string());
        
        // Handle genesis block which has empty item_hash
        let hash_display = if block.item_hash.is_empty() {
            "genesis".to_string()
        } else {
            truncate(&block.item_hash, 16)
        };
        
        println!(
            "{:<6} {:<20} {:<15} {:<12} {}",
            block.height,
            truncate(&block.file_name, 20),
            block.file_size,
            ts,
            hash_display
        );
    }

    Ok(())
}

/// Check the integrity of the blockchain
fn cmd_check(db_path: &str) -> Result<()> {
    let db = Database::new(db_path)?;
    db.init()?;

    let (count, is_valid) = db.get_stats()?;

    println!("Blockchain Status:");
    println!("  Total Blocks: {}", count);
    println!("  Integrity: {}", if is_valid { "✓ VALID" } else { "✗ INVALID" });

    if !is_valid {
        eprintln!("WARNING: The blockchain integrity check failed!");
        eprintln!("This may indicate tampering or corruption.");
    }

    Ok(())
}

/// Build a Merkle root from leaf hashes (duplicates last leaf if odd)
fn build_merkle_root(leaves: &[String]) -> String {
    if leaves.is_empty() {
        return String::new();
    }
    let mut level = leaves.to_vec();
    while level.len() > 1 {
        if level.len() % 2 != 0 {
            let last = level.last().unwrap().clone();
            level.push(last);
        }
        let mut next = Vec::new();
        for i in (0..level.len()).step_by(2) {
            next.push(hash_combine(&level[i], &level[i + 1]));
        }
        level = next;
    }
    level[0].clone()
}

/// Create Merkle tree from unbatched blocks
fn cmd_merkle(db_path: &str, batch_size: usize) -> Result<()> {
    let db = Database::new(db_path)?;
    db.init()?;

    let blocks = db.get_unbatched_blocks(batch_size)?;
    if blocks.is_empty() {
        println!("No unbatched blocks available.");
        println!("Add files first using 'proofchain add <file>'");
        return Ok(());
    }

    let leaf_hashes: Vec<String> = blocks.iter().map(|b| b.item_hash.clone()).collect();
    let root = build_merkle_root(&leaf_hashes);
    let batch_id = db.create_merkle_batch(&root, leaf_hashes.len())?;
    let block_ids: Vec<i64> = blocks.iter().filter_map(|b| b.id).collect();
    db.update_blocks_with_merkle_root(&block_ids, &root)?;

    println!("Merkle tree created successfully!");
    println!("  Batch ID: {}", batch_id);
    println!("  Merkle Root: {}", root);
    println!("  Leaf Count: {}", leaf_hashes.len());
    Ok(())
}

/// Anchor Merkle root to blockchain (mock mode by default)
fn cmd_anchor(db_path: &str, _tree_id: Option<u64>, rpc_url: Option<String>) -> Result<()> {
    let db = Database::new(db_path)?;
    db.init()?;

    let (tree_id, root_hash) = match db.get_unanchored_merkle_tree()? {
        Some(t) => t,
        None => {
            println!("No unanchored Merkle trees found.");
            println!("Run 'proofchain merkle' first to create a batch.");
            return Ok(());
        }
    };

    let (network, is_mock) = match &rpc_url {
        Some(url) => {
            println!("  RPC URL: {}", url);
            ("sepolia".to_string(), false)
        }
        None => ("mock".to_string(), true),
    };

    let ts = Utc::now().timestamp();
    let tx_hash = format!("0x{}", hash_string(&format!("{}{}", root_hash, ts)));
    db.create_anchor(tree_id, &tx_hash, &network)?;

    println!("Merkle root anchored successfully!");
    println!("  Tree ID: {}", tree_id);
    println!("  Root Hash: {}", root_hash);
    println!("  Transaction Hash: {}", tx_hash);
    if is_mock {
        println!("  Status: MOCK (not on real blockchain)");
    } else {
        println!("  Status: submitted (simulated tx - real client not wired)");
    }
    Ok(())
}

/// Show system status
fn cmd_status(db_path: &str) -> Result<()> {
    let db = Database::new(db_path)?;
    db.init()?;

    let (count, is_valid) = db.get_stats()?;
    let height = db.get_current_height()?;

    println!("ProofChain Status:");
    println!("  Database: {}", db_path);
    println!("  Block Height: {}", height);
    println!("  Total Records: {}", count);
    println!("  Chain Valid: {}", is_valid);

    let (total_trees, anchored_trees, latest_tx) = db.get_anchor_stats()?;
    println!("  Merkle Trees: {} total, {} anchored", total_trees, anchored_trees);
    match latest_tx {
        Some(tx) => {
            println!("  Latest Anchor TX: {}", truncate(&tx, 20));
            println!("  Network: Sepolia Testnet");
        }
        None => println!("  Network: Not connected (run 'proofchain anchor')"),
    }

    Ok(())
}

/// Export proof data as JSON
fn cmd_export(db_path: &str, file: Option<&str>, output: Option<String>) -> Result<()> {
    let db = Database::new(db_path)?;
    db.init()?;

    let file_path = match file {
        Some(f) => f,
        None => {
            println!("Please specify a file: proofchain export <file>");
            return Ok(());
        }
    };

    let item_hash = hash_file(file_path)?;
    match db.get_block_by_hash(&item_hash)? {
        Some(block) => {
            let merkle = match &block.merkle_root {
                Some(r) => format!("\"{}\"", r),
                None => "null".to_string(),
            };
            let anchor = match &block.anchor_tx_hash {
                Some(t) => format!("\"{}\"", t),
                None => "null".to_string(),
            };
            let json = format!(
                "{{\n  \"file_name\": \"{}\",\n  \"file_size\": {},\n  \"item_hash\": \"{}\",\n  \"block_height\": {},\n  \"block_hash\": \"{}\",\n  \"timestamp\": {},\n  \"merkle_root\": {},\n  \"anchor_tx_hash\": {}\n}}",
                block.file_name, block.file_size, block.item_hash, block.height,
                block.block_hash, block.timestamp, merkle, anchor
            );
            match output {
                Some(path) => {
                    fs::write(&path, &json)?;
                    println!("Proof exported to: {}", path);
                }
                None => println!("{}", json),
            }
        }
        None => println!("File not found in registry. Hash: {}", item_hash),
    }
    Ok(())
}

/// Initialize the database
fn cmd_init(db_path: &str) -> Result<()> {
    let db = Database::new(db_path)?;
    db.init()?;

    println!("✓ Database initialized successfully!");
    println!("  Location: {}", db_path);
    println!("  Genesis block created.");

    Ok(())
}

/// Configuration management (placeholder)
fn cmd_config(_db_path: &str, _key: Option<String>, _value: Option<String>, _show_all: bool) -> Result<()> {
    println!("Configuration management - Coming soon");
    Ok(())
}

/// Helper function to truncate strings
fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len.saturating_sub(3)])
    }
}
