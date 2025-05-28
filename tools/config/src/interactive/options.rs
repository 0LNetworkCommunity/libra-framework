use anyhow::{anyhow, Result};
use libra_types::{core_types::app_cfg::AppCfg, exports::NamedChain};
use std::path::PathBuf;
use url::Url;

use super::{fix, networks, profiles, utils};

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

    utils::display_existing_profiles(&cfg);

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
        changes_made = fix::interactive_fix_setup(&mut cfg, options.chain_name).await?;
    } else {
        // Command-line mode - validate exactly one argument is provided
        if args_count > 1 {
            return Err(anyhow!(
                "Only one argument can be provided to 'fix' command"
            ));
        }

        // Handle specific fix options
        if options.reset_address {
            let _profile = profiles::reset_address_profile(&mut cfg, options.chain_name).await?;
            changes_made = true;
        }

        if let Some(profile_name) = &options.remove_profile {
            profiles::remove_profile(&mut cfg, profile_name);
            changes_made = true;
        }

        if let Some(url) = &options.fullnode_url {
            networks::force_url_overwrite(&mut cfg, url, options.chain_name)?;
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
