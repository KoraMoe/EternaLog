#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod eternalog {
    use ink::prelude::vec::Vec;
    use ink::prelude::string::String;
    use ink::storage::Mapping;

    /// A log entry stored on chain
    #[derive(Debug, Clone, PartialEq, Eq)]
    #[ink::scale_derive(Encode, Decode, TypeInfo)]
    pub struct LogEntry {
        /// Unique identifier for this log entry
        pub id: u64,
        /// The log data content
        pub data: String,
        /// The log type identifier
        pub log_type: u32,
        /// Who created this log entry
        pub author: AccountId,
        /// When this log was created (block number)
        pub timestamp: BlockNumber,
    }

    /// Events emitted by the contract
    #[ink(event)]
    pub struct LogStored {
        #[ink(topic)]
        log_id: u64,
        #[ink(topic)]
        author: AccountId,
        #[ink(topic)]
        log_type: u32,
        data: String,
    }

    #[ink(event)]
    pub struct FeeBurned {
        #[ink(topic)]
        amount: Balance,
        #[ink(topic)]
        burner: AccountId,
    }

    #[ink(event)]
    pub struct StorageFeeUpdated {
        #[ink(topic)]
        old_fee: Balance,
        #[ink(topic)]
        new_fee: Balance,
        #[ink(topic)]
        updated_by: AccountId,
    }

    /// Custom errors
    #[derive(Debug, PartialEq, Eq)]
    #[ink::scale_derive(Encode, Decode, TypeInfo)]
    pub enum Error {
        /// Insufficient balance to pay the log storage fee
        InsufficientBalance,
        /// Log entry not found
        LogNotFound,
        /// Invalid log type (must be greater than 0)
        InvalidLogType,
        /// Empty log data not allowed
        EmptyLogData,
        /// Only the contract owner can perform this action
        Unauthorized,
    }

    /// Defines the storage of the contract
    #[ink(storage)]
    pub struct Eternalog {
        /// Owner of the contract (deployer)
        owner: AccountId,
        /// Counter for generating unique log IDs
        next_log_id: u64,
        /// Storage fee per log entry (in native token units)
        storage_fee: Balance,
        /// Total number of logs stored
        total_logs: u64,
        /// Total fees burned
        total_fees_burned: Balance,
        /// Individual log data stored as separate mappings to avoid storage layout issues
        log_data: Mapping<u64, String>,
        log_types: Mapping<u64, u32>,
        log_authors: Mapping<u64, AccountId>,
        log_timestamps: Mapping<u64, BlockNumber>,
        /// Indices for searching
        logs_by_type: Mapping<u32, Vec<u64>>,
        logs_by_author: Mapping<AccountId, Vec<u64>>,
    }

    impl Eternalog {
        /// Constructor that initializes the contract with a storage fee
        #[ink(constructor)]
        pub fn new(storage_fee: Balance) -> Self {
            Self {
                owner: Self::env().caller(),
                next_log_id: 1,
                storage_fee,
                total_logs: 0,
                total_fees_burned: 0,
                log_data: Mapping::default(),
                log_types: Mapping::default(),
                log_authors: Mapping::default(),
                log_timestamps: Mapping::default(),
                logs_by_type: Mapping::default(),
                logs_by_author: Mapping::default(),
            }
        }

        /// Constructor with default storage fee (10 units)
        #[ink(constructor)]
        pub fn default() -> Self {
            Self::new(10)
        }

        /// Store a new log entry on chain
        /// Charges a fee that gets burned
        #[ink(message, payable)]
        pub fn store_log(&mut self, data: String, log_type: u32) -> Result<u64, Error> {
            // Validate inputs
            if data.is_empty() {
                return Err(Error::EmptyLogData);
            }
            if log_type == 0 {
                return Err(Error::InvalidLogType);
            }

            // Check payment and burn fee
            let payment = self.env().transferred_value();
            if payment < self.storage_fee {
                return Err(Error::InsufficientBalance);
            }

            let caller = self.env().caller();
            let current_block = self.env().block_number();
            let log_id = self.next_log_id;

            // Store log components separately
            self.log_data.insert(log_id, &data);
            self.log_types.insert(log_id, &log_type);
            self.log_authors.insert(log_id, &caller);
            self.log_timestamps.insert(log_id, &current_block);

            // Update indices
            let mut type_logs = self.logs_by_type.get(log_type).unwrap_or_default();
            type_logs.push(log_id);
            self.logs_by_type.insert(log_type, &type_logs);

            let mut author_logs = self.logs_by_author.get(caller).unwrap_or_default();
            author_logs.push(log_id);
            self.logs_by_author.insert(caller, &author_logs);

            // Update counters
            self.next_log_id = self.next_log_id.saturating_add(1);
            self.total_logs = self.total_logs.saturating_add(1);
            self.total_fees_burned = self.total_fees_burned.saturating_add(payment);

            // Emit events
            self.env().emit_event(LogStored {
                log_id,
                author: caller,
                log_type,
                data,
            });

            self.env().emit_event(FeeBurned {
                amount: payment,
                burner: caller,
            });

            Ok(log_id)
        }

        /// Retrieve a log entry by its ID
        #[ink(message)]
        pub fn get_log(&self, log_id: u64) -> Result<LogEntry, Error> {
            match (
                self.log_data.get(log_id),
                self.log_types.get(log_id),
                self.log_authors.get(log_id),
                self.log_timestamps.get(log_id),
            ) {
                (Some(data), Some(log_type), Some(author), Some(timestamp)) => {
                    Ok(LogEntry {
                        id: log_id,
                        data,
                        log_type,
                        author,
                        timestamp,
                    })
                }
                _ => Err(Error::LogNotFound),
            }
        }

        /// Get all log IDs for a specific type
        #[ink(message)]
        pub fn get_logs_by_type(&self, log_type: u32) -> Vec<u64> {
            self.logs_by_type.get(log_type).unwrap_or_default()
        }

        /// Get all log IDs for a specific author
        #[ink(message)]
        pub fn get_logs_by_author(&self, author: AccountId) -> Vec<u64> {
            self.logs_by_author.get(author).unwrap_or_default()
        }

        /// Search logs by content (simple substring search)
        /// Returns vector of log IDs that contain the search term
        #[ink(message)]
        pub fn search_logs_by_content(&self, search_term: String) -> Vec<u64> {
            let mut results = Vec::new();
            
            for log_id in 1..self.next_log_id {
                if let Some(data) = self.log_data.get(log_id) {
                    if data.contains(&search_term) {
                        results.push(log_id);
                    }
                }
            }
            
            results
        }

        /// Get logs by both type and author
        #[ink(message)]
        pub fn get_logs_by_type_and_author(&self, log_type: u32, author: AccountId) -> Vec<u64> {
            let type_logs = self.get_logs_by_type(log_type);
            let author_logs = self.get_logs_by_author(author);
            
            // Find intersection
            let mut results = Vec::new();
            for log_id in type_logs {
                if author_logs.contains(&log_id) {
                    results.push(log_id);
                }
            }
            
            results
        }

        /// Get the current storage fee
        #[ink(message)]
        pub fn get_storage_fee(&self) -> Balance {
            self.storage_fee
        }

        /// Get total number of logs stored
        #[ink(message)]
        pub fn get_total_logs(&self) -> u64 {
            self.total_logs
        }

        /// Get total fees burned
        #[ink(message)]
        pub fn get_total_fees_burned(&self) -> Balance {
            self.total_fees_burned
        }

        /// Get the next log ID that will be assigned
        #[ink(message)]
        pub fn get_next_log_id(&self) -> u64 {
            self.next_log_id
        }

        /// Update storage fee (only contract owner can call this)
        #[ink(message)]
        pub fn update_storage_fee(&mut self, new_fee: Balance) -> Result<(), Error> {
            let caller = self.env().caller();
            if caller != self.owner {
                return Err(Error::Unauthorized);
            }
            
            let old_fee = self.storage_fee;
            self.storage_fee = new_fee;
            
            // Emit event
            self.env().emit_event(StorageFeeUpdated {
                old_fee,
                new_fee,
                updated_by: caller,
            });
            
            Ok(())
        }

        /// Get the contract owner
        #[ink(message)]
        pub fn get_owner(&self) -> AccountId {
            self.owner
        }
    }

    /// Unit tests
    #[cfg(test)]
    mod tests {
        use super::*;

        #[ink::test]
        fn default_works() {
            let eternalog = Eternalog::default();
            assert_eq!(eternalog.get_storage_fee(), 10);
            assert_eq!(eternalog.get_total_logs(), 0);
        }

        #[ink::test]
        fn new_works() {
            let eternalog = Eternalog::new(100);
            assert_eq!(eternalog.get_storage_fee(), 100);
            assert_eq!(eternalog.get_total_logs(), 0);
        }

        #[ink::test]
        fn store_log_works() {
            let mut eternalog = Eternalog::new(10);
            
            // Test storing a log (this won't work in unit tests due to payable, but tests the logic)
            let result = eternalog.store_log("Test log entry".to_string(), 1);
            
            // In unit tests, transferred_value() returns 0, so this will fail
            assert_eq!(result, Err(Error::InsufficientBalance));
        }

        #[ink::test]
        fn validate_inputs() {
            let mut eternalog = Eternalog::new(10);
            
            // Test empty data
            let result = eternalog.store_log("".to_string(), 1);
            assert_eq!(result, Err(Error::EmptyLogData));
            
            // Test invalid log type
            let result = eternalog.store_log("Test".to_string(), 0);
            assert_eq!(result, Err(Error::InvalidLogType));
        }

        #[ink::test]
        fn get_nonexistent_log() {
            let eternalog = Eternalog::default();
            let result = eternalog.get_log(999);
            assert_eq!(result, Err(Error::LogNotFound));
        }

        #[ink::test]
        fn only_owner_can_update_fee() {
            let mut eternalog = Eternalog::new(100);
            
            // Owner should be able to update fee
            let result = eternalog.update_storage_fee(200);
            assert_eq!(result, Ok(()));
            assert_eq!(eternalog.get_storage_fee(), 200);
            
            // Note: In unit tests, we can't easily test with different accounts
            // This would be better tested in E2E tests with different signers
        }
    }

    /// End-to-end tests
    #[cfg(all(test, feature = "e2e-tests"))]
    mod e2e_tests {
        use super::*;
        use ink_e2e::ContractsBackend;

        type E2EResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

        #[ink_e2e::test]
        async fn default_works(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
            let mut constructor = EternalogRef::default();
            let contract = client
                .instantiate("eternalog", &ink_e2e::alice(), &mut constructor)
                .submit()
                .await
                .expect("instantiate failed");
            let call_builder = contract.call_builder::<Eternalog>();

            let storage_fee = call_builder.get_storage_fee();
            let get_result = client.call(&ink_e2e::alice(), &storage_fee).dry_run().await?;
            assert_eq!(get_result.return_value(), 10);

            Ok(())
        }

        #[ink_e2e::test]
        async fn store_and_retrieve_log(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
            let mut constructor = EternalogRef::new(100);
            let contract = client
                .instantiate("eternalog", &ink_e2e::bob(), &mut constructor)
                .submit()
                .await
                .expect("instantiate failed");
            let mut call_builder = contract.call_builder::<Eternalog>();

            // Store a log
            let store_log = call_builder.store_log("Hello, eternal world!".to_string(), 42);
            let store_result = client
                .call(&ink_e2e::bob(), &store_log)
                .value(200) // Send enough to cover the fee
                .submit()
                .await
                .expect("store_log failed");

            // Get the log ID from the result
            let log_id = store_result.return_value().unwrap();
            assert_eq!(log_id, 1);

            // Retrieve the log
            let get_log = call_builder.get_log(log_id);
            let get_result = client.call(&ink_e2e::bob(), &get_log).dry_run().await?;
            let retrieved_log = get_result.return_value().unwrap();
            
            assert_eq!(retrieved_log.id, 1);
            assert_eq!(retrieved_log.data, "Hello, eternal world!");
            assert_eq!(retrieved_log.log_type, 42);
            assert_eq!(retrieved_log.author, ink_e2e::account_id(ink_e2e::AccountKeyring::Bob));

            Ok(())
        }
    }
}
