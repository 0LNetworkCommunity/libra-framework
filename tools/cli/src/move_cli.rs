use clap::Subcommand;
use diem::{
    common::types::CliCommand,
    move_tool::{coverage, CompilePackage, ProvePackage, TestPackage},
};
use libra_framework::framework_cli::FrameworkCli;

#[derive(Subcommand)]
/// Move language tools for Libra core-devs
pub enum MoveTool {
    #[clap(subcommand)]
    Framework(FrameworkCli),
    Compile(CompilePackage),
    #[clap(subcommand)]
    Coverage(coverage::CoveragePackage),
    Prove(ProvePackage),
    Test(TestPackage),
}

impl MoveTool {
    pub async fn execute(self) -> anyhow::Result<()> {
        match self {
            Self::Framework(tool) => {
                tool.execute()?;
            }
            Self::Compile(tool) => {
                let _ = tool.execute_serialized().await;
            }
            Self::Coverage(tool) => {
                let _ = tool.execute().await;
            }
            Self::Prove(tool) => {
                let _ = tool.execute_serialized().await;
            }
            Self::Test(tool) => {
                let _ = tool.execute_serialized().await;
            }
        };
        Ok(())
    }
}
