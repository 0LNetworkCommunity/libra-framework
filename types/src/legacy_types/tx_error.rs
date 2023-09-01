use anyhow;
use zapatos_api_types::TransactionOnChainData;

#[derive(Debug)]
/// a transaction error type specific to ol txs
pub struct TxError {
    /// the actual error type
    pub err: Option<anyhow::Error>,
    /// transaction view if the transaction got that far
    pub tx_view: Option<TransactionOnChainData>,
    /// Move module or script where error occurred
    pub location: Option<String>,
    /// Move abort code used in error
    pub abort_code: Option<u64>,
}

impl From<anyhow::Error> for TxError {
    fn from(e: anyhow::Error) -> Self {
        TxError {
            err: Some(e),
            tx_view: None,
            location: None,
            abort_code: None,
        }
    }
}
