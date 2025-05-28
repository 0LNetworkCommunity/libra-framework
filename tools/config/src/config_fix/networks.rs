use anyhow::{anyhow, Result};
use libra_types::{
    core_types::{app_cfg::AppCfg, network_playlist::NetworkPlaylist},
    exports::NamedChain,
};
use url::Url;

/// Force URL overwrite in the network profile
pub fn force_url_overwrite(cfg: &mut AppCfg, url: &Url, chain_name: Option<NamedChain>) -> Result<()> {
    let np = cfg.get_network_profile_mut(chain_name)?;
    np.nodes = vec![];
    np.add_url(url.to_owned());
    Ok(())
}

/// Add a single fullnode URL to the configuration
pub fn add_fullnode_url(cfg: &mut AppCfg, chain_name: Option<NamedChain>) -> Result<()> {
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
pub fn remove_all_fullnodes(cfg: &mut AppCfg, chain_name: Option<NamedChain>) -> Result<bool> {
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
pub async fn refresh_from_default_playlist(
    cfg: &mut AppCfg,
    chain_name: Option<NamedChain>,
) -> Result<bool> {
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
pub async fn refresh_from_custom_playlist(
    cfg: &mut AppCfg,
    chain_name: Option<NamedChain>,
) -> Result<bool> {
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

/// Interactive URL update
pub async fn interactive_update_url(cfg: &mut AppCfg, chain_name: Option<NamedChain>) -> Result<bool> {
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

/// Select and edit a specific network
pub async fn select_and_edit_network(cfg: &mut AppCfg) -> Result<bool> {
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
pub async fn add_new_network(cfg: &mut AppCfg) -> Result<bool> {
    use crate::config_fix::utils::get_available_chains_not_configured;

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
            println!("Loading default playlist for {:?}...", selected_chain);
            let mut np = NetworkPlaylist::default_for_network(Some(selected_chain)).await?;
            np.refresh_sync_status().await?;

            cfg.maybe_add_custom_playlist(&np);
            println!("Network {:?} added with default playlist.", selected_chain);
            Ok(true)
        }
        1 => {
            // Add custom URLs
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

/// Interactive network management
pub async fn interactive_manage_networks(
    cfg: &mut AppCfg,
    _chain_name: Option<NamedChain>,
) -> Result<bool> {
    use crate::config_fix::utils::get_available_chains_not_configured;

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
