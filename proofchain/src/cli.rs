use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "proofchain")]
#[command(author = "ProofChain Development Team")]
#[command(version = "0.1.0")]
#[command(about = "A Rust-based proof-of-existence notary system with blockchain anchoring", long_about = None)]
pub struct Cli {
    /// Path to the SQLite database file
    #[arg(global = true, short, long, default_value = "proofchain.db")]
    pub db: String,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Register a new file by computing its hash and storing it in the ledger
    Add {
        /// Path to the file to register
        #[arg(required = true)]
        file: String,

        /// Optional description or metadata for the file
        #[arg(short = 'D', long)]
        description: Option<String>,
    },

    /// Verify if a file was previously registered by recomputing its hash
    Verify {
        /// Path to the file to verify
        #[arg(required = true)]
        file: String,
    },

    /// List all registered files and their hashes
    List {
        /// Limit the number of results (default: 20)
        #[arg(short, long, default_value = "20")]
        limit: usize,

        /// Offset for pagination
        #[arg(short, long, default_value = "0")]
        offset: usize,
    },

    /// Check the integrity of the entire ledger (hash chain verification)
    Check,

    /// Create a Merkle tree from pending proofs and compute the root
    Merkle {
        /// Batch size for creating Merkle trees (default: 16)
        #[arg(short, long, default_value = "16")]
        batch_size: usize,
    },

    /// Anchor a Merkle root to the blockchain testnet
    Anchor {
        /// ID of the Merkle tree to anchor
        #[arg(short, long)]
        tree_id: Option<u64>,

        /// Ethereum Sepolia RPC URL (optional, can be set via config)
        #[arg(long)]
        rpc_url: Option<String>,
    },

    /// Show system status and statistics
    Status,

    /// Export proof data for a specific file as JSON
    Export {
        /// Path to the file to export proof for
        #[arg(required = true)]
        file: String,

        /// Output file path (default: stdout)
        #[arg(short, long)]
        output: Option<String>,
    },

    /// Initialize a new database with genesis block
    Init,

    /// Configure system settings
    Config {
        /// Configuration key to set
        #[arg(short, long)]
        key: Option<String>,

        /// Configuration value to set
        #[arg(short, long)]
        value: Option<String>,

        /// Show all configuration values
        #[arg(long)]
        show_all: bool,
    },
}
