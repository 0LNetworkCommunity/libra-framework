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

pub fn write_trade_data_string(list_str: String) -> String {
    format!(
        r#"
  WITH {list_str} AS tx_data
  UNWIND tx_data AS tx
  MERGE (maker:SwapAccount {{id: tx.user}})
  MERGE (taker:SwapAccount {{id: tx.accepter}})
  MERGE (from)-[rel:Swap {{
    order_type: tx.order_type,
    amount: tx.amount, price: tx.price,
    tx.created_at,
    tx.filled_at,
  }}]->(to)

  ON CREATE SET rel.created = true
  ON MATCH SET rel.created = false
  WITH tx, rel
  RETURN
      COUNT(CASE WHEN rel.created = true THEN 1 END) AS merged_tx_count,
      COUNT(CASE WHEN rel.created = false THEN 1 END) AS ignored_tx_count
"#
    )
}
