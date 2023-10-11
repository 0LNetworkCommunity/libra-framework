use std::path::PathBuf;

use diem_temppath::TempPath;
use diem_types::account_address::AccountAddress;
use libra_framework::framework_cli::make_template_files;

pub fn make_script(first_validator_address: AccountAddress) -> PathBuf {
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
        first_validator_address
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
