//! organic free trade template literals for cypher queries

pub fn write_batch_tx_string(list_str: String) -> String {
    format!(
        r#"
  WITH {list_str} AS tx_data
  UNWIND tx_data AS tx
  MERGE (from:Account {{address: tx.sender}})
  MERGE (to:Account {{address: tx.recipient}})
  MERGE (from)-[rel:Tx {{tx_hash: tx.tx_hash}}]->(to)

  ON CREATE SET rel.created = true
  ON MATCH SET rel.created = false
  WITH tx, rel
  RETURN
      COUNT(CASE WHEN rel.created = true THEN 1 END) AS merged_tx_count,
      COUNT(CASE WHEN rel.created = false THEN 1 END) AS ignored_tx_count
"#
    )
}
