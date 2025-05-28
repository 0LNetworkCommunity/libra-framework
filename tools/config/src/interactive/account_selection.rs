use anyhow::Context;
use libra_types::exports::{AccountAddress, AuthenticationKey};
use libra_wallet::account_keys::{get_ol_legacy_address, AccountKeys};

/// Interactive account selection - allows user to choose between existing address, mnemonic, or dummy account
pub fn interactive_account_selection() -> anyhow::Result<(AuthenticationKey, AccountAddress)> {
    // Ask the user if they want to include an account address in profile
    if dialoguer::Confirm::new()
        .with_prompt("Do you want to configure with an account address in this profile?")
        .interact()?
    {
        // User wants to configure with a real account
        println!("You can either provide an existing address or derive one from a mnemonic.");

        let use_existing = dialoguer::Confirm::new()
            .with_prompt("Do you want to enter an address? If not, will derive from mnemonic.")
            .interact()?;

        if use_existing {
            // User wants to provide an existing address
            let address_input: String = dialoguer::Input::new()
                .with_prompt("Enter your account address (hex format)")
                .interact()?;

            let address = AccountAddress::from_hex_literal(&address_input)
                .context("Invalid account address format")?;

            // Use dummy authkey for existing addresses since we don't control them
            let dummy_authkey = AuthenticationKey::zero();

            Ok((dummy_authkey, address))
        } else {
            // User wants to derive from mnemonic
            let account_keys = prompt_for_account()?;
            Ok((account_keys.auth_key, account_keys.account))
        }
    } else {
        // User doesn't want to configure with account, use dummy values
        println!("Using dummy account values (0x0) - you can configure a real account later.");
        let dummy_address = AccountAddress::ZERO;
        let dummy_authkey = AuthenticationKey::zero();

        Ok((dummy_authkey, dummy_address))
    }
}

/// Wrapper on get keys_from_prompt,
/// Prompts the user for account details and checks if it is a legacy account.
pub fn prompt_for_account() -> anyhow::Result<AccountKeys> {
    let mut account_keys = libra_wallet::account_keys::get_keys_from_prompt()?.child_0_owner;

    if dialoguer::Confirm::new()
        .with_prompt("Is this a legacy pre-v7 address (16 characters)?")
        .interact()?
    {
        account_keys.account = get_ol_legacy_address(account_keys.account)?;
    }

    Ok(account_keys)
}
