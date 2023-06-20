use anyhow::Result;
use clap::Parser;
use libra_types::type_extensions::client_ext::ClientExt;
use libra_query::querier::{Querier, QueryType::*};
use zapatos_sdk::{rest_client::Client, types::account_address::AccountAddress};

#[derive(Parser)]
#[clap(name = env!("CARGO_PKG_NAME"), author, version, about, long_about = None, arg_required_else_help = true)]
pub struct QueryCli {
    #[clap(subcommand)]
    subcommand: Option<Subcommand>,
}

#[derive(clap::Subcommand)]
enum Subcommand {
    /// Get account balance
    AccountBalance {
        /// Address of the onchain account to get balance from
        #[clap(short, long)]
        account_address: String,
    },

    /// Get all resources of an account
    AccountResources {
        /// Address of the onchain account to get resources from
        #[clap(short, long)]
        account_address: String,
    },
}

impl QueryCli {
    pub async fn run(&self) -> Result<()> {
        let client = Client::default()?;
        let querier = Querier::new(client);

        match &self.subcommand {
            Some(Subcommand::AccountBalance { account_address }) => {
                let account = AccountAddress::from_hex_literal(account_address)?;
                let balance = querier.query(Balance { account }).await?;
                println!("Account balance: {balance} coins");
            }
            Some(Subcommand::AccountResources { account_address }) => {
                let account = AccountAddress::from_hex_literal(account_address)?;
                let resources = querier.query(Resources { account }).await?;
                println!("{resources}");
            }
            _ => { /* do nothing */ }
        }
        Ok(())
    }
}
