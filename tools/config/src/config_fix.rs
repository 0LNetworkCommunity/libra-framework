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
        interactive_fix_setup(&mut cfg, options.chain_name).await?;
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
        }

        if let Some(profile_name) = &options.remove_profile {
            remove_profile(&mut cfg, profile_name);
        }

        if let Some(url) = &options.fullnode_url {
            force_url_overwrite(&mut cfg, url, options.chain_name)?;
        }
    }

    // Save configuration file
    cfg.save_file()?;
    println!("Configuration saved successfully!");
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
    if client.get_index().await.is_ok() && authkey != libra_types::exports::AuthenticationKey::zero() {
        account_address = match client
            .lookup_originating_address(authkey)
            .await
        {
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
        cfg.workspace
            .set_default(account_address.to_hex_literal());
    }

    cfg.get_profile_mut(Some(account_address.to_hex_literal()))
}

/// Remove a profile from the configuration
fn remove_profile(cfg: &mut AppCfg, profile_name: &str) {
    let result = cfg.try_remove_profile(profile_name);
    if result.is_err() {
        println!("no profile found matching {}", profile_name)
    }
}

/// Force URL overwrite in the network profile
fn force_url_overwrite(
    cfg: &mut AppCfg,
    url: &Url,
    chain_name: Option<NamedChain>,
) -> Result<()> {
    let np = cfg.get_network_profile_mut(chain_name)?;
    np.nodes = vec![];
    np.add_url(url.to_owned());
    Ok(())
}

/// Interactive fix setup when no command-line options are provided
async fn interactive_fix_setup(cfg: &mut AppCfg, chain_name: Option<NamedChain>) -> Result<()> {
    println!("\nWhat would you like to fix?");

    let options = vec![
        "Reset account address from mnemonic",
        "Remove a profile",
        "Update fullnode URL",
        "Exit without making changes",
    ];

    let selection = dialoguer::Select::new()
        .with_prompt("Choose an option")
        .items(&options)
        .default(0)
        .interact()?;

    match selection {
        0 => {
            // Reset address
            let _profile = reset_address_profile(cfg, chain_name).await?;
            println!("Address reset successfully!");
        }
        1 => {
            // Remove profile
            if cfg.user_profiles.is_empty() {
                println!("No profiles to remove.");
                return Ok(());
            }

            interactive_remove_profile(cfg)?;
        }
        2 => {
            // Update URL
            interactive_update_url(cfg, chain_name).await?;
        }
        3 => {
            // Exit
            println!("No changes made.");
            return Ok(());
        }
        _ => unreachable!(),
    }

    Ok(())
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
        .with_prompt(&format!(
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
async fn interactive_update_url(cfg: &mut AppCfg, chain_name: Option<NamedChain>) -> Result<()> {
    // Display current URLs
    let current_urls = match cfg.get_network_profile(chain_name) {
        Ok(np) => np.nodes.iter().map(|n| n.url.to_string()).collect::<Vec<_>>(),
        Err(_) => vec!["No URLs configured".to_string()],
    };

    println!("Current fullnode URLs:");
    for (i, url) in current_urls.iter().enumerate() {
        println!("  {}. {}", i + 1, url);
    }

    println!("\nWhat would you like to do with fullnode URLs?");

    let options = vec![
        "Add a fullnode URL",
        "Remove a specific fullnode",
        "Remove all fullnodes",
        "Refresh from default playlist",
        "Refresh from a playlist URL you specify",
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
        }
        1 => {
            // Remove a specific fullnode
            remove_specific_fullnode(cfg, chain_name)?;
        }
        2 => {
            // Remove all fullnodes
            remove_all_fullnodes(cfg, chain_name)?;
        }
        3 => {
            // Refresh from default playlist
            refresh_from_default_playlist(cfg, chain_name).await?;
        }
        4 => {
            // Refresh from specified playlist
            refresh_from_custom_playlist(cfg, chain_name).await?;
        }
        5 => {
            // Cancel
            println!("No changes made to fullnode URLs.");
            return Ok(());
        }
        _ => unreachable!(),
    }

    Ok(())
}

/// Add a single fullnode URL to the configuration
fn add_fullnode_url(cfg: &mut AppCfg, chain_name: Option<NamedChain>) -> Result<()> {
    let url_input: String = dialoguer::Input::new()
        .with_prompt("Enter fullnode URL to add")
        .interact()?;

    let url = Url::parse(&url_input)
        .map_err(|_| anyhow!("Invalid URL format"))?;

    let np = cfg.get_network_profile_mut(chain_name)?;
    np.add_url(url.clone());

    println!("Added fullnode URL: {}", url);
    Ok(())
}

/// Remove a specific fullnode URL from the configuration
fn remove_specific_fullnode(cfg: &mut AppCfg, chain_name: Option<NamedChain>) -> Result<()> {
    let np = cfg.get_network_profile(chain_name)?;

    if np.nodes.is_empty() {
        println!("No fullnode URLs to remove.");
        return Ok(());
    }

    let url_options: Vec<String> = np.nodes
        .iter()
        .map(|n| n.url.to_string())
        .collect();

    let selection = dialoguer::Select::new()
        .with_prompt("Which fullnode URL would you like to remove?")
        .items(&url_options)
        .interact()?;

    let url_to_remove = &np.nodes[selection].url;

    if dialoguer::Confirm::new()
        .with_prompt(&format!("Are you sure you want to remove '{}'?", url_to_remove))
        .interact()?
    {
        let np_mut = cfg.get_network_profile_mut(chain_name)?;
        np_mut.nodes.remove(selection);
        println!("Removed fullnode URL: {}", url_to_remove);
    } else {
        println!("Operation cancelled.");
    }

    Ok(())
}

/// Remove all fullnode URLs from the configuration
fn remove_all_fullnodes(cfg: &mut AppCfg, chain_name: Option<NamedChain>) -> Result<()> {
    if dialoguer::Confirm::new()
        .with_prompt("Are you sure you want to remove all fullnode URLs?")
        .interact()?
    {
        let np = cfg.get_network_profile_mut(chain_name)?;
        np.nodes.clear();
        println!("All fullnode URLs removed.");
    } else {
        println!("Operation cancelled.");
    }
    Ok(())
}

/// Refresh URLs from the default playlist for the network
async fn refresh_from_default_playlist(cfg: &mut AppCfg, chain_name: Option<NamedChain>) -> Result<()> {
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
    } else {
        println!("Operation cancelled.");
    }

    Ok(())
}

/// Refresh URLs from a custom playlist URL
async fn refresh_from_custom_playlist(cfg: &mut AppCfg, chain_name: Option<NamedChain>) -> Result<()> {
    use libra_types::core_types::network_playlist::NetworkPlaylist;

    let playlist_url_input: String = dialoguer::Input::new()
        .with_prompt("Enter playlist URL")
        .interact()?;

    let playlist_url = Url::parse(&playlist_url_input)
        .map_err(|_| anyhow!("Invalid playlist URL format"))?;

    println!("Fetching playlist from {}...", playlist_url);

    let mut np = NetworkPlaylist::from_playlist_url(playlist_url, chain_name).await?;
    np.refresh_sync_status().await?;

    if dialoguer::Confirm::new()
        .with_prompt("Replace current URLs with playlist URLs?")
        .interact()?
    {
        cfg.maybe_add_custom_playlist(&np);
        println!("URLs refreshed from custom playlist.");
    } else {
        println!("Operation cancelled.");
    }

    Ok(())
}
