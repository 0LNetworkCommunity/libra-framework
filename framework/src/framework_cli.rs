//! framework cli entry points

use crate::{
    builder::framework_generate_upgrade_proposal::{
        init_move_dir_wrapper, libra_compile_script, make_framework_upgrade_artifacts, save_build,
    },
    release::ReleaseTarget,
};

use anyhow::Context;
use clap::Parser;
use std::path::PathBuf;

#[derive(Debug, Parser)]
/// Creates a framework release used for test genesis (as well as production genesis).
pub struct GenesisRelease {
    /// The release target. One of head, devnet, testnet, or mainnet. Notice the type
    /// of target determines what packages are included in the release. For example,
    /// some packages may be available in testnet, but aren't in mainnet.
    #[clap(long, default_value = "head")]
    pub target: ReleaseTarget,

    /// Remove the source code from the release package to shrink its size.
    #[clap(short, long)]
    pub without_source_code: bool,
}

impl GenesisRelease {
    pub fn execute(&self) -> anyhow::Result<()> {
        self.target.create_release(!self.without_source_code, None)
    }
}

/// Creates all artifacts for a framework governance upgrade.
/// This is for all or any code deployed at the system 0x1 address.
#[derive(Debug, Parser)]
pub struct FrameworkUpgrade {
    /// dir to save all the artifacts for the release.
    #[clap(short, long)]
    pub output_dir: PathBuf,

    /// directory of the framework source code. Usually `./framework/lbra-framework`
    #[clap(short, long)]
    pub framework_local_dir: PathBuf,

    /// optional, list of core module directory names to compile. It will default to this order: move-stdlib, vendor-stdlib, libra-framework
    #[clap(long)]
    pub core_modules: Option<Vec<String>>,
}

impl FrameworkUpgrade {
    pub fn execute(&self) -> anyhow::Result<()> {
        // let release_cfg = libra_release_cfg_default();
        if !self.output_dir.exists() {
            println!(
                "creating output dir at {}",
                self.output_dir.to_str().unwrap()
            );
            std::fs::create_dir_all(&self.output_dir)?;
        }

        make_framework_upgrade_artifacts(
            &self.output_dir,
            &self.framework_local_dir,
            &self.core_modules,
        )?;

        Ok(())
    }
}

/// Builds artifacts for any governance proposal that requires human written scripts.
/// Also creates a template for a libra governance script
#[derive(Debug, Parser)]
pub struct GovernanceScript {
    /// dir to save all the artifacts for the release.
    #[clap(short, long)]
    pub output_dir: PathBuf,

    /// directory of the framework source code. Usually `./framework/lbra-framework`
    #[clap(short, long)]
    pub framework_local_dir: PathBuf,
}

impl GovernanceScript {
    pub fn execute(&self) -> anyhow::Result<()> {
        let script_name = "framework_upgrade";
        let package_dir = self.output_dir.join(script_name);
        if !package_dir.exists() {
            println!(
                "script dir does not exist, will create one now at {:?} ",
                &package_dir.to_str()
            );

            std::fs::create_dir_all(&package_dir)
                .context("could not create the output directory {new_path:?}")?;
            // TODO: rename this. init_move_package_with_local_framework
            init_move_dir_wrapper(
                package_dir.clone(),
                script_name,
                self.framework_local_dir.clone(),
            )?;
            let t = r#"
script {
  use aptos_framework::aptos_governance;

  fun main(proposal_id: u64){
    let _framework_signer = aptos_governance::resolve(proposal_id, @0000000000000000000000000000000000000000000000000000000000000001);
  }
}
"#;
            let filename = package_dir
                .join("sources")
                .join(format!("{}.move", script_name));
            std::fs::write(filename, t)?;
            println!("governance template created");
            return Ok(());
        }

        let (bytes, hash) = libra_compile_script(&package_dir, false)?;
        save_build(package_dir, &bytes, &hash)?;

        Ok(())
    }
}
