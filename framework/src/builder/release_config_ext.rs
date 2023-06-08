//! extends ReleaseConfig with new methods with custom trait.
use crate::builder::release_entry_ext::LibraReleaseEntry; // our refactored methods trait
use anyhow::anyhow;
use std::path::{Path, PathBuf};
use zapatos_release_builder::{ReleaseConfig, ExecutionMode};
use zapatos_temppath::TempPath;
use zapatos_crypto::HashValue;
// use zapatos_rest_client::Client;
use zapatos::{
  common::types::PromptOptions,
  move_tool::FrameworkPackageArgs,
  governance::CompileScriptFunction,
};
use zapatos_release_builder::components::Proposal;
use zapatos_release_builder::components::ProposalMetadata;
use zapatos_release_builder::ReleaseEntry;
use zapatos_release_builder::components::framework::FrameworkReleaseConfig;
pub trait LibraReleaseConfig {
    fn libra_generate_release_proposal_scripts(&self, base_path: &Path, framework_local_dir: PathBuf) -> anyhow::Result<()>;
}

impl LibraReleaseConfig for ReleaseConfig {
    fn libra_generate_release_proposal_scripts(&self, base_path: &Path, framework_local_dir: PathBuf) -> anyhow::Result<()> {
        // let client = self
        //     .remote_endpoint
        //     .as_ref()
        //     .map(|url| Client::new(url.clone()));

        // Create directories for source and metadata.
        let mut source_dir = base_path.to_path_buf();
        source_dir.push("sources");

        std::fs::create_dir(source_dir.as_path())
            .map_err(|err| anyhow!("Fail to create folder for source: {:?}", err))?;

        let mut metadata_dir = base_path.to_path_buf();
        metadata_dir.push("metadata");

        std::fs::create_dir(metadata_dir.as_path())
            .map_err(|err| anyhow!("Fail to create folder for metadata: {:?}", err))?;

        // If we are generating multi-step proposal files, we generate the files in reverse order,
        // since we need to pass in the hash of the next file to the previous file.
        for proposal in &self.proposals {
            let mut proposal_dir = base_path.to_path_buf();
            proposal_dir.push("sources");
            proposal_dir.push(proposal.name.as_str());

            std::fs::create_dir(proposal_dir.as_path())
                .map_err(|err| anyhow!("Fail to create folder for proposal: {:?}", err))?;

            let mut result: Vec<(String, String)> = vec![];
            if let ExecutionMode::MultiStep = &proposal.execution_mode {
                for entry in proposal.update_sequence.iter().rev() {
                    entry.libra_generate_release_script( //////// 0L //////// our change to use the extension
                        // client.as_ref(),
                        &mut result,
                        proposal.execution_mode,
                        &framework_local_dir,
                    )?;
                }
                result.reverse();
            } else {
                for entry in proposal.update_sequence.iter() {
                    entry.libra_generate_release_script( //////// 0L //////// our change to use the extension
                        // client.as_ref(),
                        &mut result,
                        proposal.execution_mode,
                        &framework_local_dir,
                    )?;
                }
            }

            for (idx, (script_name, script)) in result.into_iter().enumerate() {
                let mut script_path = proposal_dir.clone();
                let proposal_name = format!("{}-{}", idx, script_name);
                script_path.push(&proposal_name);
                script_path.set_extension("move");

                let execution_hash = append_script_hash(script, script_path.clone(), framework_local_dir.clone());
                std::fs::write(&script_path, execution_hash.as_bytes())
                    .map_err(|err| anyhow!("Failed to write to file: {:?}", err))?;
            }

            let mut metadata_path = base_path.to_path_buf();
            metadata_path.push("metadata");
            metadata_path.push(proposal.name.as_str());
            metadata_path.set_extension("json");

            std::fs::write(
                metadata_path.as_path(),
                serde_json::to_string_pretty(&proposal.metadata)?,
            )
            .map_err(|err| anyhow!("Failed to write to file: {:?}", err))?;
        }

        Ok(())
    }
}

fn append_script_hash(raw_script: String, _script_path: PathBuf, framework_local_dir: PathBuf) -> String {
    let temp_script_path = TempPath::new();
    temp_script_path.create_as_file().unwrap();

    let mut move_script_path = temp_script_path.path().to_path_buf();
    move_script_path.set_extension("move");
    dbg!(&move_script_path);
    std::fs::write(move_script_path.as_path(), raw_script.as_bytes())
        .map_err(|err| {
            anyhow!(
                "Failed to get execution hash: failed to write to file: {:?}",
                err
            )
        })
        .unwrap();

    let (_, hash) = generate_hash(move_script_path, framework_local_dir).unwrap();

    format!("// Script hash: {} \n{}", hash, raw_script)
}


// 0L Note: this comes from a different module than release builder
// see crates/aptos/src/governance/mod.rs
/// Generate execution hash for a specified script.
// pub struct LibraGenerateExecutionHash {
//     pub script_path: Option<PathBuf>,
//     pub framework_local_dir: Option<PathBuf>,
// }

pub fn generate_hash(script_path: PathBuf, framework_local_dir: PathBuf) -> anyhow::Result<(Vec<u8>, HashValue)> {
    let res = CompileScriptFunction {
        script_path: Some(script_path),
        compiled_script_path: None,
        framework_package_args: FrameworkPackageArgs {
            framework_git_rev: None,
            framework_local_dir: Some(framework_local_dir),
            skip_fetch_latest_git_deps: false,
        },
        bytecode_version: None,
    }
    .compile("execution_hash", PromptOptions::yes())?;
    Ok(res)
}

pub fn libra_release_cfg_default() -> ReleaseConfig {
    ReleaseConfig {
        remote_endpoint: None,
        proposals: vec![
          Proposal {
              execution_mode: ExecutionMode::MultiStep,
              metadata: ProposalMetadata::default(),
              name: "framework".to_string(),
              update_sequence: vec![ReleaseEntry::Framework(FrameworkReleaseConfig {
                  bytecode_version: 6, // Only numbers 5 to 6 are supported
                  git_hash: None,
              })],
          }
        ],
    }
}