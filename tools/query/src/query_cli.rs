use anyhow::Result;
use clap::Parser;
use libra_query::query_type::QueryType;

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
        // let client = Client::default()?;
        // TODO: get client from configs

        let res = self.subcommand.query_to_json(None).await?;

        println!("{:#?}", res);

        Ok(())
    }
}
