//! generate some artifacts so we can test governance framework upgrades

use std::path::PathBuf;
use std::str::FromStr;

use crate::builder::framework_generate_upgrade_proposal::make_framework_upgrade_artifacts;
use anyhow::Context;

use std::sync::Once;

static INIT: Once = Once::new();

/// testsuite helper to create fixtures before the testsuite runs.
pub fn testsuite_maybe_warmup_fixtures() {
    INIT.call_once(|| {
        // TODO: decide how to force rebuild, or never rebuild. Maybe envvar? or
        // using build.rs

        let fixture_path = fixtures_path();
        let p = fixture_path.join("upgrade-single-lib");
        if p.exists() { // don't regenerate
            return;
        }
        // initialization code here
        upgrade_fixtures().expect("could no warmup upgrade fixtures");
    });
}


/// where we store upgrade fixtures
pub fn fixtures_path() -> PathBuf {
    let this_crate = PathBuf::from_str(env!("CARGO_MANIFEST_DIR")).unwrap();
    this_crate
        .join("src")
        .join("upgrade_fixtures")
        .join("fixtures")
}

/// add the all_your_base.move file which serves as the canary to test if the
/// upgrade went through
pub fn insert_test_file(core_module_name: &str, remove: bool) -> anyhow::Result<()> {
    //1. Copy the allyourbase code to the module.
    let this_crate = PathBuf::from_str(env!("CARGO_MANIFEST_DIR")).unwrap();
    let core_module_sources = this_crate
        .parent()
        .unwrap()
        .join("framework")
        .join(core_module_name)
        .join("sources");
    assert!(
        core_module_sources.exists(),
        "cannot find sources for: {}",
        core_module_name
    );

    let away_file_path = core_module_sources.join("all_your_base.move");
    if remove {
        std::fs::remove_file(away_file_path)?;
        return Ok(());
    }

    let file_path = this_crate
        // .join("src")
        .join("src")
        .join("upgrade_fixtures")
        .join("fixtures")
        .join("all_your_base.move");

    std::fs::copy(file_path, away_file_path)?;

    Ok(())
}

// like is sounds
pub fn generate_fixtures(output_path: PathBuf, modules: Vec<String>, force_incompatible_upgrade: bool) -> anyhow::Result<()> {
    println!("generating files, this will take some time, go do some laundry");
    let destination_module = modules.last().unwrap().clone();
    insert_test_file(&destination_module, false).context("could not insert test file")?;

    let this_crate = PathBuf::from_str(env!("CARGO_MANIFEST_DIR"))?;
    let libra_framework_sources = this_crate
        .parent()
        .context("no parent dir")?
        .join("framework");

    make_framework_upgrade_artifacts(
        &output_path,
        &libra_framework_sources,
        &Some(modules),
        force_incompatible_upgrade,
    )?;
    // ok, cleanup
    insert_test_file(&destination_module, true)?;

    Ok(())
}

#[ignore]
// KEEP THIS TEST HERE TO HELP REGENERATE FIXTURES
#[test]
fn make_the_upgrade_fixtures() -> anyhow::Result<()> {
    upgrade_fixtures()
}

/// helper for testing, so that we have fresh fixtures at every run.
pub fn upgrade_fixtures() -> anyhow::Result<()> {
    let fixture_path = fixtures_path();

    // for single step upgrades
    // places the all_your_base in the move-stdlib dir
    let p = fixture_path.join("upgrade-single-lib");
    std::fs::create_dir_all(&p)?;
    let modules = vec!["move-stdlib".to_string()];
    generate_fixtures(p, modules, false)?;

    // for multi step upgrades
    // places the all_your_base in the libra_framework dir
    let p = fixture_path.join("upgrade-multi-lib");
    std::fs::create_dir_all(&p)?;
    let modules = vec![
        "move-stdlib".to_string(),
        "vendor-stdlib".to_string(),
        "libra-framework".to_string(),
    ];

    generate_fixtures(p, modules, false)?;

    // generate fixtures with arbitrary release
    let p = fixture_path.join("upgrade-multi-lib-force");
    std::fs::create_dir_all(&p)?;
    let modules = vec![
        "move-stdlib".to_string(),
        "vendor-stdlib".to_string(),
        "libra-framework".to_string(),
    ];

    generate_fixtures(p, modules, true)?;


    Ok(())
}
