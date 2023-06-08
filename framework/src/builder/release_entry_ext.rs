//! extends ReleaseEntry with new methods with custom trait.
use crate::builder::framework_generate_upgrade_proposal;
use zapatos_release_builder::components::get_execution_hash;
use zapatos_release_builder::ExecutionMode;
use zapatos_release_builder::ReleaseEntry;
use zapatos_rest_client::Client;
pub trait LibraReleaseEntry {
  fn libra_generate_release_script (
    &self,
    client: Option<&Client>,
    result: &mut Vec<(String, String)>,
    execution_mode: ExecutionMode,
  ) -> anyhow::Result<()>;
}


impl LibraReleaseEntry for ReleaseEntry {
    fn libra_generate_release_script(
        &self,
        _client: Option<&Client>,
        result: &mut Vec<(String, String)>,
        execution_mode: ExecutionMode,
    ) -> anyhow::Result<()> {
        let (is_testnet, is_multi_step) = match execution_mode {
            ExecutionMode::MultiStep => (false, true),
            ExecutionMode::SingleStep => (false, false),
            ExecutionMode::RootSigner => (true, false),
        };
        match self {
            ReleaseEntry::Framework(framework_release) => {
                result.append(
                    &mut framework_generate_upgrade_proposal::generate_upgrade_proposals(
                        framework_release,
                        is_testnet,
                        if is_multi_step {
                            get_execution_hash(result)
                        } else {
                            "".to_owned().into_bytes()
                        },
                    )
                    .unwrap(),
                );
            },
            _ => todo!(),
            // ReleaseEntry::CustomGas(gas_schedule) => {
            //     if !fetch_and_equals::<GasScheduleV2>(client, gas_schedule)? {
            //         result.append(&mut gas::generate_gas_upgrade_proposal(
            //             gas_schedule,
            //             is_testnet,
            //             if is_multi_step {
            //                 get_execution_hash(result)
            //             } else {
            //                 "".to_owned().into_bytes()
            //             },
            //         )?);
            //     }
            // },
            // ReleaseEntry::DefaultGas => {
            //     let gas_schedule = aptos_gas::gen::current_gas_schedule();
            //     if !fetch_and_equals::<GasScheduleV2>(client, &gas_schedule)? {
            //         result.append(&mut gas::generate_gas_upgrade_proposal(
            //             &gas_schedule,
            //             is_testnet,
            //             if is_multi_step {
            //                 get_execution_hash(result)
            //             } else {
            //                 "".to_owned().into_bytes()
            //             },
            //         )?);
            //     }
            // },
            // ReleaseEntry::Version(version) => {
            //     if !fetch_and_equals::<Version>(client, version)? {
            //         result.append(&mut version::generate_version_upgrade_proposal(
            //             version,
            //             is_testnet,
            //             if is_multi_step {
            //                 get_execution_hash(result)
            //             } else {
            //                 "".to_owned().into_bytes()
            //             },
            //         )?);
            //     }
            // },
            // ReleaseEntry::FeatureFlag(feature_flags) => {
            //     let mut needs_update = true;
            //     if let Some(client) = client {
            //         let features = block_on(async {
            //             client
            //                 .get_account_resource_bcs::<aptos_types::on_chain_config::Features>(
            //                     CORE_CODE_ADDRESS,
            //                     "0x1::features::Features",
            //                 )
            //                 .await
            //         })?;
            //         // Only update the feature flags section when there's a divergence between the local configs and on chain configs.
            //         // If any flag in the release config diverges from the on chain value, we will emit a script that includes all flags
            //         // we would like to enable/disable, regardless of their current on chain state.
            //         needs_update = feature_flags.has_modified(features.inner());
            //     }
            //     if needs_update {
            //         result.append(&mut feature_flags::generate_feature_upgrade_proposal(
            //             feature_flags,
            //             is_testnet,
            //             if is_multi_step {
            //                 get_execution_hash(result)
            //             } else {
            //                 "".to_owned().into_bytes()
            //             },
            //         )?);
            //     }
            // },
            // ReleaseEntry::Consensus(consensus_config) => {
            //     if !fetch_and_equals(client, consensus_config)? {
            //         result.append(&mut consensus_config::generate_consensus_upgrade_proposal(
            //             consensus_config,
            //             is_testnet,
            //             if is_multi_step {
            //                 get_execution_hash(result)
            //             } else {
            //                 "".to_owned().into_bytes()
            //             },
            //         )?);
            //     }
            // },
            // ReleaseEntry::Execution(execution_config) => {
            //     if !fetch_and_equals(client, execution_config)? {
            //         result.append(
            //             &mut execution_config::generate_execution_config_upgrade_proposal(
            //                 execution_config,
            //                 is_testnet,
            //                 if is_multi_step {
            //                     get_execution_hash(result)
            //                 } else {
            //                     "".to_owned().into_bytes()
            //                 },
            //             )?,
            //         );
            //     }
            // },
            // ReleaseEntry::RawScript(script_path) => {
            //     let base_path =
            //         PathBuf::from(std::env!("CARGO_MANIFEST_DIR")).join(script_path.as_path());
            //     let file_name = base_path
            //         .file_name()
            //         .and_then(|name| name.to_str())
            //         .ok_or_else(|| {
            //             anyhow!("Unable to obtain file name for proposal: {:?}", script_path)
            //         })?
            //         .to_string();
            //     let file_content = std::fs::read_to_string(base_path)?;

            //     if let ExecutionMode::MultiStep = execution_mode {
            //         // Render the hash for multi step proposal.
            //         // {{ script_hash }} in the provided move file will be replaced with the real hash.

            //         let mut handlebars = Handlebars::new();
            //         handlebars
            //             .register_template_string("move_template", file_content.as_str())
            //             .unwrap();

            //         let execution_hash = get_execution_hash(result);
            //         let mut hash_string = "vector[".to_string();
            //         for b in execution_hash.iter() {
            //             hash_string.push_str(format!("{}u8,", b).as_str());
            //         }
            //         hash_string.push(']');

            //         let mut data = HashMap::new();
            //         data.insert("script_hash", hash_string);

            //         result.push((
            //             file_name,
            //             handlebars
            //                 .render("move_template", &data)
            //                 .map_err(|err| anyhow!("Fail to render string: {:?}", err))?,
            //         ));
            //     } else {
            //         result.push((file_name, file_content));
            //     }
            // },
        }
        Ok(())
    }
}