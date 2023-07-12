use anyhow::{ Result};
use zapatos_types::account_address::AccountAddress;

pub async fn run(account_address: &str, _coins: u64) -> Result<()> {
    let _account_address = AccountAddress::from_hex_literal(account_address)?;

    println!("Success!");
    Ok(())
}
