// NOTE: ported from libra-legacy-v6/json-rpc/types/src/views.rs
use anyhow::Result;
// use diem_crypto::HashValue;
use serde::{de::Error, Deserialize, Deserializer, Serialize, Serializer};

use crate::version_five::{
    event_v5::EventKeyV5 as EventKey, hash_value_v5::HashValueV5 as HashValue,
    legacy_address_v5::LegacyAddressV5 as AccountAddress,
};

use hex::FromHex;
/// This is the output of a JSON API request on the V5 data
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct TransactionViewV5 {
    pub version: u64,
    pub transaction: TransactionDataView,
    pub hash: HashValue,
    pub bytes: BytesView,
    pub events: Vec<EventView>,
    pub vm_status: VMStatusView,
    pub gas_used: u64,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub enum VMStatusView {
    Executed,
    OutOfGas,
    MoveAbort {
        location: String,
        abort_code: u64,
        explanation: Option<MoveAbortExplanationView>,
    },
    ExecutionFailure {
        location: String,
        function_index: u16,
        code_offset: u16,
    },
    MiscellaneousError,
    VerificationError,
    DeserializationError,
    PublishingFailure,
    #[serde(other)]
    Unknown,
}

impl VMStatusView {
    pub fn is_executed(&self) -> bool {
        matches!(self, Self::Executed)
    }
}

impl std::fmt::Display for VMStatusView {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VMStatusView::Executed => write!(f, "Executed"),
            VMStatusView::OutOfGas => write!(f, "Out of Gas"),
            VMStatusView::MoveAbort {
                location,
                abort_code,
                explanation,
            } => {
                write!(f, "Move Abort: {} at {}", abort_code, location)?;
                if let Some(explanation) = explanation {
                    write!(f, "\nExplanation:\n{:?}", explanation)?
                }
                Ok(())
            }
            VMStatusView::ExecutionFailure {
                location,
                function_index,
                code_offset,
            } => write!(
                f,
                "Execution failure: {} {} {}",
                location, function_index, code_offset
            ),
            VMStatusView::MiscellaneousError => write!(f, "Miscellaneous Error"),
            VMStatusView::VerificationError => write!(f, "Verification Error"),
            VMStatusView::DeserializationError => write!(f, "Deserialization Error"),
            VMStatusView::PublishingFailure => write!(f, "Publishing Failure"),
            VMStatusView::Unknown => write!(f, "Unknown Error"),
        }
    }
}

#[allow(clippy::large_enum_variant)]
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(tag = "type")]
pub enum TransactionDataView {
    #[serde(rename = "blockmetadata")]
    BlockMetadata { timestamp_usecs: u64 },
    #[serde(rename = "writeset")]
    WriteSet {},
    #[serde(rename = "user")]
    UserTransaction {
        sender: AccountAddress,
        signature_scheme: String,
        signature: BytesView,
        public_key: BytesView,
        #[serde(skip_serializing_if = "Option::is_none")]
        secondary_signers: Option<Vec<AccountAddress>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        secondary_signature_schemes: Option<Vec<String>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        secondary_signatures: Option<Vec<BytesView>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        secondary_public_keys: Option<Vec<BytesView>>,
        sequence_number: u64,
        chain_id: u8,
        max_gas_amount: u64,
        gas_unit_price: u64,
        gas_currency: String,
        expiration_timestamp_secs: u64,
        script_hash: HashValue,
        script_bytes: BytesView,
        script: ScriptView,
    },
    #[serde(rename = "unknown")]
    #[serde(other)]
    UnknownTransaction,
}

#[derive(Clone, PartialEq)]
pub struct BytesView(pub Box<[u8]>);

impl BytesView {
    pub fn new<T: Into<Box<[u8]>>>(bytes: T) -> Self {
        Self(bytes.into())
    }

    pub fn into_inner(self) -> Box<[u8]> {
        self.0
    }

    pub fn inner(&self) -> &[u8] {
        &self.0
    }
}

impl std::ops::Deref for BytesView {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::convert::AsRef<[u8]> for BytesView {
    fn as_ref(&self) -> &[u8] {
        self.inner()
    }
}

impl std::fmt::Display for BytesView {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        for byte in self.inner() {
            write!(f, "{:02x}", byte)?;
        }
        Ok(())
    }
}

impl std::fmt::Debug for BytesView {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "BytesView(\"{}\")", self)
    }
}

impl From<&[u8]> for BytesView {
    fn from(bytes: &[u8]) -> Self {
        Self(bytes.into())
    }
}

impl From<Vec<u8>> for BytesView {
    fn from(bytes: Vec<u8>) -> Self {
        Self(bytes.into_boxed_slice())
    }
}

impl<'de> Deserialize<'de> for BytesView {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = <String>::deserialize(deserializer)?;
        <Vec<u8>>::from_hex(s)
            .map_err(|e| D::Error::custom(e.to_string()))
            .map(Into::into)
    }
}

impl Serialize for BytesView {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        hex::encode(self).serialize(serializer)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct EventView {
    pub key: EventKey,
    pub sequence_number: u64,
    pub transaction_version: u64,
    pub data: EventDataView,
}

// impl TryFrom<(u64, ContractEvent)> for EventView {
//     type Error = Error;

//     fn try_from((txn_version, event): (u64, ContractEvent)) -> Result<Self> {
//         Ok(EventView {
//             key: *event.key(),
//             sequence_number: event.sequence_number(),
//             transaction_version: txn_version,
//             data: event.try_into()?,
//         })
//     }
// }

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct MoveAbortExplanationView {
    pub category: String,
    pub category_description: String,
    pub reason: String,
    pub reason_description: String,
}

// impl std::fmt::Display for MoveAbortExplanationView {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         writeln!(f, "Error Category: {}", self.category)?;
//         writeln!(f, "\tCategory Description: {}", self.category_description)?;
//         writeln!(f, "Error Reason: {}", self.reason)?;
//         writeln!(f, "\tReason Description: {}", self.reason_description)
//     }
// }

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq)]
pub struct ScriptView {
    // script name / type
    pub r#type: String,

    // script code bytes
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<BytesView>,
    // script arguments, converted into string with type information
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arguments: Option<Vec<String>>,
    // script function arguments, converted into hex encoded BCS bytes
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arguments_bcs: Option<Vec<BytesView>>,
    // script type arguments, converted into string
    #[serde(skip_serializing_if = "Option::is_none")]
    pub type_arguments: Option<Vec<String>>,

    // the following fields are legacy fields: maybe removed in the future
    // please move to use above fields

    // peer_to_peer_transaction, other known script name or unknown
    // because of a bug, we never rendered mint_transaction
    // this is deprecated, please switch to use field `name` which is script name

    // peer_to_peer_transaction
    #[serde(skip_serializing_if = "Option::is_none")]
    pub receiver: Option<AccountAddress>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub amount: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub currency: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<BytesView>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata_signature: Option<BytesView>,

    // Script functions
    // The address that the module is published under
    #[serde(skip_serializing_if = "Option::is_none")]
    pub module_address: Option<AccountAddress>,
    // The name of the module that the called function is defined in
    #[serde(skip_serializing_if = "Option::is_none")]
    pub module_name: Option<String>,
    // The (unqualified) name of the function being called.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub function_name: Option<String>,
}

// impl ScriptView {
//     pub fn unknown() -> Self {
//         ScriptView {
//             r#type: "unknown".to_string(),
//             ..Default::default()
//         }
//     }
// }

// impl From<&Script> for ScriptView {
//     fn from(script: &Script) -> Self {
//         let name = ScriptCall::decode(script)
//             .map(|script_call| script_call.name().to_owned())
//             .unwrap_or_else(|| "unknown".to_owned());
//         let ty_args: Vec<String> = script
//             .ty_args()
//             .iter()
//             .map(|type_tag| match type_tag {
//                 TypeTag::Struct(StructTag { module, .. }) => module.to_string(),
//                 tag => format!("{}", tag),
//             })
//             .collect();
//         let mut view = ScriptView {
//             r#type: name.clone(),
//             code: Some(script.code().into()),
//             arguments: Some(
//                 script
//                     .args()
//                     .iter()
//                     .map(|arg| format!("{:?}", &arg))
//                     .collect(),
//             ),
//             type_arguments: Some(ty_args.clone()),
//             ..Default::default()
//         };

//         // handle legacy fields, backward compatible
//         if name == "peer_to_peer_with_metadata" {
//             if let [TransactionArgument::Address(receiver), TransactionArgument::U64(amount), TransactionArgument::U8Vector(metadata), TransactionArgument::U8Vector(metadata_signature)] =
//                 script.args()
//             {
//                 view.receiver = Some(*receiver);
//                 view.amount = Some(*amount);
//                 view.currency = Some(
//                     ty_args
//                         .get(0)
//                         .unwrap_or(&"unknown_currency".to_string())
//                         .to_string(),
//                 );
//                 view.metadata = Some(BytesView::new(metadata.as_ref()));
//                 view.metadata_signature = Some(BytesView::new(metadata_signature.as_ref()));
//             }
//         }

//         view
//     }
// }

// impl From<&ScriptFunction> for ScriptView {
//     fn from(script: &ScriptFunction) -> Self {
//         let ty_args: Vec<String> = script
//             .ty_args()
//             .iter()
//             .map(|type_tag| match type_tag {
//                 TypeTag::Struct(StructTag { module, .. }) => module.to_string(),
//                 tag => format!("{}", tag),
//             })
//             .collect();
//         ScriptView {
//             r#type: "script_function".to_string(),
//             module_address: Some(*script.module().address()),
//             module_name: Some(script.module().name().to_string()),
//             function_name: Some(script.function().to_string()),
//             arguments_bcs: Some(
//                 script
//                     .args()
//                     .iter()
//                     .map(|arg| BytesView::from(arg.as_ref()))
//                     .collect(),
//             ),
//             type_arguments: Some(ty_args),
//             ..Default::default()
//         }
//     }
// }
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type")]
pub enum EventDataView {
    #[serde(rename = "burn")]
    Burn {
        amount: AmountView,
        preburn_address: AccountAddress,
    },
    #[serde(rename = "cancelburn")]
    CancelBurn {
        amount: AmountView,
        preburn_address: AccountAddress,
    },
    #[serde(rename = "mint")]
    Mint { amount: AmountView },
    #[serde(rename = "to_xdx_exchange_rate_update")]
    ToXDXExchangeRateUpdate {
        currency_code: String,
        new_to_xdx_exchange_rate: f32,
    },
    #[serde(rename = "preburn")]
    Preburn {
        amount: AmountView,
        preburn_address: AccountAddress,
    },
    #[serde(rename = "receivedpayment")]
    ReceivedPayment {
        amount: AmountView,
        sender: AccountAddress,
        receiver: AccountAddress,
        metadata: BytesView,
    },
    #[serde(rename = "sentpayment")]
    SentPayment {
        amount: AmountView,
        receiver: AccountAddress,
        sender: AccountAddress,
        metadata: BytesView,
    },
    #[serde(rename = "admintransaction")]
    AdminTransaction { committed_timestamp_secs: u64 },
    #[serde(rename = "newepoch")]
    NewEpoch { epoch: u64 },
    #[serde(rename = "newblock")]
    NewBlock {
        round: u64,
        proposer: AccountAddress,
        proposed_time: u64,
    },
    #[serde(rename = "receivedmint")]
    ReceivedMint {
        amount: AmountView,
        destination_address: AccountAddress,
    },
    #[serde(rename = "compliancekeyrotation")]
    ComplianceKeyRotation {
        new_compliance_public_key: BytesView,
        time_rotated_seconds: u64,
    },
    #[serde(rename = "baseurlrotation")]
    BaseUrlRotation {
        new_base_url: String,
        time_rotated_seconds: u64,
    },
    #[serde(rename = "createaccount")]
    CreateAccount {
        created_address: AccountAddress,
        role_id: u64,
    },
    #[serde(rename = "diemiddomain")]
    DiemIdDomain {
        // Whether a domain was added or removed
        removed: bool,
        // Diem ID Domain string of the account
        // TODO
        domain: String,
        // domain: DiemIdVaspDomainIdentifier,
        // On-chain account address
        address: AccountAddress,
    },
    #[serde(rename = "unknown")]
    Unknown { bytes: Option<BytesView> },

    // used by client to deserialize server response
    #[serde(other)]
    UnknownToClient,
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub struct AmountView {
    pub amount: u64,
    pub currency: String,
}
