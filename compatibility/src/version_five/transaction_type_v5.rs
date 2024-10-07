use diem_crypto::{ed25519::{Ed25519Signature, Ed25519PublicKey, PublicKey, Signature}, multi_ed25519::{MultiEd25519PublicKey, MultiEd25519Signature}};


use diem_types::{chain_id::ChainId, transaction::authenticator::AccountAuthenticator};
use serde::{Deserialize, Serialize};

use super::legacy_address_v5::LegacyAddressV5;

// #[allow(clippy::large_enum_variant)]
#[derive(Clone, Debug, Serialize, Deserialize,)]
pub enum TransactionV5 {
    /// Transaction submitted by the user. e.g: P2P payment transaction, publishing module
    /// transaction, etc.
    /// TODO: We need to rename SignedTransaction to SignedUserTransaction, as well as all the other
    ///       transaction types we had in our codebase.
    UserTransaction(SignedTransaction),

    /// Transaction that applies a WriteSet to the current storage, it's applied manually via db-bootstrapper.
    // GenesisTransaction(WriteSetPayload),
    GenesisTransaction(Vec<u8>),

    /// Transaction to update the block metadata resource at the beginning of a block.
    BlockMetadata(Vec<u8>),
}


#[derive(Clone, Debug, Serialize, Deserialize)]
struct SignedTransaction {
    /// The raw transaction
    raw_txn: RawTransaction,

    /// Public key and signature to authenticate
    // authenticator: Vec<u8>,
    authenticator: TransactionAuthenticator,
}


// /// Two different kinds of WriteSet transactions.
// #[derive(Clone, Debug, Serialize, Deserialize)]
// enum WriteSetPayload {
//     /// Directly passing in the WriteSet.
//     Direct(ChangeSet),
//     /// Generate the WriteSet by running a script.
//     Script {
//         /// Execute the script as the designated signer.
//         execute_as: AccountAddress,
//         /// Script body that gets executed.
//         script: Script,
//     },
// }


// #[derive(Clone, Debug, Serialize, Deserialize)]
// struct BlockMetadata {
//     id: HashValue,
//     round: u64,
//     timestamp_usecs: u64,
//     // The vector has to be sorted to ensure consistent result among all nodes
//     previous_block_votes: Vec<AccountAddress>,
//     proposer: AccountAddress,
// }


// #[derive(Clone, Debug, Serialize, Deserialize)]
// struct TransactionInfo {
//     /// The hash of this transaction.
//     transaction_hash: HashValue,

//     /// The root hash of Sparse Merkle Tree describing the world state at the end of this
//     /// transaction.
//     state_root_hash: HashValue,

//     /// The root hash of Merkle Accumulator storing all events emitted during this transaction.
//     event_root_hash: HashValue,

//     /// The amount of gas used.
//     gas_used: u64,

//     /// The vm status. If it is not `Executed`, this will provide the general error class. Execution
//     /// failures and Move abort's recieve more detailed information. But other errors are generally
//     /// categorized with no status code or other information
//     status: KeptVMStatus,
// }


// #[derive(Clone, Serialize, Deserialize)]
// enum KeptVMStatus {
//     Executed,
//     OutOfGas,
//     MoveAbort(AbortLocation, /* code */ u64),
//     ExecutionFailure {
//         location: AbortLocation,
//         function: u16,
//         code_offset: u16,
//     },
//     MiscellaneousError,
// }

#[derive(Clone, Debug, Serialize, Deserialize)]

struct RawTransaction {
    /// Sender's address.
    sender: LegacyAddressV5,

    /// Sequence number of this transaction. This must match the sequence number
    /// stored in the sender's account at the time the transaction executes.
    sequence_number: u64,

    /// The transaction payload, e.g., a script to execute.
    payload: TransactionPayload,

    /// Maximal total gas to spend for this transaction.
    max_gas_amount: u64,

    /// Price to be paid per gas unit.
    gas_unit_price: u64,

    /// The currency code, e.g., "XUS", used to pay for gas. The `max_gas_amount`
    /// and `gas_unit_price` values refer to units of this currency.
    gas_currency_code: String,

    /// Expiration timestamp for this transaction, represented
    /// as seconds from the Unix Epoch. If the current blockchain timestamp
    /// is greater than or equal to this time, then the transaction has
    /// expired and will be discarded. This can be set to a large value far
    /// in the future to indicate that a transaction does not expire.
    expiration_timestamp_secs: u64,

    /// Chain ID of the Diem network this transaction is intended for.
    chain_id: ChainId,
}


/// Different kinds of transactions.
#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub enum TransactionPayload {
    /// A system maintenance transaction.
    // WriteSet(WriteSetPayload),
    WriteSet(Vec<u8>),
    /// A transaction that executes code.
    // Script(Script),
    Script(Vec<u8>),
    /// A transaction that publishes code.
    // Module(Module),
    Module(Vec<u8>),
    /// A transaction that executes an existing script function published on-chain.
    // ScriptFunction(ScriptFunction),
    ScriptFunction(Vec<u8>),

}


#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum TransactionAuthenticator {
    /// Single signature
    Ed25519 {
        public_key: Ed25519PublicKey,
        signature: Ed25519Signature,
    },
    /// K-of-N multisignature
    MultiEd25519 {
        public_key: MultiEd25519PublicKey,
        signature: MultiEd25519Signature,
    },
    /// Multi-agent transaction.
    MultiAgent {
        sender: AccountAuthenticator,
        secondary_signer_addresses: Vec<LegacyAddressV5>,
        secondary_signers: Vec<AccountAuthenticator>,
    },
}


// /// Support versioning of the data structure.
// #[derive(Clone, Serialize, Deserialize)]
// enum ContractEvent {
//     V0(ContractEventV0),
// }


// /// Entry produced via a call to the `emit_event` builtin.
// #[derive(Clone, Serialize, Deserialize)]
// struct ContractEventV0 {
//     /// The unique key that the event was emitted to
//     key: EventKey,
//     /// The number of messages that have been emitted to the path previously
//     sequence_number: u64,
//     /// The type of the data
//     type_tag: TypeTag,
//     /// The data payload of the event
//     #[serde(with = "serde_bytes")]
//     event_data: Vec<u8>,
// }
