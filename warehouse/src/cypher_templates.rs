//! organic free trade template literals for cypher queries

pub fn write_batch_tx_string(list_str: String) -> String {
    format!(
        r#"
  WITH {list_str} AS tx_data
  UNWIND tx_data AS tx
  MERGE (from:Account {{address: tx.sender}})
  MERGE (to:Account {{address: tx.recipient}})
  MERGE (from)-[:Tx {{tx_hash: tx.tx_hash}}]->(to)
"#
    )
}
