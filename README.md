# Eternalog 📜

A decentralized, immutable logging system built with [ink!](https://use.ink/) smart contracts for Substrate-based blockchains.

## Overview

Eternalog is an on-chain logging contract that allows users to store permanent, immutable log entries on the blockchain. Each log entry is stored forever and cannot be deleted, making it perfect for audit trails, important records, and transparent logging systems.

## ✨ Features

- **🔒 Immutable Storage**: Once logged, entries cannot be deleted or modified
- **💰 Fee-based System**: Configurable storage fees that get burned (removed from circulation)
- **🔍 Advanced Search**: Search logs by content, type, author, or combinations
- **📊 Rich Metadata**: Each log includes timestamp, author, and categorization
- **👑 Owner Controls**: Contract deployer can adjust storage fees
- **🎯 Event Emissions**: Real-time events for log storage and fee updates

## 🏗️ Contract Structure

### Log Entry Structure
```rust
pub struct LogEntry {
    pub id: u64,              // Unique identifier
    pub data: String,         // Log content
    pub log_type: u32,        // Category/type identifier
    pub author: AccountId,    // Who created the log
    pub timestamp: BlockNumber, // When it was created
}
```

### Key Features
- **Storage Fee**: Configurable fee per log entry (burned on storage)
- **Access Control**: Only contract owner can update fees
- **Efficient Indexing**: Logs indexed by type and author for fast retrieval
- **Content Search**: Full-text search capabilities

## 🚀 Getting Started

### Prerequisites

1. Install Rust and Cargo:
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

2. Install ink! dependencies:
```bash
rustup component add rust-src
rustup target add wasm32-unknown-unknown
```

3. Install cargo-contract:
```bash
cargo install --force --locked cargo-contract
```

### Building the Contract

```bash
# Clone the repository
git clone <your-repo-url>
cd eternalog

# Build the contract
cargo contract build --release

# Run tests
cargo test
```

## 📖 API Reference

### Constructor Functions

#### `new(storage_fee: Balance)`
Creates a new contract instance with a specified storage fee.
- `storage_fee`: Fee required per log entry (in native tokens)

#### `default()`
Creates a new contract instance with default storage fee (10 units).

### Core Functions

#### `store_log(data: String, log_type: u32) -> Result<u64, Error>` 💰
Stores a new log entry on-chain. **Requires payment!**
- `data`: The log content (cannot be empty)
- `log_type`: Category identifier (must be > 0)
- **Returns**: Log ID if successful
- **Payment**: Must send at least the storage fee amount

#### `get_log(log_id: u64) -> Result<LogEntry, Error>`
Retrieves a specific log entry by its ID.

### Search Functions

#### `get_logs_by_type(log_type: u32) -> Vec<u64>`
Returns all log IDs of a specific type.

#### `get_logs_by_author(author: AccountId) -> Vec<u64>`
Returns all log IDs created by a specific author.

#### `search_logs_by_content(search_term: String) -> Vec<u64>`
Returns log IDs containing the search term in their content.

#### `get_logs_by_type_and_author(log_type: u32, author: AccountId) -> Vec<u64>`
Returns log IDs matching both type and author criteria.

### Information Functions

#### `get_storage_fee() -> Balance`
Returns the current storage fee per log entry.

#### `get_total_logs() -> u64`
Returns the total number of logs stored.

#### `get_total_fees_burned() -> Balance`
Returns the total amount of fees burned.

#### `get_next_log_id() -> u64`
Returns the next log ID that will be assigned.

#### `get_owner() -> AccountId`
Returns the contract owner's account ID.

### Owner Functions

#### `update_storage_fee(new_fee: Balance) -> Result<(), Error>` 👑
Updates the storage fee. **Owner only!**
- `new_fee`: New fee amount per log entry

## 🎯 Events

### `LogStored`
Emitted when a new log is stored.
```rust
{
    log_id: u64,        // ID of the stored log
    author: AccountId,  // Who stored it
    log_type: u32,      // Log type
    data: String,       // Log content
}
```

### `FeeBurned`
Emitted when storage fees are burned.
```rust
{
    amount: Balance,    // Amount burned
    burner: AccountId,  // Who paid the fee
}
```

### `StorageFeeUpdated`
Emitted when the owner updates the storage fee.
```rust
{
    old_fee: Balance,     // Previous fee
    new_fee: Balance,     // New fee
    updated_by: AccountId, // Who updated it (owner)
}
```

## 💡 Usage Examples

### Storing a Log Entry

```rust
// In your dApp frontend (using polkadot-js or similar)
const logData = "User login successful";
const logType = 1; // Authentication logs
const storageFee = 100; // Current fee

await contract.tx.storeLog(
    { value: storageFee }, // Payment
    logData,
    logType
);
```

### Searching Logs

```javascript
// Get all authentication logs (type 1)
const authLogs = await contract.query.getLogsByType(1);

// Search for specific content
const errorLogs = await contract.query.searchLogsByContent("error");

// Get logs by specific user
const userLogs = await contract.query.getLogsByAuthor(userAddress);
```

### Administrative Tasks

```javascript
// Only contract owner can do this
await contract.tx.updateStorageFee({ value: 0 }, 200);
```

## 🧪 Testing

The contract includes comprehensive tests:

```bash
# Run unit tests
cargo test

# Run with end-to-end tests (requires substrate-contracts-node)
cargo test --features e2e-tests
```

### Test Coverage
- ✅ Contract initialization
- ✅ Log storage validation
- ✅ Access control for fee updates
- ✅ Error handling
- ✅ Search functionality

## 🔧 Configuration

### Log Types
Define your own log type conventions:
```rust
const LOG_TYPES = {
    AUTHENTICATION: 1,
    TRANSACTION: 2,
    ERROR: 3,
    AUDIT: 4,
    SYSTEM: 5,
};
```

### Storage Fee Strategy
Consider these factors when setting storage fees:
- **Network costs**: Higher fees on expensive networks
- **Usage patterns**: Lower fees for high-frequency logging
- **Economic model**: Deflationary (fees burned) vs. operational costs

## 🚨 Error Handling

The contract defines several error types:

- `InsufficientBalance`: Payment below required storage fee
- `LogNotFound`: Requested log ID doesn't exist
- `InvalidLogType`: Log type must be greater than 0
- `EmptyLogData`: Log content cannot be empty
- `Unauthorized`: Only contract owner can perform this action

## 🔐 Security Considerations

1. **Immutability**: Logs cannot be deleted - consider data privacy implications
2. **Fee Burning**: Fees are permanently removed from circulation
3. **Access Control**: Only deployer can modify contract parameters
4. **Input Validation**: All inputs are validated before processing
5. **Overflow Protection**: Uses saturating arithmetic to prevent overflows

## 🛠️ Development

### Project Structure
```
eternalog/
├── lib.rs          # Main contract code
├── Cargo.toml      # Dependencies and metadata
├── README.md       # This file
└── target/         # Build artifacts
```

### Dependencies
- `ink! 5.1.1`: Smart contract framework
- `scale-info 2.11`: Type information for encoding/decoding

## 📋 Roadmap

- [ ] Add batch log storage functionality
- [ ] Implement log expiration (with user consent)
- [ ] Add log categories and permissions
- [ ] Create web-based log viewer
- [ ] Add integration with common logging libraries

## 🤝 Contributing

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## 📄 License

This project is licensed under the MIT License - see the LICENSE file for details.

## 🙏 Acknowledgments

- [ink! Team](https://use.ink/) for the excellent smart contract framework
- [Substrate](https://substrate.io/) for the blockchain framework
- [Polkadot](https://polkadot.network/) ecosystem for the infrastructure

---

**⚠️ Disclaimer**: This is experimental software. Use at your own risk. Always test thoroughly before deploying to production networks. 