use crate::{cli_output::TestInfo, twin_swarm};
use clap::{self, Parser};
use diem_framework::ReleaseBundle;
use libra_smoke_tests::libra_smoke::LibraSmoke;
use std::{fs, path::PathBuf};
/// Twin of the network
#[derive(Parser)]

/// Starts a quick testnet locally using Diem swarm
/// Requires that you have a `libra` binary built and
/// the envvar DIEM_FORGE_NODE_BIN_PATH pointed to it
/// e.g. DIEM_FORGE_NODE_BIN_PATH=$HOME/.cargo/bin/libra
pub struct SwarmCliOpts {
    /// number of local validators to start
    #[clap(long, short)]
    pub count_vals: Option<u8>,
}
impl SwarmCliOpts {
    /// Runner for swarm
    pub async fn run(
        &self,
        framework_mrb_path: Option<PathBuf>,
        twin_db: Option<PathBuf>,
        json_output: bool,
        json_file: Option<PathBuf>,
    ) -> anyhow::Result<Vec<TestInfo>> {
        let num_validators = self.count_vals.unwrap_or(2);

        let bundle = if let Some(p) = framework_mrb_path.clone() {
            ReleaseBundle::read(p)?
        } else {
            println!("assuming you are running this in the source repo. Will try to search in this path at ./framework/releases/head.mrb");
            libra_framework::testing_local_release_bundle()
        };

        let mut smoke = LibraSmoke::new_with_bundle(Some(num_validators), None, bundle).await?;

        let out = if let Some(p) = twin_db {
            let db_path = fs::canonicalize(p)?;

            twin_swarm::awake_frankenswarm(&mut smoke, Some(db_path)).await?
        } else {
            smoke
                .swarm
                .wait_all_alive(tokio::time::Duration::from_secs(120))
                .await?;

            TestInfo::from_smoke(&smoke.swarm)?
        };

        // Print or write JSON output before the interactive prompt
        if json_output {
            println!("{}", serde_json::to_string_pretty(&out)?);
        }

        // Write to JSON file if specified - do this before waiting for user input
        if let Some(file_path) = json_file {
            println!("Writing test info to JSON file: {:?}", file_path);
            fs::write(&file_path, serde_json::to_string_pretty(&out)?)
                .map_err(|e| anyhow::anyhow!("Failed to write to JSON file: {}", e))?;
            println!("Successfully wrote test info to {:?}", file_path);
        } else {
            // If not writing to file, print the regular output
            println!("{}", serde_json::to_string_pretty(&out)?);
        }

        // NOTE: all validators will stop when the LibraSmoke goes out of context. This is intentional
        // but since it's borrowed in this function you should assume it will continue until the caller goes out of scope.
        let _ = dialoguer::Input::<String>::new()
            .with_prompt(
                format!("\n{FIRE}\nsmoke success: {num_validators} `libra` processes are running until you exit. Press any key to exit"),
            )
            .interact_text()?;

        Ok(out)
    }
}

pub const FIRE: &str = r#"
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢀⡀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢻⣦⡀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢿⣿⣦⡀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢸⣿⣿⣿⣄⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢸⣿⣿⣿⣿⣆⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣼⣿⣿⣿⣿⣿⣆⢳⡀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢰⣿⣿⣿⣿⣿⣿⣿⣾⣷⡀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢠⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣧⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠠⣄⠀⢠⣿⣿⣿⣿⡎⢻⣿⣿⣿⣿⣿⣿⡆⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⢸⣧⢸⣿⣿⣿⣿⡇⠀⣿⣿⣿⣿⣿⣿⣧⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⢸⣿⣾⣿⣿⣿⣿⠃⠀⢸⣿⣿⣿⣿⣿⣿⠀⣄⠀⠀
⠀⠀⠀⠀⠀⠀⠀⢠⣾⣿⣿⣿⣿⣿⠏⠀⠀⣸⣿⣿⣿⣿⣿⡿⢀⣿⡆⠀
⠀⠀⠀⠀⠀⢀⣴⣿⣿⣿⣿⣿⣿⠃⠀⠀⠀⣿⣿⣿⣿⣿⣿⠇⣼⣿⣿⡄
⠀⢰⠀⠀⣴⣿⣿⣿⣿⣿⣿⡿⠁⠀⠀⠀⢠⣿⣿⣿⣿⣿⡟⣼⣿⣿⣿⣧
⠀⣿⡀⢸⣿⣿⣿⣿⣿⣿⡟⠀⠀⠀⠀⠀⣸⡿⢻⣿⣿⣿⣿⣿⣿⣿⣿⣿
⠀⣿⣷⣼⣿⣿⣿⣿⣿⡟⠀⠀⠀⠀⠀⠀⢹⠃⢸⣿⣿⣿⣿⣿⣿⣿⣿⣿
⡄⢻⣿⣿⣿⣿⣿⣿⡿⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢻⣿⣿⣿⣿⣿⣿⣿⠇
⢳⣌⢿⣿⣿⣿⣿⣿⠃⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠻⣿⣿⣿⣿⣿⠏⠀
⠀⢿⣿⣿⣿⣿⣿⣿⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢹⣿⣿⣿⠋⣠⠀
⠀⠈⢻⣿⣿⣿⣿⣿⡄⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢸⣿⣿⣵⣿⠃⠀
⠀⠀⠀⠙⢿⣿⣿⣿⣷⡀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣸⣿⣿⡿⠃⠀⠀
⠀⠀⠀⠀⠀⠙⢿⣿⣿⣷⡀⠀⠀⠀⠀⠀⠀⠀⠀⢀⣴⣿⡿⠋⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠈⠛⠿⣿⣦⣀⠀⠀⠀⠀⢀⣴⠿⠛⠁⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠉⠉⠓⠂⠀⠈⠉⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
"#;
