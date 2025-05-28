use anyhow::{anyhow, Result};
use libra_types::{core_types::app_cfg::AppCfg, exports::NamedChain};

/// Change the default profile
pub fn change_default_profile(cfg: &mut AppCfg) -> Result<()> {
    if cfg.user_profiles.is_empty() {
        return Err(anyhow!("No profiles available"));
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

            let default_marker = if is_default { " (current default)" } else { "" };
            format!("{} ({}){}", p.nickname, p.account, default_marker)
        })
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

/// Change the default chain
pub fn change_default_chain(cfg: &mut AppCfg) -> Result<()> {
    use dialoguer::Select;

    if cfg.network_playlist.is_empty() {
        return Err(anyhow!(
            "No networks configured. Please configure a network first."
        ));
    }

    let configured_chains: Vec<NamedChain> = cfg
        .network_playlist
        .iter()
        .map(|np| np.chain_name)
        .collect();

    let chain_options: Vec<String> = configured_chains
        .iter()
        .map(|chain| format!("{:?}", chain).to_lowercase())
        .collect();

    let selection = Select::new()
        .with_prompt("Choose default chain")
        .items(&chain_options)
        .default(0)
        .interact()?;

    let selected_chain = configured_chains[selection];

    cfg.workspace.default_chain_id = selected_chain;
    println!("Default chain changed to: {:?}", selected_chain);
    Ok(())
}

/// Interactive defaults management
pub async fn interactive_change_defaults(cfg: &mut AppCfg) -> Result<bool> {
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
