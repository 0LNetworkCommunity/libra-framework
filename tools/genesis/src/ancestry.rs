use std::path::PathBuf;

use libra_types::legacy_types::legacy_address::LegacyAddress;
use serde::Deserialize;

#[derive(Debug, Clone)]
pub struct Ancestry {
  pub address: LegacyAddress,
  pub tree: Vec<LegacyAddress>
}

#[derive(Debug, Clone, Deserialize)]
pub struct JsonAncestry {
  address: LegacyAddress,
  tree: JsonTree,
}

#[derive(Debug, Clone, Deserialize)]

pub struct JsonTree {
  #[serde(default = "default_addr")]
  parent: LegacyAddress,
}

fn default_addr() -> LegacyAddress {
  LegacyAddress::from_hex_literal("0x666").unwrap()
}


pub fn parse_ancestry_json(path: PathBuf) -> anyhow::Result<Vec<JsonAncestry>>{
  let json_str = std::fs::read_to_string(path)?;
  Ok(serde_json::from_str(&json_str)?)
}

pub fn find_all_ancestors(my_account: &JsonAncestry, list: &Vec<JsonAncestry>) -> anyhow::Result<Vec<LegacyAddress>>{
  let mut my_ancestors: Vec<LegacyAddress> = vec![];
  let mut i = 0;
  let mut found = true;

  let mut parent_to_find_next = my_account.tree.parent;
  // my_ancestors.push(parent_to_find_next);

  while found && i < 100 {
    let parent_struct = list.iter()
    .find(|el|{
      el.address == parent_to_find_next
    });
    if let Some(p) = parent_struct {
      my_ancestors.push(p.address);
      parent_to_find_next = p.tree.parent;
    } else {
      found = false;
      break;
    }
    i+=1;
  }
  // need to reverse such that oldest is 0th.
  my_ancestors.reverse();
  Ok(my_ancestors)

}

pub fn map_ancestry(list: &Vec<JsonAncestry>) -> anyhow::Result<Vec<Ancestry>>{
  list.iter()
    .map(|el| {
      let tree = find_all_ancestors(el, list).unwrap_or(vec![]);
      Ok(Ancestry {
        address: el.address,
        tree,
      })
    })
  .collect()
}

#[test]
fn parse_file() {
    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/ancestry.json");
    let json_ancestry = parse_ancestry_json(p).unwrap();
    dbg!(&json_ancestry.len());
}

#[test]
fn test_find() {
    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/ancestry.json");
    let json_ancestry = parse_ancestry_json(p).unwrap();
    let all = find_all_ancestors(json_ancestry.iter().next().unwrap(), &json_ancestry).unwrap();
    // dbg!(&all);
    assert!(all.len() == 6);
    
}


#[test]
fn test_map() {
    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/ancestry.json");
    let json_ancestry = parse_ancestry_json(p).unwrap();
    let res = map_ancestry(&json_ancestry).unwrap();
    dbg!(res.len());
    dbg!(&res.iter().next());
    
}
