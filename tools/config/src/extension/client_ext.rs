use super::cli_config_ext::CliConfigExt;
use anyhow::{Context, Result};
use std::{str::FromStr, time::Duration};
use url::Url;
use zapatos::common::types::{CliConfig, ConfigSearchMode, DEFAULT_PROFILE};
use zapatos_rest_client::Client;

pub const DEFAULT_TIMEOUT_SECS: u64 = 10;
pub const USER_AGENT: &str = concat!("libra-config/", env!("CARGO_PKG_VERSION"));

pub trait ClientExt {
    fn default() -> Result<Client>;
}

impl ClientExt for Client {
    fn default() -> Result<Client> {
        let profile =
            CliConfig::load_profile_ext(Some(DEFAULT_PROFILE), ConfigSearchMode::CurrentDir)?
                .unwrap_or_default();
        let rest_url = profile.rest_url.context("Rest url is not set")?;
        Ok(Client::new_with_timeout_and_user_agent(
            Url::from_str(&rest_url).unwrap(),
            Duration::from_secs(DEFAULT_TIMEOUT_SECS),
            USER_AGENT,
        ))
    }
}
