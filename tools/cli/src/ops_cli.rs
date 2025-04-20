use clap::Subcommand;
use libra_genesis_tools::cli::GenesisCli;
use libra_rescue::cli_main::RescueCli;
use libra_storage::storage_cli::StorageCli;
use libra_testnet::cli_main::TestnetCli;

#[derive(Subcommand)]
/// Node and DB operations tools
pub enum OpsTool {
    Genesis(GenesisCli),
    Storage(StorageCli),
    Rescue(RescueCli),
    Testnet(TestnetCli),
}

impl OpsTool {
    // TODO: note that run here is consuming the self, and not borrowing.
    // this is because downstream StorageCli::Db::DbTool, cannot be copied.
    pub async fn run(self) -> anyhow::Result<()> {
        match self {
            OpsTool::Genesis(genesis_cli) => {
                if let Err(e) = genesis_cli.run().await {
                    eprintln!("Failed to execute genesis tool, message: {}", &e);
                }
            }
            OpsTool::Storage(storage_cli) => {
                if let Err(e) = storage_cli.run().await {
                    eprintln!("Failed to execute genesis tool, message: {}", &e);
                }
            }
            OpsTool::Rescue(rescue_cli) => {
                if let Err(e) = rescue_cli.run() {
                    eprintln!("Failed to execute genesis tool, message: {}", &e);
                }
            }
            OpsTool::Testnet(tesnet_cli) => {
                if let Err(e) = tesnet_cli.run().await {
                    eprintln!("Failed to execute testnet tool, message: {}", &e);
                }
            }
        };
        Ok(())
    }
}
