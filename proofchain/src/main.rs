mod cli;
mod error;
mod ledger;

use clap::Parser;
use cli::{Cli, Commands};
use error::Result;
use ledger::{Database, hash_file, Block};
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

/// Create Merkle tree from unanchored blocks (placeholder for Phase 4)
fn cmd_merkle(_db_path: &str, batch_size: usize) -> Result<()> {
    println!("Merkle tree batching - Coming in Phase 4");
    println!("  Batch size: {}", batch_size);
    Ok(())
}

/// Anchor Merkle root to blockchain (placeholder for Phase 5)
fn cmd_anchor(_db_path: &str, _tree_id: Option<u64>, _rpc_url: Option<String>) -> Result<()> {
    println!("Blockchain anchoring - Coming in Phase 5");
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
    println!("  Network: Sepolia Testnet (not yet connected)");

    Ok(())
}

/// Export proof data (placeholder)
fn cmd_export(_db_path: &str, _file: Option<&str>, _output: Option<String>) -> Result<()> {
    println!("Export functionality - Coming soon");
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
