// Copyright © Diem Foundation
// Parts of the project are originally copyright © Meta Platforms, Inc.
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use diem_types::{
    account_address::AccountAddress,
    account_config::diem_test_root_address,
    transaction::{Script, WriteSetPayload},
};
use handlebars::Handlebars;
use move_compiler::{compiled_unit::AnnotatedCompiledUnit, Compiler, Flags};
use serde::Serialize;
use std::{collections::HashMap, io::Write, path::{PathBuf, Path}};
use tempfile::NamedTempFile;

/// The relative path to the scripts templates
pub const SCRIPTS_DIR_PATH: &str = "templates";

pub fn compile_script(source_file_str: String, bytecode_version: Option<u32>) -> Vec<u8> {
    let (_files, mut compiled_program) = Compiler::from_files(
        vec![source_file_str],
        libra_framework::head_release_bundle()
            .files()
            .unwrap(),
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

fn compile_admin_script(input: &str, execute_as: Option<AccountAddress>, bytecode_version: Option<u32>) -> Result<Script> {
    let mut temp_file = NamedTempFile::new()?;
    temp_file.write_all(input.as_bytes())?;
    let cur_path = temp_file.path().to_str().unwrap().to_owned();
    Ok(Script::new(
        compile_script(cur_path, bytecode_version),
        vec![],
        vec![],
    ))
}

pub fn custom_script(script_path: &Path, execute_as: Option<AccountAddress>,bytecode_version: Option<u32>) -> WriteSetPayload {

    WriteSetPayload::Script {
        script: Script::new(
            compile_script(script_path.to_str().unwrap().to_owned(), bytecode_version),
            vec![],
            vec![],
        ),
        execute_as: execute_as.unwrap_or_else(diem_test_root_address),
    }
}
