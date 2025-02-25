use crate::version_five::{
    event_v5::EventKeyV5,
    hash_value_v5::HashValueV5,
    language_storage_v5::{ModuleIdV5, TypeTagV5},
    legacy_address_v5::LegacyAddressV5,
    module_v5::ModuleBundle,
    script_v5::Script,
};

use diem_crypto::{
    ed25519::{Ed25519PublicKey, Ed25519Signature},
    multi_ed25519::{MultiEd25519PublicKey, MultiEd25519Signature},
};

use diem_types::transaction::ChangeSet;

use diem_types::{chain_id::ChainId, transaction::authenticator::AccountAuthenticator};
use serde::{Deserialize, Serialize};

use super::script_v5::ScriptFunction;

#[allow(clippy::large_enum_variant)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum TransactionV5 {
    /// Transaction submitted by the user. e.g: P2P payment transaction, publishing module
    /// transaction, etc.
    /// TODO: We need to rename SignedTransaction to SignedUserTransaction, as well as all the other
    ///       transaction types we had in our codebase.
    UserTransaction(SignedTransaction),

    /// Transaction that applies a WriteSet to the current storage, it's applied manually via db-bootstrapper.
    GenesisTransaction(WriteSetPayload),

    /// Transaction to update the block metadata resource at the beginning of a block.
    BlockMetadata(BlockMetadata),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SignedTransaction {
    /// The raw transaction
    pub raw_txn: RawTransaction,

    /// Public key and signature to authenticate
    pub authenticator: TransactionAuthenticator,
}

/// Two different kinds of WriteSet transactions.
#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub enum WriteSetPayload {
    /// Directly passing in the WriteSet.
    Direct(ChangeSet),
    /// Generate the WriteSet by running a script.
    Script {
        /// Execute the script as the designated signer.
        execute_as: LegacyAddressV5,
        /// Script body that gets executed.
        script: Script,
    },
}

impl WriteSetPayload {
    pub fn should_trigger_reconfiguration_by_default(&self) -> bool {
        match self {
            Self::Direct(_) => true,
            Self::Script { .. } => false,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BlockMetadata {
    id: HashValueV5,
    round: u64,
    timestamp_usecs: u64,
    // The vector has to be sorted to ensure consistent result among all nodes
    previous_block_votes: Vec<LegacyAddressV5>,
    proposer: LegacyAddressV5,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TransactionInfoV5 {
    /// The hash of this transaction.
    transaction_hash: HashValueV5,

    /// The root hash of Sparse Merkle Tree describing the world state at the end of this
    /// transaction.
    state_root_hash: HashValueV5,

    /// The root hash of Merkle Accumulator storing all events emitted during this transaction.
    event_root_hash: HashValueV5,

    /// The amount of gas used.
    gas_used: u64,

    /// The vm status. If it is not `Executed`, this will provide the general error class. Execution
    /// failures and Move abort's recieve more detailed information. But other errors are generally
    /// categorized with no status code or other information
    status: KeptVMStatus,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
enum KeptVMStatus {
    Executed,
    OutOfGas,
    MoveAbort(AbortLocation, /* code */ u64),
    ExecutionFailure {
        location: AbortLocation,
        function: u16,
        code_offset: u16,
    },
    MiscellaneousError,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum AbortLocation {
    /// Indicates `abort` occurred in the specified module
    Module(ModuleIdV5),
    /// Indicates the `abort` occurred in a script
    Script,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RawTransaction {
    /// Sender's address.
    pub sender: LegacyAddressV5,

    /// Sequence number of this transaction. This must match the sequence number
    /// stored in the sender's account at the time the transaction executes.
    pub sequence_number: u64,

    /// The transaction payload, e.g., a script to execute.
    pub payload: TransactionPayload,

    /// Maximal total gas to spend for this transaction.
    pub max_gas_amount: u64,

    /// Price to be paid per gas unit.
    pub gas_unit_price: u64,

    /// The currency code, e.g., "XUS", used to pay for gas. The `max_gas_amount`
    /// and `gas_unit_price` values refer to units of this currency.
    pub gas_currency_code: String,

    /// Expiration timestamp for this transaction, represented
    /// as seconds from the Unix Epoch. If the current blockchain timestamp
    /// is greater than or equal to this time, then the transaction has
    /// expired and will be discarded. This can be set to a large value far
    /// in the future to indicate that a transaction does not expire.
    pub expiration_timestamp_secs: u64,

    /// Chain ID of the Diem network this transaction is intended for.
    pub chain_id: ChainId,
}

impl RawTransaction {
    pub fn into_payload(self) -> TransactionPayload {
        self.payload
    }
}

#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub enum TransactionPayload {
    /// A system maintenance transaction.
    WriteSet(WriteSetPayload),
    /// A transaction that executes code.
    Script(Script),
    /// A transaction that publishes multiple modules at the same time.
    ModuleBundle(ModuleBundle),
    /// A transaction that executes an existing script function published on-chain.
    ScriptFunction(ScriptFunction),
}

impl TransactionPayload {
    pub fn should_trigger_reconfiguration_by_default(&self) -> bool {
        match self {
            Self::WriteSet(ws) => ws.should_trigger_reconfiguration_by_default(),
            Self::Script(_) | Self::ScriptFunction(_) | Self::ModuleBundle(_) => false,
        }
    }

    pub fn into_script_function(self) -> ScriptFunction {
        match self {
            Self::ScriptFunction(f) => f,
            payload => panic!("Expected ScriptFunction(_) payload, found: {:#?}", payload),
        }
    }
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

/// Support versioning of the data structure.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ContractEventV5 {
    V0(ContractEventV0),
}

/// Entry produced via a call to the `emit_event` builtin.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ContractEventV0 {
    /// The unique key that the event was emitted to
    key: EventKeyV5,
    /// The number of messages that have been emitted to the path previously
    sequence_number: u64,
    /// The type of the data
    type_tag: TypeTagV5,
    /// The data payload of the event
    #[serde(with = "serde_bytes")]
    event_data: Vec<u8>,
}
