use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Deserializer};
use std::fs::File;
use std::io::Read;
use std::path::Path;

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct Order {
    user: u32,
    #[serde(rename = "orderType")]
    order_type: String,
    #[serde(deserialize_with = "deserialize_amount")]
    amount: f64,
    #[serde(deserialize_with = "deserialize_amount")]
    price: f64,
    created_at: DateTime<Utc>,
    filled_at: DateTime<Utc>,
    accepter: u32,
}

// Custom deserialization function for "amount"
fn deserialize_amount<'de, D>(deserializer: D) -> Result<f64, D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    s.parse::<f64>().map_err(serde::de::Error::custom)
}

fn deserialize_orders(json_data: &str) -> Result<Vec<Order>> {
    let orders: Vec<Order> = serde_json::from_str(json_data)?;
    Ok(orders)
}

pub fn read_json_from_file<P: AsRef<Path>>(path: P) -> Result<Vec<Order>> {
    let mut file = File::open(path)?;
    let mut json_data = String::new();
    file.read_to_string(&mut json_data)?;
    Ok(deserialize_orders(&json_data)?)
}

#[test]
fn test_deserialize_orders() {
    // Raw string literal for test JSON data
    let json_data = r#"
        [
            {"user":1,"orderType":"Sell","amount":"40000.000","price":"0.00460","created_at":"2024-05-12T15:25:14.991Z","filled_at":"2024-05-14T15:04:13.000Z","accepter":3768},
            {"user":2,"orderType":"Sell","amount":"100000.000","price":"0.00994","created_at":"2024-03-11T17:23:49.860Z","filled_at":"2024-03-11T17:31:43.000Z","accepter":2440},
            {"user":3,"orderType":"Sell","amount":"50000.000","price":"0.00998","created_at":"2024-03-11T14:46:49.377Z","filled_at":"2024-03-11T14:47:12.000Z","accepter":3710},
            {"user":4,"orderType":"Buy","amount":"3027220.000","price":"0.00110","created_at":"2024-01-14T13:33:13.688Z","filled_at":"2024-01-14T18:02:44.000Z","accepter":227}
        ]
        "#;

    // Use the `deserialize_orders` function to parse the raw JSON data
    let orders = deserialize_orders(json_data).expect("Failed to deserialize orders");

    // Check that the result matches the expected values
    assert_eq!(orders.len(), 4);
    assert_eq!(orders[0].user, 1);
    assert_eq!(orders[0].order_type, "Sell");
    assert_eq!(orders[0].amount, 40000.000);
    assert_eq!(orders[0].accepter, 3768);
}
