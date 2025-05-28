use libra_types::{core_types::app_cfg::AppCfg, exports::NamedChain};

/// Display existing profiles in the configuration
pub fn display_existing_profiles(cfg: &AppCfg) {
    println!("\nCurrent libra configuration profiles:");
    if cfg.user_profiles.is_empty() {
        println!("  No profiles found");
    } else {
        for (i, profile) in cfg.user_profiles.iter().enumerate() {
            let is_default = cfg
                .workspace
                .default_profile
                .as_ref()
                .map(|default| default == &profile.account.to_hex_literal())
                .unwrap_or(false);

            let default_marker = if is_default { " (default)" } else { "" };
            println!(
                "  {}. {} - {} {}",
                i + 1,
                profile.nickname,
                profile.account,
                default_marker
            );
        }
    }

    // Display current networks
    println!("\nCurrent networks:");
    if cfg.network_playlist.is_empty() {
        println!("  No networks configured");
    } else {
        for (i, np) in cfg.network_playlist.iter().enumerate() {
            let current_chain = cfg.workspace.default_chain_id;
            let is_default = current_chain == np.chain_name;
            let default_marker = if is_default { " (default)" } else { "" };

            println!(
                "  {}. {:?} ({} URLs){}",
                i + 1,
                np.chain_name,
                np.nodes.len(),
                default_marker
            );
        }
    }
}

/// Get available chains that are not yet configured
pub fn get_available_chains_not_configured(configured_chains: &[NamedChain]) -> Vec<NamedChain> {
    let all_chains = vec![NamedChain::MAINNET, NamedChain::TESTING];

    all_chains
        .into_iter()
        .filter(|chain| !configured_chains.contains(chain))
        .collect()
}
