//! network configs
use crate::exports::{Client, NamedChain};
use anyhow::{bail, Context};
use futures::{stream::FuturesUnordered, StreamExt};
use rand::{seq::SliceRandom, thread_rng};
use serde_with::{serde_as, DisplayFromStr};
use url::Url;

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
pub struct HostProfile {
    pub url: Url,
    pub note: String,
    #[serde(default)]
    pub version: u64,
    #[serde(default)]
    pub is_api: bool,
    #[serde(default)]
    pub is_sync: bool,
}
/// from the list of seed_peers find the best peer to connect to.
/// First does a light port check on all peers, and eliminated unresponsive
/// Then from a random list fetches the first 3 nodes to respond with a waypoint.
/// picks the node with the highest waypoint.
/// moves that node othe top of the seed peers vector.
/// sets it in preferences as the default peer.
impl Default for HostProfile {
    fn default() -> Self {
        Self {
            url: "http://localhost:8080".parse().unwrap(),
            version: 0,
            is_api: false,
            is_sync: false,
            note: "default".to_string(),
        }
    }
}

/// The metadata of fullnodes we may connect to
/// The local data will be more complete than published data in seed peers.
impl HostProfile {
    pub fn new(url: Url) -> Self {
        HostProfile {
            url,
            ..Default::default()
        }
    }

    async fn check_sync(mut self) -> anyhow::Result<HostProfile> {
        // dbg!("check_sync", &self.url);
        let client = Client::new(self.url.clone());

        match client.get_index().await {
            Ok(res) => {
                self.version = res.into_inner().ledger_version.into();
                self.is_api = true;
            }
            Err(_) => {
                // not interested in the result just need to mark is as a failing api endpoint.
                self.is_api = false;
            }
        };

        Ok(self)
    }
}
#[serde_as]
#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
pub struct NetworkPlaylist {
    #[serde(default = "default_chain")]
    #[serde_as(as = "DisplayFromStr")]
    pub chain_id: NamedChain,
    pub nodes: Vec<HostProfile>,
}
fn default_chain() -> NamedChain {
    NamedChain::MAINNET
}

impl Default for NetworkPlaylist {
    fn default() -> Self {
        NetworkPlaylist {
            chain_id: NamedChain::MAINNET,
            nodes: vec![HostProfile::default()],
        }
    }
}

pub fn find_default_playlist(chain_id: Option<NamedChain>) -> anyhow::Result<Url> {
    let url: Url = match chain_id {
      Some(NamedChain::TESTNET) => {
      "https://raw.githubusercontent.com/0o-de-lally/seed-peers/main/fullnode_seed_playlist.json"
      .parse()?
    },
      _ => { // MAINNET and everything else
          "https://raw.githubusercontent.com/0o-de-lally/seed-peers/main/fullnode_seed_playlist.json"
          .parse()?
      },
    };
    Ok(url)
}

impl NetworkPlaylist {
    pub fn localhost(chain_name: Option<NamedChain>) -> Self {
      Self {
          chain_id: chain_name.unwrap_or(NamedChain::MAINNET),
          nodes: vec![HostProfile {
            url: "http://localhost:8080".parse().unwrap(),
            note: "localhost".to_owned(),
            version: 0,
            is_api: true,
            is_sync: true,
        }],
      }
    }
    pub async fn default_for_network(chain_id: Option<NamedChain>) -> anyhow::Result<Self> {
        if let Some(NamedChain::TESTING) = chain_id {
            return Ok(Self::testing(None));
        }
        let url = find_default_playlist(chain_id)?;

        Self::from_url(url, chain_id).await
    }

    pub async fn from_url(
        playlist_url: Url,
        chain_id: Option<NamedChain>,
    ) -> anyhow::Result<NetworkPlaylist> {
        let res = reqwest::get(playlist_url).await?;
        let out = res.text().await?;
        let mut play: NetworkPlaylist = serde_json::from_str(&out)?; //res.text()?.

        if let Some(c) = chain_id {
            play.chain_id = c;
        }

        Ok(play)
    }

    pub fn testing(url: Option<Url>) -> Self {
        let mut np = NetworkPlaylist::default();
        if let Some(u) = url {
            let h = np
                .nodes
                .iter_mut()
                .next()
                .expect("didn't find a hostprofile");
            h.url = u;
        }
        np.chain_id = NamedChain::TESTING;
        np
    }

    pub fn add_url(&mut self, url: Url) {
        let h = HostProfile::new(url);
        self.nodes.push(h);
    }

    pub fn shuffle_order(&mut self) {
        let urls_list = &mut self.nodes;
        let mut rng = thread_rng();
        urls_list.shuffle(&mut rng);
    }

    pub fn all_urls(&self) -> anyhow::Result<Vec<Url>> {
        let list_urls: Vec<Url> = self.nodes.iter().map(|e| e.url.to_owned()).collect();

        Ok(list_urls)
    }

    pub fn the_good_ones(&self) -> anyhow::Result<Vec<Url>> {
        let list_urls: Vec<Url> = self
            .nodes
            .iter()
            .filter_map(|e| {
                if e.is_sync && e.is_api {
                    Some(e.url.to_owned())
                } else {
                    None
                }
            })
            .collect();
        if list_urls.is_empty() {
            bail!("no verified nodes found")
        }
        Ok(list_urls)
    }

    pub fn the_best_one(&self) -> anyhow::Result<Url> {
        match self.the_good_ones()?.first() {
            Some(url) => Ok(url.clone()),
            None => bail!("Expected an URL for the best one"),
        }
    }

    pub fn pick_one(&self) -> anyhow::Result<Url> {
     match self.the_best_one() {
          Ok(u) => Ok(u),
          Err(_) => self
              .all_urls()?
              .into_iter()
              .next()
              .context("no urls to choose from"),
      }
    }



    pub async fn refresh_sync_status(&mut self) -> anyhow::Result<()> {
        // let _cfg = get_cfg()?;
        dbg!("check_which_are_synced");

        // randomize to balance load on carpe nodes
        //shuffle while we are here
        self.shuffle_order();

        let futures = FuturesUnordered::new();

        // TODO: remove clone
        self.nodes.clone().into_iter().for_each(|p| {
            futures.push(p.check_sync());
        });

        let sync_list = futures
            .filter_map(|e| async move { e.ok() })
            .collect::<Vec<HostProfile>>()
            .await;

        // find the RMS of the versions. Reject anything below rms
        let sum_squares: u64 = sync_list
            .iter()
            .map(|e| u64::pow(e.version, 2))
            .collect::<Vec<u64>>()
            .iter()
            .sum();

        let avg = sum_squares as f64 / sync_list.len() as f64;
        let rms = avg.sqrt();

        let mut checked: Vec<HostProfile> = sync_list
            .into_iter()
            .map(|mut p| {
                if p.version as f64 >= rms {
                    // there may be only one in list
                    p.is_sync = true
                }
                p
            })
            .collect();

        checked.sort_by_key(|p| p.version);
        self.nodes = checked;

        Ok(())
    }

    pub async fn check_which_are_alive(mut self) -> anyhow::Result<Self> {
        let mut upstream = self.nodes;

        // try getting waypoint from upstream nodes
        // no waypoint is necessary in advance.
        let futures = FuturesUnordered::new();

        // let mut list = cfg.profile.upstream_nodes.to_owned();
        upstream.shuffle(&mut thread_rng());

        // randomize to balance load on carpe nodes
        upstream.into_iter().for_each(|p| {
            futures.push(p.check_sync());
        });

        let checked = futures
            .filter_map(|e| async move { e.ok() })
            .collect::<Vec<_>>()
            .await;

        self.nodes = checked;
        Ok(self)
    }
}

// #[tokio::test]
// async fn test_parse_from_url() {
//   let url = find_default_playlist(NamedChain::MAINNET).unwrap();
//   let pl = NetworkPlaylist::from_url(url, N).await.unwrap();
//   dbg!(&pl);
// }
