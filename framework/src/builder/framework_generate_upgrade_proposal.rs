//! generate framework upgrade proposal scripts
//! see vendor aptos-move/framework/src/release_bundle.rs

use anyhow::Result;
use zapatos_framework::{BuildOptions, BuiltPackage, ReleasePackage};
use zapatos_temppath::TempPath;
use zapatos_types::account_address::AccountAddress;
use std::path::{Path, PathBuf};
// use zapatos_release_builder::components::get_execution_hash;
use zapatos_crypto::HashValue;

use crate::builder::framework_release_bundle::libra_generate_script_proposal_impl;
// use zapatos_release_builder::components::framework::FrameworkReleaseConfig;

pub fn make_framework_upgrade_artifacts(
    // config: &FrameworkReleaseConfig,
    // is_testnet: bool,
    // next_execution_hash: Vec<u8>,
    framework_local_dir: &Path,
) -> Result<Vec<(String, String)>> {
    let mut next_execution_hash = vec![];

    // 0L TODO: don't make this hard coded
    let mut package_path_list = vec![
        // ("0x1", "move-stdlib"),
        // ("0x1", "vendor-stdlib"),
        ("0x1", "libra-framework"),
        // ("0x3", "aptos-move/framework/aptos-token"),
        // ("0x4", "aptos-move/framework/aptos-token-objects"),
    ];

    let mut result: Vec<(String, String)> = vec![];

    // let temp_root_path = TempPath::new();
    // temp_root_path.create_as_dir()?;

    let commit_info =  zapatos_build_info::get_git_hash();

    // For generating multi-step proposal files, we need to generate them in the reverse order since
    // we need the hash of the next script.
    // We will reverse the order back when writing the files into a directory.
    if !next_execution_hash.is_empty() {
        package_path_list.reverse();
    }

    // let mut root_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).to_path_buf();

    for (publish_addr, relative_package_path) in package_path_list.iter() {
        let account = AccountAddress::from_hex_literal(publish_addr)?;
        let temp_script_path = TempPath::new();
        temp_script_path.create_as_file()?;
        let mut move_script_path = temp_script_path.path().to_path_buf();
        move_script_path.set_extension("move");

        // let mut package_path = if config.git_hash.is_some() {
        //     temp_root_path.path().to_path_buf()
        // } else {
        //     framework_local_dir.to_owned().canonicalize()?
        // };

        let mut package_path = framework_local_dir.to_owned().canonicalize()?;

        package_path.push(relative_package_path);

        let script_name = package_path
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();

        // If this file is the first framework file being generated (if `result.is_empty()` is true),
        // its `next_execution_hash` should be the `next_execution_hash` value being passed in.
        // If the `result` vector is not empty, the current file's `next_execution_hash` should be the
        // hash of the latest framework file being generated (the hash of result.last()).
        // For example, let's say we are going to generate these files:
        // 0-move-stdlib.move	2-aptos-framework.move	4-gas-schedule.move	6-features.move
        // 1-aptos-stdlib.move	3-aptos-token.move	5-version.move		7-consensus-config.move
        // The first framework file being generated is 3-aptos-token.move. It's using the next_execution_hash being passed in (so in this case, the hash of 4-gas-schedule.move being passed in mod.rs).
        // The second framework file being generated would be 2-aptos-framework.move, and it's using the hash of 3-aptos-token.move (which would be result.last()).

        let options = BuildOptions {
            with_srcs: true,
            with_abis: false,
            with_source_maps: false,
            with_error_map: true,
            skip_fetch_latest_git_deps: false,
            bytecode_version: Some(6),
            ..BuildOptions::default()
        };

        assert!(package_path.exists(), "package path does not exist at {}", package_path.to_str().unwrap());

        let package = BuiltPackage::build(package_path.clone(), options)?;
        let release = ReleasePackage::new(package)?;

        // 0L only uses multi-step proposal workflow
          // let next_execution_hash_bytes = if result.is_empty() {
          //   next_execution_hash.clone()
          // } else {
          //     get_execution_hash(&result)
          // };

          // release.generate_script_proposal_multi_step(
          //     account,
          //     move_script_path.clone(),
          //     next_execution_hash_bytes,
          // )?;

          libra_generate_script_proposal_impl(&release, account, move_script_path.clone())?;

          let (_, hash) = libra_compile_script(&package_path)?;
          next_execution_hash = hash.to_vec();
        // next_execution_hash_bytes
        // // If we're generating a single-step proposal on testnet
        // if is_testnet && next_execution_hash.is_empty() {
        //     release.generate_script_proposal_testnet(account, move_script_path.clone())?;
        //     // If we're generating a single-step proposal on mainnet
        // } else if next_execution_hash.is_empty() {
        //     release.generate_script_proposal(account, move_script_path.clone())?;
        //     // If we're generating a multi-step proposal
        // } else {
        //     let next_execution_hash_bytes = if result.is_empty() {
        //         next_execution_hash.clone()
        //     } else {
        //         get_execution_hash(&result)
        //     };
        //     release.generate_script_proposal_multi_step(
        //         account,
        //         move_script_path.clone(),
        //         next_execution_hash_bytes,
        //     )?;

        //     // generate_script_proposal_impl(for_address, out, true, true, next_execution_hash)
        // };

        let mut script = format!(
            "// Framework commit hash: {}\n// Builder commit hash: {}\n",
            commit_info,
            zapatos_build_info::get_git_hash()
        );

        script.push_str(&std::fs::read_to_string(move_script_path.as_path())?);

        result.push((script_name, script));
    }
    Ok(result)
}



pub fn write_to_file(result: Vec<(String, String)>, proposal_dir: PathBuf) -> anyhow::Result<()>{
      println!("writing upgrade scripts to folder");

    for (idx, (script_name, script)) in result.into_iter().enumerate() {
        let mut script_path = proposal_dir.clone();
        let proposal_name = format!("{}-{}", idx, script_name);
        script_path.push(&proposal_name);
        script_path.set_extension("move");

        // let execution_hash = append_script_hash(script, script_path.clone(), framework_local_dir.clone())?;
        std::fs::write(&script_path, script.as_bytes())?;
    }
    Ok(())
}
// /Users/lucas/code/rust/zapatos/crates/aptos/src/move_tool/mod.rs
/// Need to create a dummy package so that we can build the script into bytecode
/// so that we can then get the hash of the script.
/// ... so that we can then submit it as part of a proposal framework/libra-framework/sources/modified_source/aptos_governance.move
/// ... so that then the VM doesn't complain about its size /aptos-move/aptos-vm/src/aptos_vm_impl.rs
/// ... and so that when the proposal is approved a third party can execute the source upgrade.

pub fn init_move_dir_wrapper(package_dir: PathBuf, script_name: &str, framework_local_dir: PathBuf) -> anyhow::Result<()>{
  zapatos::move_tool::init_move_dir_generic(
    &package_dir,
    script_name,
    "LibraFramework".to_string(),
    std::fs::canonicalize(&framework_local_dir)?,
  )?;
  Ok(())
}

pub fn libra_compile_script(
    // skip_fetch_latest_git_deps: bool,
    package_dir: &Path,
    // bytecode_version: Option<u32>,
) -> Result<(Vec<u8>, HashValue)> {
    println!("compiling libra script...");
    dbg!(&package_dir);
    // these are the options only for the upgrade SCRIPT
    // the payload needs to be small, because even approved TX scripts have
    // an upperbound in the transaction admission.
    let build_options = BuildOptions {
        with_srcs: false,
        with_abis: false,
        with_source_maps: false,
        with_error_map: false,
        skip_fetch_latest_git_deps: true,
        bytecode_version: Some(6),
        ..BuildOptions::default()
    };

    let pack = BuiltPackage::build(package_dir.to_path_buf(), build_options)?;
        // .map_err(|e| CliError::MoveCompilationError(format!("{:#}", e)))?;

    let scripts_count = pack.script_count();

    if scripts_count != 1 {
        anyhow::bail!(
            "Only one script can be prepared a time. Make sure one and only one script file \
                is included in the Move package. Found {} scripts.",
            scripts_count
        );
    }

    let bytes = pack.extract_script_code().pop().unwrap();
    let hash = HashValue::sha3_256_of(bytes.as_slice());
    Ok((bytes, hash))
}