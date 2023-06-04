use zapatos_types::account_config::CORE_CODE_ADDRESS;
use zapatos_vm::{
    move_vm_ext::SessionExt,
};

use move_core_types::{
    resolver::MoveResolver,
        value::{serialize_values, MoveValue},

};

use zapatos_vm_genesis::{Validator, exec_function};


pub fn create_and_initialize_validators( //////// 0L ////////
    session: &mut SessionExt<impl MoveResolver>,
    validators: &[Validator],
) {
    let validators_bytes = bcs::to_bytes(validators).expect("Validators can be serialized");
    let mut serialized_values = serialize_values(&vec![MoveValue::Signer(CORE_CODE_ADDRESS)]);
    serialized_values.push(validators_bytes);
    exec_function(
        session,
        "genesis_migration",
        "migrate_legacy_user",
        vec![],
        serialized_values,
    );
}