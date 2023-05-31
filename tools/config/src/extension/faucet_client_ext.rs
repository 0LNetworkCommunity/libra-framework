use super::cli_config_ext::CliConfigExt;
use anyhow::{Context, Result};
use std::str::FromStr;
use url::Url;
use zapatos::common::types::{CliConfig, ConfigSearchMode, DEFAULT_PROFILE};
use zapatos_crypto::_once_cell::sync::Lazy;
use zapatos_rest_client::FaucetClient;

pub trait FaucetClientExt {
    fn default() -> Result<FaucetClient>;
}

static FAUCET_URL: Lazy<Url> = Lazy::new(|| {
    Url::from_str(
        std::env::var("DIEM_FAUCET_URL")
            .as_ref()
            .map(|s| s.as_str())
            .unwrap_or("http://0.0.0.0:8081"),
    )
    .unwrap()
});

impl FaucetClientExt for FaucetClient {
    fn default() -> Result<FaucetClient> {
        let profile = CliConfig::load_profile_ext(
            Some(DEFAULT_PROFILE),
            ConfigSearchMode::CurrentDirAndParents,
        )
        .context("Unable to locate 0l config file!")?
        .unwrap_or_default();
        let rest_url = profile.rest_url.context("Rest url is not set")?;
        Ok(FaucetClient::new(
            FAUCET_URL.clone(),
            Url::from_str(&rest_url).unwrap(),
        ))
    }
}
