# ProofChain

**A Rust-based proof-of-existence notary system with blockchain anchoring**

![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)
![Rust](https://img.shields.io/badge/Rust-1.70+-orange.svg)

## Overview

ProofChain is a blockchain-based document notarization system that allows users to prove that a file existed at a specific time without storing the actual file content. The application hashes files using SHA-256, stores the hash in a local tamper-evident ledger, groups multiple hashes into Merkle trees, and anchors the Merkle root to the Ethereum Sepolia testnet.

## Features

- 🔐 **File Hashing**: SHA-256 cryptographic hashing of any file type
- ⛓️ **Local Blockchain**: Tamper-evident ledger with hash chaining
- 🌳 **Merkle Trees**: Efficient batching of multiple proofs
- 🔗 **Blockchain Anchoring**: Optional anchoring to Ethereum Sepolia testnet
- ✅ **Verification**: Prove file existence without revealing content
- 💻 **CLI Interface**: Easy-to-use command-line interface
- 📊 **Export**: Generate proof certificates in JSON format

## Installation

### Prerequisites

- Rust 1.70 or higher ([install](https://rustup.rs/))
- SQLite3 (for database storage)
- (Optional) Ethereum wallet for blockchain anchoring

### Build from Source

```bash
git clone https://github.com/yourusername/proofchain.git
cd proofchain
cargo build --release
```

The binary will be available at `target/release/proofchain`.

### Quick Start

```bash
# Initialize the database
./proofchain init

# Register a file
./proofchain register document.pdf

# Verify a file
./proofchain verify document.pdf

# List all registered proofs
./proofchain list

# Build Merkle tree from pending proofs
./proofchain merkle

# Anchor to blockchain (requires configuration)
./proofchain anchor
```

## Commands

| Command | Description |
|---------|-------------|
| `init` | Initialize the database and configuration |
| `register` | Register a file's hash in the ledger |
| `verify` | Verify if a file was previously registered |
| `list` | List registered proofs with optional filters |
| `merkle` | Build Merkle tree from pending proofs |
| `anchor` | Anchor Merkle root to blockchain |
| `status` | Check anchoring status |
| `export` | Export proof certificate |
| `config` | Manage configuration settings |

## Usage Examples

### Register a File

```bash
# Basic registration
proofchain register myfile.pdf

# With custom name
proofchain register -n "Contract v1" myfile.pdf

# With metadata
proofchain register -m '{"author":"John","type":"contract"}' document.pdf

# Quiet mode (output only hash)
proofchain register -q largefile.zip
```

### Verify a File

```bash
# Verify by file
proofchain verify myfile.pdf

# Verify by hash
proofchain verify -H a3f2b8c9d4e5f6a7b8c9d0e1f2a3b4c5...

# Full details
proofchain verify -f myfile.pdf

# JSON output
proofchain verify -j myfile.pdf
```

### List Proofs

```bash
# List all proofs
proofchain list

# Filter by status
proofchain list -s anchored

# Date range
proofchain list -f 2024-01-01 -t 2024-01-31

# JSON output
proofchain list -j --limit 100
```

### Merkle Tree Operations

```bash
# Build Merkle tree from pending proofs
proofchain merkle

# Verbose output
proofchain merkle -v

# Dry run (simulate)
proofchain merkle --dry-run
```

### Blockchain Anchoring

```bash
# Anchor latest Merkle root
proofchain anchor -l

# Anchor specific root
proofchain anchor -r <merkle_root>

# Dry run
proofchain anchor --dry-run
```

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                     CLI Interface                            │
├─────────────────────────────────────────────────────────────┤
│  Application Layer                                           │
│  • File Processor  • Block Manager                          │
│  • Merkle Builder  • Blockchain Anchor                      │
├─────────────────────────────────────────────────────────────┤
│  Core Logic                                                  │
│  • SHA-256 Hashing  • Hash Chain Validation                 │
│  • Merkle Tree      • Transaction Building                  │
├─────────────────────────────────────────────────────────────┤
│  Data Layer                                                  │
│  • SQLite Ledger    • Ethereum Sepolia Testnet              │
└─────────────────────────────────────────────────────────────┘
```

## Database Schema

ProofChain uses SQLite with the following tables:

- **blocks**: Blockchain block headers
- **proofs**: Individual file proof records
- **merkle_trees**: Merkle tree data
- **anchors**: Blockchain anchoring records
- **config**: Application configuration

See `database_schema.sql` for complete schema documentation.

## Development

### Project Structure

```
proofchain/
├── src/
│   ├── main.rs          # CLI entry point
│   ├── cli/             # CLI commands
│   ├── blockchain/      # Blockchain components
│   ├── merkle/          # Merkle tree implementation
│   ├── storage/         # Database layer
│   ├── crypto/          # Cryptographic utilities
│   └── models/          # Data models
├── tests/               # Integration tests
├── contracts/           # Solidity smart contracts
└── docs/                # Documentation
```

### Running Tests

```bash
# Run all tests
cargo test

# Run with coverage (requires cargo-tarpaulin)
cargo tarpaulin

# Run specific test
cargo test test_merkle_tree
```

### Building Documentation

```bash
cargo doc --open
```

## Configuration

Configuration is stored in SQLite and can be managed via CLI:

```bash
# List all config
proofchain config --list

# Set a value
proofchain config network sepolia

# Reset to defaults
proofchain config --reset
```

### Environment Variables

| Variable | Description |
|----------|-------------|
| `PROOFCHAIN_DATABASE_PATH` | Custom database location |
| `PROOFCHAIN_CONFIG_PATH` | Custom config file path |
| `PROOFCHAIN_PRIVATE_KEY` | Ethereum private key (for anchoring) |
| `RUST_LOG` | Log level (debug, info, warn, error) |

## Security Considerations

- **Private Keys**: Never store private keys in code or database. Use environment variables.
- **Hash Verification**: Always verify hash chain integrity before trusting proofs.
- **Database Backups**: Regularly backup your SQLite database.
- **Testnet Only**: Blockchain anchoring uses Sepolia testnet (not production).

## Limitations

- Local ledger (not distributed blockchain)
- No mining or consensus algorithm
- No cryptocurrency token
- Testnet anchoring only (not mainnet)

## Future Enhancements

- REST API for web integration
- Multiple blockchain support (Polygon, Bitcoin)
- GUI application
- IPFS integration for encrypted file storage
- Multi-signature anchoring
- PDF certificate generation

## Troubleshooting

### Database Errors

```bash
# Reset database (backup first!)
proofchain init --force
```

### Blockchain Connection Issues

```bash
# Check configuration
proofchain config --list

# Verify RPC endpoint accessibility
curl https://sepolia.infura.io/v3/YOUR_KEY
```

## Contributing

Contributions are welcome! Please see our contributing guidelines for details.

1. Fork the repository
2. Create a feature branch
3. Commit your changes
4. Push to the branch
5. Open a Pull Request

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## References

- [Bitcoin Whitepaper](https://bitcoin.org/bitcoin.pdf)
- [Ethereum Documentation](https://ethereum.org/en/developers/docs/)
- [The Rust Programming Language](https://doc.rust-lang.org/book/)
- [Merkle Trees](https://en.wikipedia.org/wiki/Merkle_tree)

## Support

For issues and questions:
- GitHub Issues: https://github.com/yourusername/proofchain/issues
- Documentation: See `docs/` directory

---

**Version**: 0.1.0  
**Last Updated**: 2024  
**Status**: Phase 1 Complete - Planning & Design
