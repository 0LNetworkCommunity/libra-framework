use anyhow::{anyhow, Result};
use libra_types::{
    core_types::app_cfg::{self, AppCfg},
    exports::{Client, NamedChain},
    type_extensions::client_ext::ClientExt,
};

use crate::config_wizard;

/// Reset address profile by prompting for account details
pub async fn reset_address_profile(
    cfg: &mut AppCfg,
    chain_name: Option<NamedChain>,
) -> Result<&mut app_cfg::Profile> {
    let (authkey, mut account_address) = config_wizard::interactive_account_selection()?;

    let client = Client::new(cfg.pick_url(chain_name)?);

    // Lookup originating address if client index is successful and we have a real authkey
    if client.get_index().await.is_ok()
        && authkey != libra_types::exports::AuthenticationKey::zero()
    {
        account_address = match client.lookup_originating_address(authkey).await {
            Ok(a) => a,
            Err(_) => {
                println!("INFO: could not find this address or authkey on chain. Maybe it has never been initialized? Have someone send a transaction to it.");
                account_address
            }
        };
    };

    // Create profile based on account keys
    let profile = app_cfg::Profile::new(authkey, account_address);

    // Add profile to configuration
    cfg.maybe_add_profile(profile)?;

    // Prompt to set as default profile
    if dialoguer::Confirm::new()
        .with_prompt("set as default profile?")
        .interact()?
    {
        cfg.workspace.set_default(account_address.to_hex_literal());
    }

    cfg.get_profile_mut(Some(account_address.to_hex_literal()))
}

/// Remove a profile from the configuration
pub fn remove_profile(cfg: &mut AppCfg, profile_identifier: &str) {
    // First try by the identifier directly (could be nickname or address)
    let result = cfg.try_remove_profile(profile_identifier);

    // If that fails, try to find by account address directly
    if result.is_err() {
        // Find the profile index by direct account comparison
        if let Some(index) = cfg.user_profiles.iter().position(|p| {
            p.account.to_hex_literal() == profile_identifier
                || p.account.to_string() == profile_identifier
        }) {
            cfg.user_profiles.remove(index);
            println!("Profile removed successfully!");
            return;
        }
    }

    if result.is_err() {
        println!("no profile found matching {}", profile_identifier);
    } else {
        println!("Profile removed successfully!");
    }
}

/// Interactive profile removal
pub fn interactive_remove_profile(cfg: &mut AppCfg) -> Result<()> {
    let profile_options: Vec<String> = cfg
        .user_profiles
        .iter()
        .map(|p| {
            let is_default = cfg
                .workspace
                .default_profile
                .as_ref()
                .map(|default| default == &p.account.to_hex_literal())
                .unwrap_or(false);

            let default_marker = if is_default { " (default)" } else { "" };
            format!("{} ({}){}", p.nickname, p.account, default_marker)
        })
        .collect();

    let selection = dialoguer::Select::new()
        .with_prompt("Which profile would you like to remove?")
        .items(&profile_options)
        .interact()?;

    let profile_to_remove = &cfg.user_profiles[selection];
    let profile_id = profile_to_remove.account.to_hex_literal();

    if dialoguer::Confirm::new()
        .with_prompt(format!(
            "Are you sure you want to remove profile '{}' ({})?",
            profile_to_remove.nickname, profile_to_remove.account
        ))
        .interact()?
    {
        remove_profile(cfg, &profile_id);
        println!("Profile removed successfully!");
    } else {
        println!("Profile removal cancelled.");
    }

    Ok(())
}

/// Interactive edit for a single profile (with option to add new profile)
pub async fn interactive_edit_single_profile(
    cfg: &mut AppCfg,
    chain_name: Option<NamedChain>,
) -> Result<bool> {
    let current_profile = &cfg.user_profiles[0];
    println!(
        "\nCurrent profile: {} ({})",
        current_profile.nickname, current_profile.account
    );

    println!("\nWhat would you like to do?");

    let options = vec![
        "Reset account address from mnemonic",
        "Add a new profile",
        "Exit without making changes",
    ];

    let selection = dialoguer::Select::new()
        .with_prompt("Choose an option")
        .items(&options)
        .default(0)
        .interact()?;

    match selection {
        0 => {
            // Reset address for current profile
            let _profile = reset_address_profile(cfg, chain_name).await?;
            println!("Address reset successfully!");
            Ok(true)
        }
        1 => {
            // Add new profile
            add_new_profile(cfg, chain_name).await?;
            Ok(true)
        }
        2 => {
            // Exit
            Ok(false)
        }
        _ => unreachable!(),
    }
}

/// Interactive profile management menu (for multiple profiles)
pub async fn interactive_profile_management(
    cfg: &mut AppCfg,
    chain_name: Option<NamedChain>,
) -> Result<bool> {
    println!("\nProfile Management");

    let options = vec![
        "Add a new profile",
        "Change default profile",
        "Edit a profile",
        "Remove a profile",
        "Exit without making changes",
    ];

    let selection = dialoguer::Select::new()
        .with_prompt("Choose an option")
        .items(&options)
        .default(0)
        .interact()?;

    match selection {
        0 => {
            // Add new profile
            add_new_profile(cfg, chain_name).await?;
            Ok(true)
        }
        1 => {
            // Change default profile
            super::defaults::change_default_profile(cfg)?;
            Ok(true)
        }
        2 => {
            // Edit a profile
            edit_profile(cfg, chain_name).await
        }
        3 => {
            // Remove profile
            interactive_remove_profile(cfg)?;
            Ok(true)
        }
        4 => {
            // Exit
            Ok(false)
        }
        _ => unreachable!(),
    }
}

/// Interactive profile management based on number of profiles
pub async fn interactive_manage_profiles(
    cfg: &mut AppCfg,
    chain_name: Option<NamedChain>,
) -> Result<bool> {
    use std::cmp::Ordering;

    match cfg.user_profiles.len().cmp(&1) {
        Ordering::Equal => {
            // Single profile: Show edit profile menu with option to add new profile
            interactive_edit_single_profile(cfg, chain_name).await
        }
        Ordering::Greater => {
            // Multiple profiles: Show profile management menu
            interactive_profile_management(cfg, chain_name).await
        }
        Ordering::Less => {
            // No profiles: Show basic options
            println!("\nNo profiles found. What would you like to do?");

            let options = vec!["Add a new profile", "Cancel"];

            let selection = dialoguer::Select::new()
                .with_prompt("Choose an option")
                .items(&options)
                .default(0)
                .interact()?;

            match selection {
                0 => {
                    add_new_profile(cfg, chain_name).await?;
                    Ok(true)
                }
                1 => {
                    Ok(false)
                }
                _ => unreachable!(),
            }
        }
    }
}

/// Add a new profile to the configuration
pub async fn add_new_profile(cfg: &mut AppCfg, chain_name: Option<NamedChain>) -> Result<()> {
    println!("\nAdding a new profile...");
    let _profile = reset_address_profile(cfg, chain_name).await?;
    println!("New profile added successfully!");
    Ok(())
}

/// Edit a specific profile
pub async fn edit_profile(cfg: &mut AppCfg, chain_name: Option<NamedChain>) -> Result<bool> {
    if cfg.user_profiles.is_empty() {
        println!("No profiles to edit.");
        return Ok(false);
    }

    let profile_options: Vec<String> = cfg
        .user_profiles
        .iter()
        .map(|p| {
            let is_default = cfg
                .workspace
                .default_profile
                .as_ref()
                .map(|default| default == &p.account.to_hex_literal())
                .unwrap_or(false);

            let default_marker = if is_default { " (default)" } else { "" };
            format!("{} ({}){}", p.nickname, p.account, default_marker)
        })
        .collect();

    let selection = dialoguer::Select::new()
        .with_prompt("Which profile would you like to edit?")
        .items(&profile_options)
        .default(0)
        .interact()?;

    let selected_profile = &cfg.user_profiles[selection];
    println!(
        "\nEditing profile: {} ({})",
        selected_profile.nickname, selected_profile.account
    );

    let options = vec![
        "Reset account address from mnemonic",
        "Change nickname",
        "Cancel",
    ];

    let edit_selection = dialoguer::Select::new()
        .with_prompt("What would you like to edit?")
        .items(&options)
        .default(0)
        .interact()?;

    match edit_selection {
        0 => {
            // Reset address - remove old profile and create new one
            let profile_id = selected_profile.account.to_hex_literal();
            remove_profile(cfg, &profile_id);
            let _profile = reset_address_profile(cfg, chain_name).await?;
            println!("Profile address reset successfully!");
            Ok(true)
        }
        1 => {
            // Change nickname
            let new_nickname: String = dialoguer::Input::new()
                .with_prompt("Enter new nickname")
                .with_initial_text(&selected_profile.nickname)
                .interact()?;

            // Update the profile nickname
            if let Some(profile) = cfg.user_profiles.get_mut(selection) {
                profile.nickname = new_nickname.clone();
                println!("Nickname changed to: {}", new_nickname);
                Ok(true)
            } else {
                Err(anyhow!("Profile not found"))
            }
        }
        2 => {
            // Cancel
            Ok(false)
        }
        _ => unreachable!(),
    }
}
