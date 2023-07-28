//! generate some artifacts so we can test governance framework upgrades

use std::path::PathBuf;
use std::str::FromStr;

use libra_framework::builder::framework_generate_upgrade_proposal::make_framework_upgrade_artifacts;


// TODO make this a once_cell

pub fn fixtures_path() -> PathBuf {
    let this_crate = PathBuf::from_str(env!("CARGO_MANIFEST_DIR")).unwrap();
    this_crate.join("src").join("tests").join("fixtures")
}


pub fn insert_test_file(core_module_name: &str, remove: bool) -> anyhow::Result<()>{
  //1. Copy the allyourbase code to the module.
  let this_crate = PathBuf::from_str(env!("CARGO_MANIFEST_DIR")).unwrap();
  let core_module_sources = this_crate.parent().unwrap().join("framework").join(core_module_name).join("sources");
  dbg!(&core_module_sources);
  let away_file_path = core_module_sources.join("all_your_base.move");
  if remove {

    std::fs::remove_file(away_file_path)?;
    return Ok(())
  }

  let file_path = this_crate.join("src").join("tests").join("fixtures").join("all_your_base.move");
  dbg!(&file_path);

  std::fs::copy(file_path, away_file_path)?;

  Ok(())
}


pub fn generate_fixtures(output_path: PathBuf, modules: Vec<String>) -> anyhow::Result<()> {

  println!("generating files, this will take some time, go do some laundry");
  let destination_module = modules.last().unwrap().clone();
  insert_test_file(&destination_module, false)?;

  let this_crate = PathBuf::from_str(env!("CARGO_MANIFEST_DIR")).unwrap();
  dbg!(&this_crate);
  let libra_framework_sources = this_crate.parent().unwrap().join("framework");

  dbg!(&libra_framework_sources);

  make_framework_upgrade_artifacts(&output_path, &libra_framework_sources, &Some(modules))?;

  dbg!("ok");

  // ok, cleanup
  insert_test_file(&destination_module, true)?;

  Ok(())
}


#[ignore]
// KEEP THIS TEST HERE TO HELP REGENERATE FIXTURES
#[test]
fn make_the_upgrade_fixtures() -> anyhow::Result<()>{

    let fixture_path = fixtures_path();

    let p = fixture_path.join("upgrade_single_step");
    std::fs::create_dir_all(&p)?;
    dbg!(&p);
    let modules = vec!["move-stdlib".to_string()];

    generate_fixtures(p.clone(), modules)?;

    let p = fixture_path.join("upgrade_multi_step");
    std::fs::create_dir_all(&p)?;
    let modules = vec!["move-stdlib".to_string(), "vendor-stdlib".to_string()];

    generate_fixtures(p, modules)?;
    Ok(())
}

