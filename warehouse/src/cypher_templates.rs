//! organic free trade template literals for cypher queries
use anyhow::Result;

// TODO move this to a .CQL file so we can lint and debug
pub fn write_batch_tx_string(list_str: String) -> String {
    format!(
        r#"
WITH {list_str} AS tx_data
UNWIND tx_data AS tx
// Merge Accounts and set creation or modification flags
MERGE (from:Account {{address: tx.sender}})
ON CREATE SET from.created_at = timestamp(), from.modified_at = null
ON MATCH SET from.modified_at = timestamp()

MERGE (to:Account {{address: tx.recipient}})
ON CREATE SET to.created_at = timestamp(), to.modified_at = null
ON MATCH SET to.modified_at = timestamp()

// CREATE Transaction Relationship and set creation flag

MERGE (from)-[rel:Tx {{tx_hash: tx.tx_hash}}]->(to)
ON CREATE SET rel.created_at = timestamp(), rel.modified_at = null
ON MATCH SET rel.modified_at = timestamp()
SET
    rel += tx.args,
    rel.block_datetime = tx.block_datetime,
    rel.block_timestamp = tx.block_timestamp,
    rel.relation = tx.relation,
    rel.function = tx.function

// Count created, modified, and unchanged Account nodes and Tx relationships based on current timestamp
WITH
  COUNT(CASE WHEN from.created_at = timestamp() THEN 1 END) AS created_accounts,
  COUNT(CASE WHEN from.modified_at = timestamp() AND from.created_at IS NULL THEN 1 END) AS modified_accounts,
  COUNT(CASE WHEN from.modified_at < timestamp() THEN 1 END) AS unchanged_accounts,
  COUNT(CASE WHEN rel.created_at = timestamp() THEN 1 END) AS created_tx
RETURN created_accounts, modified_accounts, unchanged_accounts, created_tx
"#
    )
}

use anyhow::bail;
use serde::Serialize;
use serde_json::Value;

/// Converts a serializable struct to a Cypher-compatible object string,
/// handling nested objects, arrays, and basic types.
///
/// # Arguments
/// - `object`: The serializable struct.
///
/// # Returns
/// A string in the format `{key: value, nested: {key2: value2}, array: [value3, value4]}` that can be used in Cypher queries.
///  Thanks Copilot ;)
pub fn to_cypher_object<T: Serialize>(object: &T, prefix: Option<&str>) -> Result<String> {
    // Serialize the struct to a JSON value

    let serialized_value = serde_json::to_value(object).expect("Failed to serialize");
    dbg!(&serialized_value);

    // Convert the JSON value into a map for easy processing
    let map = if let Value::Object(obj) = serialized_value {
        obj
    } else {
        bail!("Expected the serialized value to be an object");
    };

    // Build properties part of the Cypher object
    let properties: Vec<String> = map
        .into_iter()
        .map(|(mut key, value)| {
            let formatted_value = match value {
                Value::String(s) => format!("'{}'", s), // Wrap strings in single quotes
                Value::Number(n) => n.to_string(),      // Use numbers as-is
                Value::Bool(b) => b.to_string(),        // Use booleans as-is
                Value::Null => "null".to_string(),      // Represent null values
                Value::Array(arr) => {
                    // Handle arrays by formatting each element
                    let elements: Vec<String> = arr
                        .iter()
                        .map(|elem| match elem {
                            Value::String(s) => format!("'{}'", s),
                            Value::Number(n) => n.to_string(),
                            Value::Bool(b) => b.to_string(),
                            Value::Null => "null".to_string(),
                            Value::Object(_) => {
                                to_cypher_object(elem, None).unwrap_or("error".to_owned())
                            } // Recurse for nested objects in arrays
                            _ => panic!("Unsupported type in array for Cypher serialization"),
                        })
                        .collect();
                    format!("[{}]", elements.join(", "))
                }
                Value::Object(_) => {
                    if let Some(p) = prefix {
                        key = format!("{}.{}", p, key);
                    }
                    to_cypher_object(&value, Some(&key)).unwrap_or("error".to_owned())
                } // Recurse for nested objects
            };
            format!("{}: {}", key, formatted_value)
        })
        .collect();

    // Join properties with commas and wrap in curly braces to form a Cypher-compatible object
    Ok(format!("{{{}}}", properties.join(", ")))
}

#[test]
fn test_serialize_to_cypher_object() {
    use diem_types::account_address::AccountAddress;

    // Example structs to demonstrate usage
    #[derive(Serialize)]
    struct Address {
        city: String,
        zip: String,
    }

    #[derive(Serialize)]
    struct Person {
        name: String,
        account: AccountAddress,
        age: u32,
        active: bool,
        hobbies: Vec<String>,
        address: Address, // Nested object
    }

    // Example usage with a `Person` struct that includes a nested `Address` struct and an array
    let person = Person {
        name: "Alice".to_string(),
        account: AccountAddress::ZERO,
        age: 30,
        active: true,
        hobbies: vec![
            "Reading".to_string(),
            "Hiking".to_string(),
            "Coding".to_string(),
        ],
        address: Address {
            city: "Wonderland".to_string(),
            zip: "12345".to_string(),
        },
    };

    // Serialize to a Cypher object
    let cypher_object = to_cypher_object(&person, None).unwrap();
    println!("{}", cypher_object);
}
