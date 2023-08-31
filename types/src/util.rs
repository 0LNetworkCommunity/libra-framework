use anyhow::{bail, Context, Result};
use diem_sdk::{
    move_types::identifier::Identifier,
    types::{account_address::AccountAddress, transaction::SignedTransaction},
};
use std::fmt::{Debug, Display};

pub fn format_signed_transaction(signed_trans: &SignedTransaction) -> String {
    let mut raw_trans = signed_trans
        .to_owned()
        .into_raw_transaction()
        .format_for_client(|_| String::new())
        .replace('\t', "    ");

    if let Some(index) = raw_trans.rfind('\n') {
        raw_trans.replace_range(index.., "\n}");
    }

    let authenticator =
        format!("{:#?}", signed_trans.authenticator()).replace("Ed25519 {", "Authenticator {");

    format!("{raw_trans}\n{authenticator}")
}

pub fn format_type_args<T: Display>(type_args: &[T]) -> String {
    format!(
        "Type Arguments: [{}]",
        type_args
            .iter()
            .map(|t| t.to_string())
            .collect::<Vec<_>>()
            .join(", ")
    )
}

pub fn format_args<T: Debug>(args: &[T]) -> String {
    format!("Arguments: {args:#?}")
}

pub fn parse_function_id(function_id: &str) -> Result<(AccountAddress, Identifier, Identifier)> {
    let id_parts = function_id
        .split("::")
        .map(|i| i.trim())
        .collect::<Vec<_>>();
    if id_parts.len() != 3 {
        bail!("Invalid function id: {function_id}");
    }
    let module_address = AccountAddress::from_hex_literal(id_parts[0])
        .context(format!("Failed to parse module address: {}", id_parts[0]))?;
    let module_name = Identifier::new(id_parts[1])
        .context(format!("Failed to parse module name: {}", id_parts[1]))?;
    let function_name = Identifier::new(id_parts[2])
        .context(format!("Failed to parse function name: {}", id_parts[2]))?;
    Ok((module_address, module_name, function_name))
}
