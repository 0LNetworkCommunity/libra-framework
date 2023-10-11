use diem_types::{
    account_address::AccountAddress,
    account_config::reserved_vm_address,
    transaction::{Script, WriteSetPayload},
};
use move_compiler::{compiled_unit::AnnotatedCompiledUnit, Compiler, Flags};
use std::path::Path;

/// compiles a script for execution on the db offline.
// TODO: not sure if this is duplicated with the framework upgrade code in ./framework
pub fn compile_script(source_file_str: &Path, bytecode_version: Option<u32>) -> Vec<u8> {
    let (_files, mut compiled_program) = Compiler::from_files(
        vec![source_file_str.to_str().unwrap().to_owned()],
        libra_framework::head_release_bundle().files().unwrap(),
        libra_framework::release::named_addresses().clone(),
    )
    .set_flags(Flags::empty().set_sources_shadow_deps(false))
    .build_and_report()
    .unwrap();
    assert!(compiled_program.len() == 1);
    match compiled_program.pop().unwrap() {
        AnnotatedCompiledUnit::Module(_) => panic!("Unexpected module when compiling script"),
        x @ AnnotatedCompiledUnit::Script(_) => x.into_compiled_unit().serialize(bytecode_version),
    }
}

/// create a custom script payload from a link to .move file
/// note that there are no Move args or types passed in this helper
///  see /templates for examples of rescue scripts
pub fn custom_script(
    script_path: &Path,
    execute_as: Option<AccountAddress>,
    bytecode_version: Option<u32>,
) -> WriteSetPayload {
    WriteSetPayload::Script {
        script: Script::new(
            compile_script(script_path, bytecode_version),
            vec![],
            vec![],
        ),
        execute_as: execute_as.unwrap_or_else(reserved_vm_address),
    }
}
