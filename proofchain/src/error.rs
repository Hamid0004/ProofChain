#![allow(dead_code)]
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ProofChainError {
    #[error("File not found: {0}")]
    FileNotFound(String),

    #[error("Failed to read file: {0}")]
    FileReadError(String),

    #[error("Hash computation failed: {0}")]
    HashError(String),

    #[error("Database error: {0}")]
    DatabaseError(#[from] rusqlite::Error),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Block verification failed: {0}")]
    BlockVerificationError(String),

    #[error("Merkle tree error: {0}")]
    MerkleTreeError(String),

    #[error("Blockchain anchoring error: {0}")]
    BlockchainError(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Proof not found for file: {0}")]
    ProofNotFound(String),
}

pub type Result<T> = std::result::Result<T, ProofChainError>;
