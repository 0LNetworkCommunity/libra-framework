#![allow(dead_code)]
use std::{
    path::PathBuf,
    process::Command,
    time::{Duration, Instant},
};

use anyhow::Context;
use diem_config::config::NodeConfig;
use diem_forge::{LocalNode, NodeExt, Validator};
use diem_logger::info;
use diem_temppath::TempPath;
use diem_types::account_address::AccountAddress;
use libra_framework::framework_cli::make_template_files;
use smoke_test::test_utils::{MAX_CONNECTIVITY_WAIT_SECS, MAX_HEALTHY_WAIT_SECS};

pub fn make_script(remove_validator: AccountAddress) -> PathBuf {
    let script = format!(
        r#"
        script {{
            use diem_framework::stake;
            use diem_framework::diem_governance;
            use diem_framework::block;

            fun main(vm_signer: &signer, framework_signer: &signer) {{
                stake::remove_validators(framework_signer, &vector[@0x{:?}]);
                block::emit_writeset_block_event(vm_signer, @0x1);
                diem_governance::reconfigure(framework_signer);
            }}
    }}
    "#,
        remove_validator
    );

    let framework_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("framework")
        .join("libra-framework");

    let mut temp_script_path = TempPath::new();
    temp_script_path.create_as_dir().unwrap();
    temp_script_path.persist();

    assert!(temp_script_path.path().exists());

    make_template_files(
        temp_script_path.path(),
        &framework_path,
        "rescue",
        Some(script),
    )
    .unwrap();

    temp_script_path.path().to_owned()
}


pub fn make_script_exp() -> PathBuf {
    let script = format!(
        r#"
        script {{
            use diem_framework::reconfiguration;

            fun main(vm_signer: &signer, _framework_signer: &signer) {{
                reconfiguration::emit_epoch(vm_signer);
            }}
        }}
    "#,
        remove_validator
    );

    let framework_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("framework")
        .join("libra-framework");

    let mut temp_script_path = TempPath::new();
    temp_script_path.create_as_dir().unwrap();
    temp_script_path.persist();

    assert!(temp_script_path.path().exists());

    make_template_files(
        temp_script_path.path(),
        &framework_path,
        "rescue",
        Some(script),
    )
    .unwrap();

    temp_script_path.path().to_owned()
}

pub fn deadline_secs(secs: u64) -> Instant {
    Instant::now()
        .checked_add(Duration::from_secs(secs))
        .expect("no deadline")
}

pub fn update_node_config_restart(
    validator: &mut LocalNode,
    mut config: NodeConfig,
) -> anyhow::Result<()> {
    validator.stop();
    let node_path = validator.config_path();
    config.save_to_path(node_path)?;
    validator.start()?;
    Ok(())
}

pub async fn wait_for_node(
    validator: &mut dyn Validator,
    expected_to_connect: usize,
) -> anyhow::Result<()> {
    let healthy_deadline = Instant::now()
        .checked_add(Duration::from_secs(MAX_HEALTHY_WAIT_SECS))
        .context("no deadline")?;
    validator
        .wait_until_healthy(healthy_deadline)
        .await
        .unwrap_or_else(|err| {
            let lsof_output = Command::new("lsof").arg("-i").output().unwrap();
            panic!(
                "wait_until_healthy failed. lsof -i: {:?}: {}",
                lsof_output, err
            );
        });
    info!("Validator restart health check passed");

    let connectivity_deadline = Instant::now()
        .checked_add(Duration::from_secs(MAX_CONNECTIVITY_WAIT_SECS))
        .context("can't get new deadline")?;
    validator
        .wait_for_connectivity(expected_to_connect, connectivity_deadline)
        .await?;
    info!("Validator restart connectivity check passed");
    Ok(())
}
