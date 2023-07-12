//! Configs for all 0L apps.

use anyhow::{Context, bail};
use crate::{
    exports::{AccountAddress, AuthenticationKey, NamedChain},
    global_config_dir,
};
use serde::{Deserialize, Serialize};
use url::Url;
use zapatos_crypto::ed25519::Ed25519PrivateKey;

use std::{
    fs,
    io::Write,
    path::PathBuf,
    str::FromStr,
};

use super::network_playlist::{NetworkPlaylist, self, HostProfile};

const CONFIG_FILE_NAME: &str = "libra.yaml";
/// MinerApp Configuration
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AppCfg {
    /// Workspace config
    pub workspace: Workspace,
    /// accounts which we have profiles for
    // NOTE: for v7 it will load the default() for migration
    // #[serde(default)]
    pub user_profiles: Vec<Profile>,
    /// Network profile
    // NOTE: new field from V7, so it's an option so that previous files can load.
    pub network_playlist: Vec<NetworkPlaylist>,
    /// Transaction configurations
    pub tx_configs: TxConfigs,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct LegacyToml {
    /// Workspace config
    pub workspace: Workspace,
    /// User Profile
    // NOTE: this profile is being deprecated
    pub profile: Profile,
    /// Chain Info for all users
    pub chain_info: ChainInfo,
    /// Transaction configurations
    pub tx_configs: TxConfigs,

}

impl LegacyToml {
    /// Get a AppCfg object from toml file
    pub fn parse_toml(path: Option<PathBuf>) -> anyhow::Result<Self> {
      let path = path.unwrap_or(global_config_dir().join("0L.toml"));
      let toml_buf = fs::read_to_string(path)?;
      Ok(toml::from_str(&toml_buf)?)
  }

}

pub fn default_file_path() -> PathBuf {
    global_config_dir().join(CONFIG_FILE_NAME)
}

impl AppCfg {
    /// load from default path
    pub fn load(file: Option<PathBuf>) -> anyhow::Result<Self> {
        let path = file.unwrap_or(default_file_path());

        let s = fs::read_to_string(path)?;
        Ok(serde_yaml::from_str(&s)?)
    }

    /// save the config file to 0L.toml to the workspace home path
    pub fn save_file(&self) -> anyhow::Result<PathBuf> {
        // let toml = toml::to_string(&self)?;
        let yaml = serde_yaml::to_string(&self)?;
        let home_path = &self.workspace.node_home.clone();
        // create home path if doesn't exist, usually only in dev/ci environments.
        fs::create_dir_all(&home_path)?;
        let toml_path = home_path.join(CONFIG_FILE_NAME);
        let mut file = fs::File::create(&toml_path)?;
        file.write(&yaml.as_bytes())?;

        println!(
            "app configs file saved to: {:?}",
            &toml_path
        );
        Ok(toml_path)
    }

    pub fn migrate(legacy_file: Option<PathBuf>, output: Option<PathBuf>) -> anyhow::Result<Self> {
      let l = LegacyToml::parse_toml(legacy_file)?;

      let nodes = if let Some(v) = l.profile.upstream_nodes.as_ref() {
        v.iter().map(|u| {
        let mut h = HostProfile::default();
        h.url = u.to_owned();
        h.note = u.to_string();
        h
      }).collect::<Vec<HostProfile>>()
    } else { vec![] };
      let np = NetworkPlaylist{
        chain_id: l.chain_info.chain_id,
        nodes: nodes,
      };
      let app_cfg = AppCfg {
        workspace: l.workspace,
        user_profiles: vec![l.profile],
        network_playlist: vec![np],
        tx_configs: l.tx_configs,
      };

      if let Some(p) = output {
        fs::create_dir_all(&p)?;
        println!("created file for {}", p.to_str().unwrap());
        let yaml = serde_yaml::to_string(&app_cfg)?;
        fs::write(p, yaml.as_bytes())?;
      } else {
        app_cfg.save_file()?;
      }

      Ok(app_cfg)
    }


    /// Get where the block/proofs are stored.
    pub fn get_block_dir(&self, nickname: Option<String>) -> anyhow::Result<PathBuf> {
        let mut home = self.workspace.node_home.clone();
        let profile = self.get_profile(nickname)?;
        let vdf_dir_name = format!("vdf_proofs_{}", &profile.account.to_string());

        home.push(vdf_dir_name);
        Ok(home)
    }

    /// get profile index by account fragment: full account string or shortened "nickname"
    fn get_profile_idx(&self, mut nickname: Option<String>) -> anyhow::Result<usize>{
      if self.user_profiles.len() == 0 { bail!("no profiles found") };
      if self.user_profiles.len() == 1 {
        return Ok(0)
      };

      // try to use the default profile unless one was requested
      if nickname.is_none() {
        nickname = Some(self.workspace.default_profile.clone())
      };

      if let Some(n) = nickname {
        let found = self.user_profiles.iter()
        .enumerate()
        .find_map(|(i, e)| {
          if e.nickname.contains(&n) || e.account.to_string().contains(&n) {
            Some(i)
          } else { None }
        });
        return found.context("could not find a profile")
      };

      bail!("no profiles found")
    }

    /// can get profile by account fragment: full account string or shortened "nickname"
    pub fn get_profile(&self, nickname: Option<String>) -> anyhow::Result<Profile>{
      let idx = self.get_profile_idx(nickname)?;
      let p = self.user_profiles.iter().nth(idx).context("no profile at index")?;
      Ok(p.to_owned())
    }

    /// get profile mutable borrow
    pub fn get_profile_mut(&mut self, nickname: Option<String>) -> anyhow::Result<&mut Profile>{
      let idx = self.get_profile_idx(nickname)?;
      let p = self.user_profiles.iter_mut().nth(idx).context("no profile at index")?;
      Ok(p)
    }

    pub fn maybe_add_profile(&mut self, profile: Profile) -> anyhow::Result<()>{
      if self.user_profiles.len() == 0 {
        self.user_profiles = vec![profile];
        return Ok(())
      }

      let mut found = false;
      // if it exists lets update it.
      self.user_profiles
      .iter_mut()
      .for_each(|e| {
        if e.account == profile.account {
          *e = profile.clone();
          found = true;
        }
      });

      if !found { self.user_profiles.push(profile); }

      Ok(())

    }
    /// Get where node key_store.json stored.
    pub fn init_app_configs(
        authkey: AuthenticationKey,
        account: AccountAddress,
        config_path: Option<PathBuf>,
        network_id: Option<NamedChain>,
        network_playlist: Option<NetworkPlaylist>,
    ) -> anyhow::Result<Self> {
        // TODO: Check if configs exist and warn on overwrite.
        let mut default_config = AppCfg::default();

        let profile = Profile::new(authkey, account);
        default_config.user_profiles = vec![profile];

        default_config.workspace.node_home =
            config_path.clone().unwrap_or_else(|| default_file_path());

        if let Some(id) = network_id {
            default_config.workspace.default_chain_id = id.to_owned();
        };

        if let Some(np) = network_playlist {
          default_config.network_playlist.push(np)
        }

        default_config.save_file()?;

        Ok(default_config)
    }

    // /// Removes current node from upstream nodes
    // /// To be used when DB is corrupted for instance.
    // pub fn remove_node(&mut self, host: String) -> anyhow::Result<()> {
    //     let nodes = self.profile.upstream_nodes.clone();
    //     match nodes.len() {
    //         1 => bail!("Cannot remove last node"),
    //         _ => {
    //             self.profile.upstream_nodes = nodes
    //                 .into_iter()
    //                 .filter(|each| !each.to_string().contains(&host))
    //                 .collect();
    //             self.save_file()?;
    //             Ok(())
    //         }
    //     }
    // }
    pub fn set_chain_id(&mut self, chain_id: NamedChain) {

      self.workspace.default_chain_id = chain_id;
    }

    pub async fn update_network_playlist(&mut self, chain_id: Option<NamedChain>,  playlist_url: Option<Url>) -> anyhow::Result<NetworkPlaylist>{
      // let chain_id = chain_id.unwrap_or(self.chain_info.chain_id);
      let url = playlist_url.unwrap_or(network_playlist::find_default_playlist(chain_id)?);

      let np = NetworkPlaylist::from_url(url, chain_id).await?;

      self.maybe_add_custom_playlist(&np);
      Ok(np)

    }
    ///fetch a network profile, optionally by profile name
    pub fn get_network_profile(
        &self,
        chain_id: Option<NamedChain>,
    ) -> anyhow::Result<NetworkPlaylist> {
        // TODO: avoid clone
        let np = self
            .network_playlist
            .clone();

        let chain_id = chain_id.unwrap_or(self.workspace.default_chain_id);
        let profile = np.into_iter().find(|each| each.chain_id == chain_id);

        Ok(profile.context("could not find a network profile")?)
    }

    pub async fn refresh_network_profile_and_save(
        &mut self,
        chain_id: Option<NamedChain>,
    ) -> anyhow::Result<NetworkPlaylist> {
        let mut np = self.get_network_profile(chain_id)?;
        np.refresh_sync_status().await?;
        self.save_file()?;
        Ok(np)
    }

    /// if there is a custom playlist update it
    pub fn maybe_add_custom_playlist(&mut self, new_playlist: &NetworkPlaylist) {
      let mut found = false;
      self.network_playlist.iter_mut().for_each(|play| {
        if play.chain_id == new_playlist.chain_id {
          found = true;
          *play = new_playlist.to_owned();
        }
      });
      if !found {
        self.network_playlist.push(new_playlist.to_owned());
      }

    }
    // TODO: always use CHAIN_ID from AppCfg
    ///fetch a network profile, optionally by profile name
    pub fn pick_url(&self, chain_id: Option<NamedChain>) -> anyhow::Result<Url> {
        let np = self.get_network_profile(chain_id)?;
        match np.the_best_one() {
            Ok(u) => Ok(u),
            Err(_) => {
              np.all_urls()?
                .into_iter()
                .next()
                .context("no urls to choose from")
            },
        }
    }
}

/// Default configuration settings.
impl Default for AppCfg {
    fn default() -> Self {
        Self {
            workspace: Workspace::default(),
            // profile: None,
            user_profiles: vec![],
            network_playlist: vec![],
            // chain_info: ChainInfo::default(),
            tx_configs: TxConfigs::default(),
        }
    }
}

/// Information about the Chain to mined for
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Workspace {
    /// default profile. Will match the substring of a full address or the nickname
    #[serde(default)]
    pub default_profile: String,
    /// default chain network profile to use
    pub default_chain_id: NamedChain,

    /// home directory of the diem node, may be the same as miner.
    pub node_home: PathBuf,
    // /// Directory of source code (for developer tests only)
    // pub source_path: Option<PathBuf>,
    // /// Directory to store blocks in
    // pub block_dir: String,
    // /// Directory for the database
    // #[serde(default = "default_db_path")]
    // pub db_path: PathBuf,
    // /// Path to which stdlib binaries for upgrades get built typically
    // /// /language/diem-framework/staged/stdlib.mv
    // pub stdlib_bin_path: Option<PathBuf>,
}

// fn default_db_path() -> PathBuf {
//     global_config_dir().join("db")
// }

impl Default for Workspace {
    fn default() -> Self {
        Self {
            default_profile: "default".to_string(),
            default_chain_id: NamedChain::MAINNET,
            node_home: crate::global_config_dir(),
            // source_path: None,
            // block_dir: "vdf_proofs".to_owned(),
            // db_path: default_db_path(),
            // stdlib_bin_path: None,
        }
    }
}

/// Information about the Chain to mined for
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ChainInfo {
    /// Chain that this work is being committed to
    pub chain_id: NamedChain,
}

// TODO: These defaults serving as test fixtures.
impl Default for ChainInfo {
    fn default() -> Self {
        Self {
            chain_id: NamedChain::MAINNET,
        }
    }
}

/// Miner profile to commit this work chain to a particular identity
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Profile {
    /// The 0L account for the Miner and prospective validator. This is derived from auth_key
    pub account: AccountAddress,
    /// Miner Authorization Key for 0L Blockchain. Note: not the same as public key, nor account.
    pub auth_key: AuthenticationKey,

    /// Private key only for use with testing
    /// Note: skip_serializing so that it is never saved to disk.
    #[serde(skip_serializing)]
    pub test_private_key: Option<Ed25519PrivateKey>,

    /// nickname for this profile
    pub nickname: String,
    #[serde(default)]
    /// is it already on chain
    pub on_chain: bool,
    #[serde(default)]
    /// what the last balance checked
    pub balance: u64,
    /// Language settings, for use with Carpe
    pub locale: Option<String>,
    /// An opportunity for the Miner to write a message on their genesis block.
    pub statement: String,

    // NOTE: V7: deprecated fields from 0L.toml
    // should have no effect on reading legacy files

    // /// ip address of this node. May be different from transaction URL.
    // pub ip: Ipv4Addr,

    // /// ip address of the validator fullnodee
    // pub vfn_ip: Option<Ipv4Addr>,

    /// Deprecation: Other nodes to connect for fallback connections
    pub upstream_nodes: Option<Vec<Url>>,

    // /// fullnode playlist URL to override default
    // pub override_playlist: Option<Url>,

    // /// Link to another delay tower.
    // pub tower_link: Option<String>,





}

impl Default for Profile {
    fn default() -> Self {
        Self {
            account: AccountAddress::ZERO,
            auth_key: AuthenticationKey::from_str(
                "0000000000000000000000000000000000000000000000000000000000000000",
            )
            .unwrap(),
            statement: "Protests rage across the nation".to_owned(),
            // ip: "0.0.0.0".parse().unwrap(),
            // vfn_ip: "0.0.0.0".parse().ok(),
            // // default_node: Some("http://localhost:8080".parse().expect("parse url")),
            // override_playlist: None,
            // upstream_nodes: vec!["http://localhost:8080".parse().expect("parse url")],
            // tower_link: None,
            test_private_key: None,
            locale: None,
            nickname: "default".to_string(),
            on_chain: false,
            balance: 0,
            upstream_nodes: None, // deprecation
        }
    }
}

impl Profile {
  pub fn new(auth: AuthenticationKey, acc: AccountAddress) -> Self {
    let mut p = Self::default();
    p.account = acc;
    p.auth_key = auth;
    p.nickname = get_nickname(p.account);
    return p
  }
}

pub fn get_nickname(acc: AccountAddress) -> String {
    // let's check if this is a legacy/founder key, it will have 16 zeros at the start, and that's not a useful nickname
    if acc.to_string()[..32] == "00000000000000000000000000000000".to_string() {
        return acc.to_string()[33..36].to_owned()
    }

    acc.to_string()[..3].to_owned()
}
/// Transaction types
pub enum TxType {
    /// critical txs
    Critical,
    /// management txs
    Mgmt,
    /// miner txs
    Miner,
    /// cheap txs
    Cheap,
}

/// Transaction types used in 0L clients
#[derive(Clone, Debug, Deserialize, Serialize)]
// #[serde(deny_unknown_fields)]
pub struct TxConfigs {
    /// baseline cost
    #[serde(default = "default_baseline_cost")]
    pub baseline_cost: TxCost,
    /// critical transactions cost
    #[serde(default = "default_critical_txs_cost")]
    pub critical_txs_cost: Option<TxCost>,
    /// management transactions cost
    #[serde(default = "default_management_txs_cost")]
    pub management_txs_cost: Option<TxCost>,
    /// Miner transactions cost
    #[serde(default = "default_miner_txs_cost")]
    pub miner_txs_cost: Option<TxCost>,
    /// Cheap or test transation costs
    #[serde(default = "default_cheap_txs_cost")]
    pub cheap_txs_cost: Option<TxCost>,
}

impl TxConfigs {
    /// get the user txs cost preferences for given transaction type
    pub fn get_cost(&self, tx_type: TxType) -> TxCost {
        let ref baseline = self.baseline_cost.clone();
        let cost = match tx_type {
            TxType::Critical => self.critical_txs_cost.as_ref().unwrap_or(baseline),
            TxType::Mgmt => self
                .management_txs_cost
                .as_ref()
                .unwrap_or_else(|| baseline),
            TxType::Miner => self.miner_txs_cost.as_ref().unwrap_or(baseline),
            TxType::Cheap => self.cheap_txs_cost.as_ref().unwrap_or(baseline),
        };
        cost.to_owned()
    }
}

/// Transaction preferences for a given type of transaction
#[derive(Clone, Debug, Deserialize, Serialize)]
// #[serde(deny_unknown_fields)]
pub struct TxCost {
    /// Max gas units to pay per transaction
    pub max_gas_unit_for_tx: u64, // gas UNITS of computation
    /// Max coin price per unit of gas
    pub coin_price_per_unit: u64, // price in micro GAS
    /// Time in seconds to timeout, from now
    pub user_tx_timeout: u64, // seconds,
}

impl TxCost {
    /// create new cost object
    pub fn new(cost: u64) -> Self {
        TxCost {
            max_gas_unit_for_tx: cost, // oracle upgrade transaction is expensive.
            coin_price_per_unit: 1,
            user_tx_timeout: 5_000,
        }
    }
}
impl Default for TxConfigs {
    fn default() -> Self {
        Self {
            baseline_cost: default_baseline_cost(),
            critical_txs_cost: default_critical_txs_cost(),
            management_txs_cost: default_management_txs_cost(),
            miner_txs_cost: default_miner_txs_cost(),
            cheap_txs_cost: default_cheap_txs_cost(),
        }
    }
}

fn default_baseline_cost() -> TxCost {
    TxCost::new(10_000)
}
fn default_critical_txs_cost() -> Option<TxCost> {
    Some(TxCost::new(1_000_000))
}
fn default_management_txs_cost() -> Option<TxCost> {
    Some(TxCost::new(100_000))
}
fn default_miner_txs_cost() -> Option<TxCost> {
    Some(TxCost::new(10_000))
}
fn default_cheap_txs_cost() -> Option<TxCost> {
    Some(TxCost::new(1_000))
}


#[tokio::test]
async fn test_create() {
  let mut a = AppCfg::default();
  a.user_profiles = vec![Profile::default()];
  a.save_file().unwrap();
}


#[test]
fn read_write() {

  let raw_yaml = r"
workspace:
  default_profile: '636'
  default_chain_id: TESTING
  node_home: /Users/lucas/.0L
user_profiles:
- account: 63609dfa4c8786bef29b201500064b2864689de724ca134f4e975784e3642776
  auth_key: 0x63609dfa4c8786bef29b201500064b2864689de724ca134f4e975784e3642776
  test_private_key: null
  nickname: '636'
  on_chain: false
  balance: 0
  locale: null
  statement: Protests rage across the nation
  upstream_nodes: null
- account: 4cca8361dfcab8ab5d80523cfea7d9fca5103e070ed7023d6b80a27eea2acc5d
  auth_key: 0x4cca8361dfcab8ab5d80523cfea7d9fca5103e070ed7023d6b80a27eea2acc5d
  test_private_key: null
  nickname: 4cc
  on_chain: false
  balance: 0
  locale: null
  statement: Protests rage across the nation
  upstream_nodes: null
- account: 771dcb53b7f69e0f3f7b0d2a6b7bd8d5ec44a1cca079fd501f7b3228360f3f92
  auth_key: 0x771dcb53b7f69e0f3f7b0d2a6b7bd8d5ec44a1cca079fd501f7b3228360f3f92
  test_private_key: null
  nickname: '771'
  on_chain: false
  balance: 0
  locale: null
  statement: Protests rage across the nation
  upstream_nodes: null
network_playlist:
- chain_id: MAINNET
  nodes:
  - url: http://204.186.74.42:8080/
    note: w
    version: 0
    is_api: false
    is_sync: false
- chain_id: TESTING
  nodes:
  - url: http://localhost:8080/
    note: default
    version: 0
    is_api: false
    is_sync: false
tx_configs:
  baseline_cost:
    max_gas_unit_for_tx: 10000
    coin_price_per_unit: 1
    user_tx_timeout: 5000
  critical_txs_cost:
    max_gas_unit_for_tx: 1000000
    coin_price_per_unit: 1
    user_tx_timeout: 5000
  management_txs_cost:
    max_gas_unit_for_tx: 100000
    coin_price_per_unit: 1
    user_tx_timeout: 5000
  miner_txs_cost:
    max_gas_unit_for_tx: 10000
    coin_price_per_unit: 1
    user_tx_timeout: 5000
  cheap_txs_cost:
    max_gas_unit_for_tx: 1000
    coin_price_per_unit: 1
    user_tx_timeout: 5000
";

  let cfg: AppCfg = serde_yaml::from_str(&raw_yaml).unwrap();
  // dbg!(&cfg);
  assert!(cfg.workspace.default_chain_id == NamedChain::TESTING);

  let np = cfg.get_network_profile(None).unwrap();
  assert!(np.chain_id == NamedChain::TESTING);

  let np = cfg.get_network_profile(Some(NamedChain::MAINNET)).unwrap();
  assert!(np.chain_id == NamedChain::MAINNET);

  // none of the node have been verified
  // dbg!(&np.the_good_ones().unwrap());

  assert!(np.the_good_ones().is_err());
  assert!(np.the_best_one().is_err());

  // pick url will failover to get the best, or the first in list
  let url = cfg.pick_url(None).unwrap();
  assert!(url.host_str().unwrap().contains(&"localhost"));

}