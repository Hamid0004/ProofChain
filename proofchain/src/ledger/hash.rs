#![allow(dead_code)]
use sha2::{Sha256, Digest};
use hex;
use std::fs::File;
use std::io::{Read, BufReader};
use crate::error::Result;

/// Hash a file using SHA-256
pub fn hash_file(file_path: &str) -> Result<String> {
    let file = File::open(file_path)?;
    let mut reader = BufReader::new(file);
    let mut hasher = Sha256::new();
    
    let mut buffer = [0u8; 8192];
    loop {
        let count = reader.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        hasher.update(&buffer[..count]);
    }
    
    let result = hasher.finalize();
    Ok(hex::encode(result))
}

/// Hash a string
pub fn hash_string(data: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data.as_bytes());
    let result = hasher.finalize();
    hex::encode(result)
}

/// Hash bytes directly
pub fn hash_bytes(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    let result = hasher.finalize();
    hex::encode(result)
}

/// Combine two hashes (for Merkle tree)
pub fn hash_combine(left: &str, right: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(hex::decode(left).unwrap_or_default());
    hasher.update(hex::decode(right).unwrap_or_default());
    let result = hasher.finalize();
    hex::encode(result)
}
