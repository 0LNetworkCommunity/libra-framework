use super::submit_transaction::Sender;
use anyhow::Context;
use diem_sdk::move_types::{
    language_storage::{ModuleId, TypeTag},
    parser::{parse_transaction_arguments, parse_type_tags},
    transaction_argument::convert_txn_args,
};
use diem_types::transaction::{EntryFunction, TransactionArgument, TransactionPayload};
use libra_types::util::parse_function_id;

impl Sender {
    pub async fn generic(
        &mut self,
        function_id: &str,
        ty_args: &Option<String>,
        args: &Option<String>,
    ) -> anyhow::Result<()> {
        // TODO: should return a UserTransaction as does transfer.rs

        let payload =
            TransactionPayload::EntryFunction(build_entry_function(function_id, ty_args, args)?);

        self.sign_submit_wait(payload).await?;
        Ok(())
    }
}

pub fn build_entry_function(
    function_id: &str,
    ty_args: &Option<String>,
    args: &Option<String>,
) -> anyhow::Result<EntryFunction> {
    let (module_address, module_name, function_name) = parse_function_id(function_id)?;
    let module = ModuleId::new(module_address, module_name);
    let ty_args: Vec<TypeTag> = if let Some(ty_args) = ty_args {
        parse_type_tags(ty_args)
            .context(format!("Unable to parse the type argument(s): {ty_args}"))?
    } else {
        vec![]
    };
    let args: Vec<TransactionArgument> = if let Some(args) = args {
        parse_transaction_arguments(args).context(format!("Unable to parse argument(s): {args}"))?
    } else {
        vec![]
    };

    let entry = EntryFunction::new(module, function_name, ty_args, convert_txn_args(&args));

    Ok(entry)
}
