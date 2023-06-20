use anyhow::Result;
use clap::Parser;
use libra_types::type_extensions::client_ext::ClientExt;
use libra_query::querier::QueryType;
use zapatos_sdk::rest_client::Client;

#[derive(Parser)]
#[clap(name = env!("CARGO_PKG_NAME"), author, version, about, long_about = None, arg_required_else_help = true)]
pub struct QueryCli {
    #[clap(subcommand)]
    subcommand: QueryType,
}

// #[derive(clap::Subcommand)]
// enum Subcommand {
//     /// Get account balance
//     AccountBalance {
//         /// Address of the onchain account to get balance from
//         #[clap(short, long)]
//         account_address: String,
//     },

//     /// Get all resources of an account
//     AccountResources {
//         /// Address of the onchain account to get resources from
//         #[clap(short, long)]
//         account_address: String,
//     },
// }

impl QueryCli {
    pub async fn run(&self) -> Result<()> {
        let client = Client::default()?;
        // let querier = Querier::new(client);

        let res = self.subcommand.query(&client).await?;

        println!("{:#?}", res);

        Ok(())
    }
}
