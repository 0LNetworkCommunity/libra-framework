use crate::config_wizard;
use anyhow::{anyhow, Result};
use libra_types::{
    core_types::app_cfg::{self, AppCfg},
    exports::{Client, NamedChain},
    type_extensions::client_ext::ClientExt,
};
use std::path::PathBuf;
use url::Url;

/// Fix configuration options
pub struct FixOptions {
    pub reset_address: bool,
    pub remove_profile: Option<String>,
    pub fullnode_url: Option<Url>,
    pub config_path: Option<PathBuf>,
    pub chain_name: Option<NamedChain>,
}

/// Handle the fix configuration command
pub async fn fix_config(options: FixOptions) -> Result<()> {
    // Load configuration file
    let mut cfg = AppCfg::load(options.config_path.clone())
        .map_err(|e| anyhow!("no config file found for libra tools, {}", e))?;

    display_existing_profiles(&cfg);

    let mut changes_made = false;

    // If no command-line options provided, enter interactive mode
    let args_count = [
        options.reset_address,
        options.remove_profile.is_some(),
        options.fullnode_url.is_some(),
    ]
    .iter()
    .filter(|&&x| x)
    .count();

    if args_count == 0 {
        // Interactive mode
        changes_made = interactive_fix_setup(&mut cfg, options.chain_name).await?;
    } else {
        // Command-line mode - validate exactly one argument is provided
        if args_count > 1 {
            return Err(anyhow!(
                "Only one argument can be provided to 'fix' command"
            ));
        }

        // Handle specific fix options
        if options.reset_address {
            let _profile = reset_address_profile(&mut cfg, options.chain_name).await?;
            changes_made = true;
        }

        if let Some(profile_name) = &options.remove_profile {
            remove_profile(&mut cfg, profile_name);
            changes_made = true;
        }

        if let Some(url) = &options.fullnode_url {
            force_url_overwrite(&mut cfg, url, options.chain_name)?;
            changes_made = true;
        }
    }

    // Save configuration file only if changes were made
    if changes_made {
        cfg.save_file()?;
        println!("Configuration saved successfully!");
    } else {
        println!("No changes were made to the configuration.");
    }

    Ok(())
}

/// Display existing profiles in the configuration
fn display_existing_profiles(cfg: &AppCfg) {
    if !cfg.user_profiles.is_empty() {
        println!("your profiles:");
        for p in &cfg.user_profiles {
            println!("- address: {}, nickname: {}", p.account, p.nickname);
        }
    } else {
        println!("no profiles found");
    }
}

/// Reset address profile by prompting for account details
async fn reset_address_profile(
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
            Ok(r) => r,
            _ => {
                println!("This looks like a new account, and it's not yet on chain. If this is not what you expected, are you sure you are using the correct recovery mnemonic?");
                // do nothing
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
fn remove_profile(cfg: &mut AppCfg, profile_identifier: &str) {
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

/// Force URL overwrite in the network profile
fn force_url_overwrite(cfg: &mut AppCfg, url: &Url, chain_name: Option<NamedChain>) -> Result<()> {
    let np = cfg.get_network_profile_mut(chain_name)?;
    np.nodes = vec![];
    np.add_url(url.to_owned());
    Ok(())
}

/// Interactive fix setup when no command-line options are provided
async fn interactive_fix_setup(cfg: &mut AppCfg, chain_name: Option<NamedChain>) -> Result<bool> {
    println!("\nWhat do you want to do?");

    let options = vec![
        "Manage Profiles",
        "Manage Networks (edit fullnodes for each chain name)",
        "Change defaults (profile or network)",
        "Exit without making changes",
    ];

    let selection = dialoguer::Select::new()
        .with_prompt("Choose an option")
        .items(&options)
        .default(0)
        .interact()?;

    match selection {
        0 => {
            // Manage Profiles
            interactive_manage_profiles(cfg, chain_name).await
        }
        1 => {
            // Manage Networks
            interactive_manage_networks(cfg, chain_name).await
        }
        2 => {
            // Change defaults
            interactive_change_defaults(cfg).await
        }
        3 => {
            // Exit
            Ok(false)
        }
        _ => unreachable!(),
    }
}

/// Interactive profile removal
fn interactive_remove_profile(cfg: &mut AppCfg) -> Result<()> {
    let profile_options: Vec<String> = cfg
        .user_profiles
        .iter()
        .map(|p| format!("{} ({})", p.nickname, p.account))
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

/// Interactive URL update
async fn interactive_update_url(cfg: &mut AppCfg, chain_name: Option<NamedChain>) -> Result<bool> {
    // Display current URLs
    let current_urls = match cfg.get_network_profile(chain_name) {
        Ok(np) => np
            .nodes
            .iter()
            .map(|n| n.url.to_string())
            .collect::<Vec<_>>(),
        Err(_) => vec!["No URLs configured".to_string()],
    };

    println!("Current fullnode URLs:");
    for (i, url) in current_urls.iter().enumerate() {
        println!("  {}. {}", i + 1, url);
    }

    println!("\nWhat would you like to do with fullnode URLs?");

    let options = vec![
        "Add a fullnode URL",
        "Remove all fullnodes",
        "Refresh from default playlist",
        "Refresh from a playlist you specify",
        "Cancel",
    ];

    let selection = dialoguer::Select::new()
        .with_prompt("Choose an option")
        .items(&options)
        .default(0)
        .interact()?;

    match selection {
        0 => {
            // Add a fullnode URL
            add_fullnode_url(cfg, chain_name)?;
            Ok(true)
        }
        1 => {
            // Remove all fullnodes
            let changed = remove_all_fullnodes(cfg, chain_name)?;
            Ok(changed)
        }
        2 => {
            // Refresh from default playlist
            let changed = refresh_from_default_playlist(cfg, chain_name).await?;
            Ok(changed)
        }
        3 => {
            // Refresh from specified playlist
            let changed = refresh_from_custom_playlist(cfg, chain_name).await?;
            Ok(changed)
        }
        4 => {
            // Cancel
            Ok(false)
        }
        _ => unreachable!(),
    }
}

/// Add a single fullnode URL to the configuration
fn add_fullnode_url(cfg: &mut AppCfg, chain_name: Option<NamedChain>) -> Result<()> {
    let url_input: String = dialoguer::Input::new()
        .with_prompt("Enter fullnode URL to add")
        .interact()?;

    let url = Url::parse(&url_input).map_err(|_| anyhow!("Invalid URL format"))?;

    let np = cfg.get_network_profile_mut(chain_name)?;
    np.add_url(url.clone());

    println!("Added fullnode URL: {}", url);
    Ok(())
}

/// Remove all fullnode URLs from the configuration
fn remove_all_fullnodes(cfg: &mut AppCfg, chain_name: Option<NamedChain>) -> Result<bool> {
    if dialoguer::Confirm::new()
        .with_prompt("Are you sure you want to remove all fullnode URLs?")
        .interact()?
    {
        let np = cfg.get_network_profile_mut(chain_name)?;
        np.nodes.clear();
        println!("All fullnode URLs removed.");
        Ok(true)
    } else {
        println!("Operation cancelled.");
        Ok(false)
    }
}

/// Refresh URLs from the default playlist for the network
async fn refresh_from_default_playlist(
    cfg: &mut AppCfg,
    chain_name: Option<NamedChain>,
) -> Result<bool> {
    use libra_types::core_types::network_playlist::NetworkPlaylist;

    println!("Refreshing from default playlist...");

    let mut np = NetworkPlaylist::default_for_network(chain_name).await?;
    np.refresh_sync_status().await?;

    if dialoguer::Confirm::new()
        .with_prompt("Replace current URLs with default playlist URLs?")
        .interact()?
    {
        cfg.maybe_add_custom_playlist(&np);
        println!("URLs refreshed from default playlist.");
        Ok(true)
    } else {
        println!("Operation cancelled.");
        Ok(false)
    }
}

/// Refresh URLs from a custom playlist URL
async fn refresh_from_custom_playlist(
    cfg: &mut AppCfg,
    chain_name: Option<NamedChain>,
) -> Result<bool> {
    use libra_types::core_types::network_playlist::NetworkPlaylist;

    let playlist_url_input: String = dialoguer::Input::new()
        .with_prompt("Enter playlist URL")
        .interact()?;

    let playlist_url =
        Url::parse(&playlist_url_input).map_err(|_| anyhow!("Invalid playlist URL format"))?;

    println!("Fetching playlist from {}...", playlist_url);

    let mut np = NetworkPlaylist::from_playlist_url(playlist_url, chain_name).await?;
    np.refresh_sync_status().await?;

    if dialoguer::Confirm::new()
        .with_prompt("Replace current URLs with playlist URLs?")
        .interact()?
    {
        cfg.maybe_add_custom_playlist(&np);
        println!("URLs refreshed from custom playlist.");
        Ok(true)
    } else {
        println!("Operation cancelled.");
        Ok(false)
    }
}

/// Interactive edit for a single profile (with option to add new profile)
async fn interactive_edit_single_profile(
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
async fn interactive_profile_management(
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
            change_default_profile(cfg)?;
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
async fn interactive_manage_profiles(
    cfg: &mut AppCfg,
    chain_name: Option<NamedChain>,
) -> Result<bool> {
    if cfg.user_profiles.len() == 1 {
        // Single profile: Show edit profile menu with option to add new profile
        interactive_edit_single_profile(cfg, chain_name).await
    } else if cfg.user_profiles.len() > 1 {
        // Multiple profiles: Show profile management menu
        interactive_profile_management(cfg, chain_name).await
    } else {
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
                println!("No changes made.");
                Ok(false)
            }
            _ => unreachable!(),
        }
    }
}

/// Interactive network management
async fn interactive_manage_networks(
    cfg: &mut AppCfg,
    _chain_name: Option<NamedChain>,
) -> Result<bool> {
    // Display current networks
    println!("\nConfigured networks:");
    if cfg.network_playlist.is_empty() {
        println!("  No networks configured");
    } else {
        for (i, np) in cfg.network_playlist.iter().enumerate() {
            let url_count = np.nodes.len();
            println!(
                "  {}. {:?} ({} fullnode URLs)",
                i + 1,
                np.chain_name,
                url_count
            );
        }
    }

    println!("\nWhat would you like to do?");

    let mut options = vec![];

    // If we have networks, add edit options
    if !cfg.network_playlist.is_empty() {
        options.push("Edit fullnode URLs for a specific network");
    }

    // Add option to add new network if there are unconfigured chains
    let configured_chains: Vec<NamedChain> = cfg
        .network_playlist
        .iter()
        .map(|np| np.chain_name)
        .collect();

    let available_chains = get_available_chains_not_configured(&configured_chains);
    if !available_chains.is_empty() {
        options.push("Add a new network (chain)");
    }

    options.push("Cancel");

    let selection = dialoguer::Select::new()
        .with_prompt("Choose an option")
        .items(&options)
        .default(0)
        .interact()?;

    match selection {
        0 if !cfg.network_playlist.is_empty() => {
            // Edit URLs for specific network
            select_and_edit_network(cfg).await
        }
        i if options[i] == "Add a new network (chain)" => {
            // Add new network
            add_new_network(cfg).await
        }
        _ => {
            // Cancel
            Ok(false)
        }
    }
}

/// Interactive defaults management
async fn interactive_change_defaults(cfg: &mut AppCfg) -> Result<bool> {
    println!("\nWhat default would you like to change?");

    let options = vec!["Change default profile", "Change default chain", "Cancel"];

    let selection = dialoguer::Select::new()
        .with_prompt("Choose an option")
        .items(&options)
        .default(0)
        .interact()?;

    match selection {
        0 => {
            // Change default profile
            if cfg.user_profiles.len() > 1 {
                change_default_profile(cfg)?;
                Ok(true)
            } else {
                println!("Only one profile available. Cannot change default.");
                Ok(false)
            }
        }
        1 => {
            // Change default chain
            change_default_chain(cfg)?;
            Ok(true)
        }
        2 => {
            // Cancel
            Ok(false)
        }
        _ => unreachable!(),
    }
}

/// Change the default profile
fn change_default_profile(cfg: &mut AppCfg) -> Result<()> {
    if cfg.user_profiles.is_empty() {
        return Err(anyhow!("No profiles available"));
    }

    let profile_options: Vec<String> = cfg
        .user_profiles
        .iter()
        .map(|p| format!("{} ({})", p.nickname, p.account))
        .collect();

    let selection = dialoguer::Select::new()
        .with_prompt("Choose default profile")
        .items(&profile_options)
        .default(0)
        .interact()?;

    let selected_profile = &cfg.user_profiles[selection];
    cfg.workspace
        .set_default(selected_profile.account.to_hex_literal());

    println!(
        "Default profile changed to: {} ({})",
        selected_profile.nickname, selected_profile.account
    );
    Ok(())
}

/// Add a new profile to the configuration
async fn add_new_profile(cfg: &mut AppCfg, chain_name: Option<NamedChain>) -> Result<()> {
    println!("\nAdding a new profile...");
    let _profile = reset_address_profile(cfg, chain_name).await?;
    println!("New profile added successfully!");
    Ok(())
}

/// Change the default chain
fn change_default_chain(cfg: &mut AppCfg) -> Result<()> {
    use dialoguer::Select;

    let chain_options = vec!["mainnet", "testnet", "devnet", "testing"];

    let selection = Select::new()
        .with_prompt("Choose default chain")
        .items(&chain_options)
        .default(0)
        .interact()?;

    let selected_chain = match selection {
        0 => NamedChain::MAINNET,
        1 => NamedChain::TESTNET,
        2 => NamedChain::DEVNET,
        3 => NamedChain::TESTING,
        _ => unreachable!(),
    };

    cfg.workspace.default_chain_id = selected_chain;
    println!("Default chain changed to: {:?}", selected_chain);
    Ok(())
}

/// Edit a specific profile
async fn edit_profile(cfg: &mut AppCfg, chain_name: Option<NamedChain>) -> Result<bool> {
    if cfg.user_profiles.is_empty() {
        println!("No profiles to edit.");
        return Ok(false);
    }

    let profile_options: Vec<String> = cfg
        .user_profiles
        .iter()
        .map(|p| format!("{} ({})", p.nickname, p.account))
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

/// Get available chains that are not yet configured
fn get_available_chains_not_configured(configured_chains: &[NamedChain]) -> Vec<NamedChain> {
    let all_chains = vec![
        NamedChain::MAINNET,
        NamedChain::TESTNET,
        NamedChain::DEVNET,
        NamedChain::TESTING,
    ];

    all_chains
        .into_iter()
        .filter(|chain| !configured_chains.contains(chain))
        .collect()
}

/// Select and edit a specific network
async fn select_and_edit_network(cfg: &mut AppCfg) -> Result<bool> {
    if cfg.network_playlist.is_empty() {
        println!("No networks configured.");
        return Ok(false);
    }

    let network_options: Vec<String> = cfg
        .network_playlist
        .iter()
        .map(|np| format!("{:?} ({} URLs)", np.chain_name, np.nodes.len()))
        .collect();

    let selection = dialoguer::Select::new()
        .with_prompt("Which network would you like to edit?")
        .items(&network_options)
        .default(0)
        .interact()?;

    let selected_chain = cfg.network_playlist[selection].chain_name;
    println!("\nEditing network: {:?}", selected_chain);

    interactive_update_url(cfg, Some(selected_chain)).await
}

/// Add a new network (chain) to the configuration
async fn add_new_network(cfg: &mut AppCfg) -> Result<bool> {
    let configured_chains: Vec<NamedChain> = cfg
        .network_playlist
        .iter()
        .map(|np| np.chain_name)
        .collect();

    let available_chains = get_available_chains_not_configured(&configured_chains);

    if available_chains.is_empty() {
        println!("All available chains are already configured.");
        return Ok(false);
    }

    let chain_options: Vec<String> = available_chains
        .iter()
        .map(|chain| format!("{:?}", chain))
        .collect();

    let selection = dialoguer::Select::new()
        .with_prompt("Which chain would you like to add?")
        .items(&chain_options)
        .default(0)
        .interact()?;

    let selected_chain = available_chains[selection];

    println!("\nAdding network: {:?}", selected_chain);
    println!("Choose how to configure this network:");

    let config_options = vec![
        "Use default playlist for this chain",
        "Add custom fullnode URLs",
        "Load from custom playlist URL",
        "Cancel",
    ];

    let config_selection = dialoguer::Select::new()
        .with_prompt("Choose configuration method")
        .items(&config_options)
        .default(0)
        .interact()?;

    match config_selection {
        0 => {
            // Use default playlist
            use libra_types::core_types::network_playlist::NetworkPlaylist;

            println!("Loading default playlist for {:?}...", selected_chain);
            let mut np = NetworkPlaylist::default_for_network(Some(selected_chain)).await?;
            np.refresh_sync_status().await?;

            cfg.maybe_add_custom_playlist(&np);
            println!("Network {:?} added with default playlist.", selected_chain);
            Ok(true)
        }
        1 => {
            // Add custom URLs
            use libra_types::core_types::network_playlist::NetworkPlaylist;

            let mut np = NetworkPlaylist::new(None, Some(selected_chain));

            loop {
                let url_input: String = dialoguer::Input::new()
                    .with_prompt("Enter fullnode URL (or press Enter to finish)")
                    .allow_empty(true)
                    .interact()?;

                if url_input.trim().is_empty() {
                    break;
                }

                match Url::parse(&url_input) {
                    Ok(url) => {
                        np.add_url(url.clone());
                        println!("Added URL: {}", url);
                    }
                    Err(_) => {
                        println!("Invalid URL format. Please try again.");
                        continue;
                    }
                }

                if !dialoguer::Confirm::new()
                    .with_prompt("Add another URL?")
                    .default(false)
                    .interact()?
                {
                    break;
                }
            }

            if np.nodes.is_empty() {
                println!("No URLs added. Network not configured.");
                Ok(false)
            } else {
                cfg.maybe_add_custom_playlist(&np);
                println!(
                    "Network {:?} added with {} custom URLs.",
                    selected_chain,
                    np.nodes.len()
                );
                Ok(true)
            }
        }
        2 => {
            // Load from custom playlist
            use libra_types::core_types::network_playlist::NetworkPlaylist;

            let playlist_url_input: String = dialoguer::Input::new()
                .with_prompt("Enter playlist URL")
                .interact()?;

            let playlist_url = Url::parse(&playlist_url_input)
                .map_err(|_| anyhow!("Invalid playlist URL format"))?;

            println!("Fetching playlist from {}...", playlist_url);
            let mut np =
                NetworkPlaylist::from_playlist_url(playlist_url, Some(selected_chain)).await?;
            np.refresh_sync_status().await?;

            cfg.maybe_add_custom_playlist(&np);
            println!("Network {:?} added from custom playlist.", selected_chain);
            Ok(true)
        }
        3 => {
            // Cancel
            println!("Network addition cancelled.");
            Ok(false)
        }
        _ => unreachable!(),
    }
}
