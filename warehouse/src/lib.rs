// pub mod age_init;
pub mod cypher_templates;
pub mod extract_snapshot;
pub mod extract_transactions;
// pub mod load_account;
// pub mod load_coin;
pub mod load;
pub mod load_supporting_data;
pub mod load_tx_cypher;
// pub mod migrate;
pub mod neo4j_init;
// pub mod query_balance;
// pub mod restaurant;
pub mod scan;
pub mod supporting_data;
pub mod table_structs;
pub mod unzip_temp;
pub mod warehouse_cli;

use std::sync::Once;

static LOGGER: Once = Once::new();

/// Setup function that is only run once, even if called multiple times.
pub fn log_setup() {
    LOGGER.call_once(|| {
        env_logger::init();
    });
}
