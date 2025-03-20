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
                if tool.execute().await.is_err() {
                    std::process::exit(1);
                }
            }
            Self::Coverage(tool) => {
                if tool.execute().await.is_err() {
                    std::process::exit(1);
                }
            }
            Self::Prove(tool) => {
                if tool.execute().await.is_err() {
                    std::process::exit(1);
                }
            }
            Self::Test(tool) => {
                if tool.execute().await.is_err() {
                    std::process::exit(1);
                }
            }
        };
        Ok(())
    }
}
