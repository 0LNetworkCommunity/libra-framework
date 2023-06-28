//! network configs
use crate::exports::{NamedChain, Client};
use futures::{stream::FuturesUnordered, StreamExt};
use anyhow::bail;
use anyhow::Context;
use rand::{seq::SliceRandom, thread_rng};
use serde::Deserialize;
use url::Url;

#[derive(Deserialize)]
/// A list of host information for upstream fullnodes serving RPC servers
pub struct FullnodePlaylist {
    ///
    pub nodes: Vec<HostInfo>,
}

#[derive(Deserialize)]
/// infor for the RPC peers connection.
pub struct HostInfo {
    ///
    pub note: String,
    ///
    pub url: Url,
}

/// try to fetch current fullnodes from a URL, or default to a seed peer list
pub async fn get_known_fullnodes(seed_url: Option<Url>) -> anyhow::Result<FullnodePlaylist> {
    let url = seed_url.unwrap_or(
        "https://raw.githubusercontent.com/0o-de-lally/seed-peers/main/fullnode_seed_playlist.json"
            .parse()
            .context("cannot parse playlist url")?,
    );

    FullnodePlaylist::http_fetch_playlist(url).await
}

impl FullnodePlaylist {
    /// use a URL to load a fullnode playlist
    pub async fn http_fetch_playlist(url: Url) -> anyhow::Result<FullnodePlaylist> {
        let res = reqwest::get(url).await?;
        let out = res.text().await?;
        let play: FullnodePlaylist = serde_json::from_str(&out)?; //res.text()?.parse()?;
        Ok(play)
    }

    /// extract the urls from the playlist struct
    pub fn get_urls(&self) -> Vec<Url> {
        self.nodes
            .iter()
            .filter_map(|a| Some(a.url.to_owned()))
            .collect()
    }

    // /// update the app configs 0L.toml file
    // pub fn update_config_file(&self, path: Option<PathBuf>) -> anyhow::Result<()> {

    //     let path = path.unwrap_or(default_config_path());
    //     let mut new_cfg = AppCfg::parse_toml(path)?;
    //     let mut peers = self.get_urls();
    //     let mut rng = thread_rng();
    //     peers.shuffle(&mut rng);

    //     new_cfg.profile.upstream_nodes = peers;

    //     new_cfg.save_file()?;
    //     Ok(())
    // }
}


#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub struct NetworkProfile {
  pub chain_id: NamedChain, // Todo, use the Network Enum
  pub urls: Vec<Url>,
  // pub waypoint: Waypoint,
  pub profile: String, // tbd, to use default node, or to use upstream, or a custom url.
}

// impl NetworkProfile {
//   pub fn read_from_cfg() -> anyhow::Result<Self> {
//     let cfg = get_cfg()?;
//     Ok(NetworkProfile {
//       chain_id: cfg.chain_info.chain_id,
//       urls: cfg.profile.upstream_nodes,
//       // waypoint: cfg.chain_info.base_waypoint.unwrap_or_default(),
//       profile: "default".to_string(),
//     })
//   }
// }

// #[derive(serde::Deserialize, serde::Serialize, Debug)]
// pub enum Networks {
//   Mainnet,
//   Tesnet, // REX

//   Custom { playlist_url: Url },
// }

// impl fmt::Display for Networks {
//   fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//     write!(f, "{:?}", self)
//     // or, alternatively:
//     // fmt::Debug::fmt(self, f)
//   }
// }

pub async fn set_network_configs(
  chain_id: NamedChain,
  custom_playlist: Option<Url>,
) -> anyhow::Result<NetworkProfile> {
  // dbg!("toggle network");
  // dbg!(&network);
  let playlist = if let Some(u) = custom_playlist {
    get_known_fullnodes(Some(u)).await?
  } else {
    // TODO: set the default playlists
    match chain_id {
      NamedChain::TESTNET => get_known_fullnodes(Some(
        "https://raw.githubusercontent.com/0o-de-lally/seed-peers/main/fullnode_seed_playlist.json"
          .parse()?
      )).await?,
      NamedChain::TESTING => {
        FullnodePlaylist {
          nodes: vec![HostInfo {
            note: "local".to_string(),
            url: Url::parse("http://localhost:8080").unwrap(),
          }],
        }
      },
      _ => { // MAINNET and everything else
          get_known_fullnodes(Some(
          "https://raw.githubusercontent.com/0o-de-lally/seed-peers/main/fullnode_seed_playlist.json"
            .parse()?
        )).await?
      }, // assume mainnet
    }
  };

  dbg!("playlist");

  
  // playlist.update_config_file(None)?; // None uses default path of 0L.toml

  // // TODO: I don't think chain ID needs to change.
  // set_chain_id(network).map_err(|e| {
  //   let err_msg = format!("could not set chain id, message: {}", &e.to_string());
  //   CarpeError::misc(&err_msg)
  // })?;

  let mut urls = playlist.get_urls();
  let mut rng = thread_rng();
  urls.shuffle(&mut rng);
  // dbg!("set_chain id");
  Ok(NetworkProfile {
    chain_id,
    urls,
    // waypoint: cfg.chain_info.base_waypoint.unwrap_or_default(),
    profile: "default".to_string(), // todo: option name
  })

  // NetworkProfile::read_from_cfg()
}


// fn get_swarm_url() -> anyhow::Result<Url>{
//   let app_cfg = get_cfg()?;
//   let (url, _) = ol_types::config::get_swarm_rpc_url(app_cfg.workspace.node_home.join("swarm_temp/"));
//   Ok(url)
// }

// fn get_swarm_playlist() -> anyhow::Result<FullnodePlaylist> {

//   let h = HostInfo {
//       note: "swarm".to_string(),
//       url: get_swarm_url()?,
//   };
  
//   let f = FullnodePlaylist {
//     nodes: vec![h],
//   };

//   Ok(f)
// }
// pub async fn set_waypoint_from_upstream() -> Result<AppCfg, Error> {
//   let cfg = get_cfg()?;

//   // try getting waypoint from upstream nodes
//   // no waypoint is necessary in advance.
//   let mut futures = FuturesUnordered::new();

//   let mut list = cfg.profile.upstream_nodes.to_owned();
//   list.shuffle(&mut thread_rng());

//   // randomize to balance load on carpe nodes
//   list.into_iter().for_each(|url| {
//     futures.push(waypoint::bootstrap_waypoint_from_rpc(url.to_owned()));
//   });

//   while !futures.is_empty() {
//     if let Some(wp) = futures.next().await {
//       match wp {
//         Ok(w) => {
//           set_waypoint(w)?;
//           return Ok(cfg);
//           // break
//         }
//         Err(_) => {}
//       }
//     }
//   }

//   bail!("no waypoint found while querying upstream nodes")
// }

// pub async fn set_waypoint_from_upstream() -> Result<AppCfg, Error> {
//   let prefs = read_preferences()?;
//   if let Some(upstream) = prefs.network {
//     let mut urls = upstream.the_good_ones()?;
//     urls.shuffle(&mut thread_rng());
//     if let Some(u) = urls.first() {
//       match waypoint::bootstrap_waypoint_from_rpc(u.to_owned()).await {
//         Ok(w) => set_waypoint(w),
//         Err(e) => Err(e),
//       }
//     } else {
//       bail!("cannot find a synced upstream URL")
//     }
//   } else {
//     bail!("could not fetch network stats from preferences.json")
//   }
// }

// /// Set the base_waypoint used for client connections.
// pub fn set_waypoint(wp: Waypoint) -> Result<AppCfg, Error> {
//   let mut cfg = get_cfg()?;
//   cfg.chain_info.base_waypoint = Some(wp);
//   cfg.save_file()?;
//   Ok(cfg)
// }

// /// Get all the 0L configs. For tx sending and upstream nodes
// /// Note: The default_node key in 0L is not used by Carpe. Carpe randomly tests
// /// all the endpoints in upstream_peers on every TX.
// pub fn override_upstream_node(url: Url) -> Result<AppCfg, Error> {
//   let mut cfg = get_cfg()?;
//   cfg.profile.upstream_nodes = vec![url];
//   cfg.save_file()?;
//   Ok(cfg)
// }

// // the 0L configs. For tx sending and upstream nodes
// pub fn set_chain_id(chain_id: NamedChain) -> Result<AppCfg, Error> {
//   let mut cfg = get_cfg()?;
//   cfg.chain_info.chain_id = chain_id;
//   cfg.save_file()?;
//   Ok(cfg)
// }

// /// Set the list of upstream nodes
// pub fn set_upstream_nodes(vec_url: Vec<Url>) -> Result<AppCfg, Error> {
//   let mut cfg = get_cfg()?;
//   cfg.profile.upstream_nodes = vec_url;
//   cfg.save_file()?;
//   Ok(cfg)
// }


#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]

pub struct UpstreamStats {
  nodes: Vec<FullnodeProfile>,
}

impl UpstreamStats {
  pub fn new(urls: Vec<Url>) -> Self {
    let nodes = urls
      .into_iter()
      .map(|u| FullnodeProfile {
        url: u,
        version: 0,
        is_api: false,
        is_sync: false,
      })
      .collect();
    UpstreamStats { nodes }
  }

  pub async fn refresh(self) -> anyhow::Result<Self> {
    self.check_which_are_synced().await
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

    Ok(list_urls)
  }

  pub fn the_best_one(&self) -> anyhow::Result<Url> {
    match self.the_good_ones()?.first() {
      Some(url) => Ok(url.clone()),
      None => bail!("Expected an URL for the best one"),
    }
  }

  pub async fn check_which_are_synced(mut self) -> anyhow::Result<Self> {
    // let _cfg = get_cfg()?;
    dbg!("check_which_are_synced");

    // try getting waypoint from upstream nodes
    // no waypoint is necessary in advance.
    let futures = FuturesUnordered::new();
    let mut upstream = self.nodes;
    // let mut list = cfg.profile.upstream_nodes.to_owned();
    upstream.shuffle(&mut thread_rng());

    // randomize to balance load on carpe nodes
    upstream.into_iter().for_each(|p| {
      futures.push(p.check_sync());
    });

    // dbg!(&list);

    let sync_list = futures
      .filter_map(|e| async move { e.ok() })
      .collect::<Vec<FullnodeProfile>>()
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

    let checked: Vec<FullnodeProfile> = sync_list
      .into_iter()
      // .sort_by(|p| {
      //   p.version
      // })
      .map(|mut p| {
        if p.version as f64 >= rms {
          // there may be only one in list
          p.is_sync = true
        }
        p
      })
      .collect();

    self.nodes = checked;
    Ok(self)
  }

  pub async fn check_which_are_alive(mut self) -> anyhow::Result<Self> {
    // let _cfg = get_cfg()?;
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

    // dbg!(&list);

    let checked = futures
      .filter_map(|e| async move { e.ok() })
      .collect::<Vec<_>>()
      .await;

    self.nodes = checked;
    Ok(self)
  }
}
#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
pub struct FullnodeProfile {
  url: Url,
  version: u64,
  is_api: bool,
  is_sync: bool,
}
/// from the list of seed_peers find the best peer to connect to.
/// First does a light port check on all peers, and eliminated unresponsive
/// Then from a random list fetches the first 3 nodes to respond with a waypoint.
/// picks the node with the highest waypoint.
/// moves that node othe top of the seed peers vector.
/// sets it in preferences as the default peer.
impl FullnodeProfile {
  async fn check_sync(mut self) -> anyhow::Result<FullnodeProfile> {
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

//   /// get the waypoint from a fullnode
//   pub async fn check_rpc_header(mut self) -> anyhow::Result<FullnodeProfile> {
//     self.is_api = false;

//     let client = ClientBuilder::new()
//       .timeout(Duration::from_secs(1))
//       .build()?;

//     // handle all errors as a is_api = false
//     match client.head(self.url.to_owned()).send().await {
//       Ok(resp) => match resp.text().await {
//         Ok(_) => {
//           self.is_api = true;
//           return Ok(self.to_owned());
//         }
//         Err(_) => {}
//       },
//       Err(_) => {}
//     };
//     Ok(self)
//   }
// }

#[test]
fn test_pick_upstream() {
  let node_good = FullnodeProfile {
    url: "http://165.232.136.149:8080/".parse().unwrap(),
    version: 0,
    is_api: false,
    is_sync: false,
  };

  let node_bad = FullnodeProfile {
    url: "http://165.232.136.14:8080/".parse().unwrap(),
    version: 0,
    is_api: false,
    is_sync: false,
  };

  let upstream = UpstreamStats {
    nodes: vec![node_good, node_bad],
  };

  tauri::async_runtime::block_on(UpstreamStats::check_which_are_alive(upstream)).unwrap();
}
