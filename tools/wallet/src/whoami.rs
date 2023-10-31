use crate::keys::make_validator_keys;
use dialoguer::Confirm;


    // given a mnemonic what are all the settings which could be expected
  pub fn who_am_i(show_validator: bool) -> anyhow::Result<()> {
    let keep_legacy_address = Confirm::new()
      .with_prompt("Is this a legacy (v5 or prior) address?")
      .interact()?;

    // NOTE: we use the validator keygen so that we can optionally show that
    // info
    // the owner key will derive to the same.
    let (_validator_blob, _vfn_blob, _private_identity, public_identity, _legacy_keys) =
      make_validator_keys(None, keep_legacy_address)?;

    if show_validator {
      println!("validator public credentials:");
      println!("{}", serde_json::to_string_pretty(&public_identity).unwrap());
    } else {
      println!("owner address: {}", &public_identity.account_address);
      println!("owner authentication key: {}", &public_identity.account_public_key);
    }

    Ok(())
  }