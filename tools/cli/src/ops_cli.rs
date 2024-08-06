use libra_genesis_tools::cli::GenesisCli;
#[derive(Subcommand)]
/// Node and DB operations tools
pub enum OpsTool {
    Genesis(GenesisCli),
    // Storage(StorageCli),
    // Rescue(RescueCli),
}


impl OpsTool {
    pub fn execute(self) -> anyhow::Result<()> {
        match self {
                    // Execute Genesis CLI subcommand
          Self::Genesis(genesis_cli) => {
              if let Err(e) = genesis_cli.execute().await {
                  eprintln!("Failed to execute genesis tool, message: {}", &e);
              }
          }
        //     Self::Framework(tool) => {
        //         tool.execute()?;
        //     }
        //     Self::Compile(tool) => {
        //         let _ = tool.execute_serialized().await;
        //     }
        //     Self::Coverage(tool) => {
        //         let _ = tool.execute().await;
        //     }
        //     Self::Prove(tool) => {
        //         let _ = tool.execute_serialized().await;
        //     }
        //     Self::Test(tool) => {
        //         let _ = tool.execute_serialized().await;
        //     }
        };
        Ok(())
    }
}
