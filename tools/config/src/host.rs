use crate::node_yaml;
use dialoguer::{Confirm, Input};
use libra_types::legacy_types::mode_ol::MODE_0L;
use libra_types::ol_progress::OLProgress;
use libra_wallet::validator_files::SetValidatorConfiguration;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use zapatos_genesis::config::HostAndPort;
use zapatos_types::chain_id::NamedChain;

pub fn initialize_host(
    home_path: Option<PathBuf>,
    username: Option<&str>,
    host: HostAndPort,
    mnem: Option<String>,
    keep_legacy_address: bool,
) -> anyhow::Result<()> {
    libra_wallet::keys::refresh_validator_files(mnem, home_path.clone(), keep_legacy_address)?;
    OLProgress::complete("Initialized validator key files");

    let effective_username = username.unwrap_or("default_username"); // Use default if None
                                                                     // TODO: set validator fullnode configs. Not NONE
    SetValidatorConfiguration::new(home_path.clone(), effective_username.to_owned(), host, None)
        .set_config_files()?;
    OLProgress::complete("Saved genesis registration files locally");

    node_yaml::save_validator_yaml(home_path)?;
    OLProgress::complete("Saved validator node yaml file locally");
    Ok(())
}

/// interact with user to get ip address
pub fn what_host() -> Result<HostAndPort, anyhow::Error> {
    // get from external source since many cloud providers show different interfaces for `machine_ip`
    let resp = reqwest::blocking::get("https://ifconfig.me")?;
    // let ip_str = resp.text()?;

    let host = match resp.text() {
        Ok(ip_str) => {
            let h = HostAndPort::from_str(&format!("{}:6180", ip_str))?;
            if *MODE_0L == NamedChain::DEVNET {
                return Ok(h);
            }
            Some(h)
        }
        _ => None,
    };

    if let Some(h) = host {
        let txt = &format!(
            "Will you use this host, and this IP address {:?}, for your node?",
            h
        );
        if Confirm::new().with_prompt(txt).interact().unwrap() {
            return Ok(h);
        }
    };

    let input: String = Input::new()
        .with_prompt("Enter the DNS or IP address, with port 6180")
        .interact_text()
        .unwrap();
    let ip = input
        .parse::<HostAndPort>()
        .expect("Could not parse IP or DNS address");

    Ok(ip)
}

pub fn initialize_validator_configs(
    data_path: &Path,
    github_username: Option<&str>,
) -> Result<(), anyhow::Error> {
    let to_init = Confirm::new()
        .with_prompt(format!(
            "Want to freshen configs at {} now?",
            data_path.display()
        ))
        .interact()?;
    if to_init {
        let host = what_host()?;

        let keep_legacy_address = Confirm::new()
            .with_prompt("Is this a legacy V5 address you wish to keep?")
            .interact()?;

        initialize_host(
            Some(data_path.to_path_buf()),
            github_username,
            host,
            None,
            keep_legacy_address,
        )?;
    }

    Ok(())
}

#[test]
fn test_validator_files_config() {
    use libra_types::global_config_dir;
    let alice_mnem = "talent sunset lizard pill fame nuclear spy noodle basket okay critic grow sleep legend hurry pitch blanket clerk impose rough degree sock insane purse".to_string();
    let h = HostAndPort::local(6180).unwrap();
    let test_path = global_config_dir().join("test_genesis");
    if test_path.exists() {
        std::fs::remove_dir_all(&test_path).unwrap();
    }

    initialize_host(
        Some(test_path.clone()),
        Some("validator"),
        h,
        Some(alice_mnem),
        false,
    )
    .unwrap();

    std::fs::remove_dir_all(&test_path).unwrap();
}
