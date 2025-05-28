use anyhow::Result;
use libra_types::{
    core_types::app_cfg::AppCfg,
    exports::NamedChain,
};

use super::{defaults, networks, profiles};

/// Interactive fix setup when no command-line options are provided
pub async fn interactive_fix_setup(cfg: &mut AppCfg, chain_name: Option<NamedChain>) -> Result<bool> {
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
            profiles::interactive_manage_profiles(cfg, chain_name).await
        }
        1 => {
            // Manage Networks
            networks::interactive_manage_networks(cfg, chain_name).await
        }
        2 => {
            // Change defaults
            defaults::interactive_change_defaults(cfg).await
        }
        3 => {
            // Exit
            Ok(false)
        }
        _ => unreachable!(),
    }
}
