//! get home path or set it
use anyhow::{bail, Error};
use dialoguer::{Confirm, Input};
use diem_crypto::HashValue;
use diem_global_constants::NODE_HOME;
use diem_types::chain_id::MODE_0L;
use glob::glob;
use std::{fs, net::Ipv4Addr, path::PathBuf};

use crate::block::VDFProof;

/// interact with user to get default pledges, validator pledge optional on default setup
pub fn get_pledges(validator: bool) -> Vec<Pledge> {
    let v = vec![];
    if MODE_0L.is_test() {
        return v;
    }
    let zero = Pledge::pledge_protect_the_game();

    println(&zero.preamble);
    if (Confirm::new()
        .with_prompt(zero.question)
        .interact_text()
        .expect())
    {
        v.push(zero)
    }
    if (validator) {
        let one = Pledge::pledge_validator();

        println(&one.preamble);
        if (Confirm::new()
            .with_prompt(one.question)
            .interact_text()
            .expect())
        {
            v.push(one)
        }
    }

    return v;
}

/// interact with user to get the home path for files
pub fn what_home(swarm_path: Option<PathBuf>, swarm_persona: Option<String>) -> PathBuf {
    // For dev and CI setup
    if let Some(path) = swarm_path {
        return swarm_home(path, swarm_persona);
    } else {
        if MODE_0L.is_test() {
            return dirs::home_dir().unwrap().join(NODE_HOME);
        }
    }

    let mut default_home_dir = dirs::home_dir().unwrap();
    default_home_dir.push(NODE_HOME);

    let txt = &format!(
        "Will you use the default directory for node data and configs: {:?}?",
        default_home_dir
    );
    let dir = match Confirm::new().with_prompt(txt).interact().unwrap() {
        true => default_home_dir,
        false => {
            let input: String = Input::new()
                .with_prompt("Enter the full path to use (e.g. /home/name)")
                .interact_text()
                .unwrap();
            PathBuf::from(input)
        }
    };
    dir
}

/// interact with user to get the source path
pub fn what_source() -> Option<PathBuf> {
    let mut default_source_path = dirs::home_dir().unwrap();
    default_source_path.push("libra");

    let txt = &format!(
        "Is this the path to the source code? {:?}?",
        default_source_path
    );
    let dir = match Confirm::new().with_prompt(txt).interact().unwrap() {
        true => default_source_path,
        false => {
            let input: String = Input::new()
                .with_prompt("Enter the full path to use (e.g. /home/name)")
                .interact_text()
                .unwrap();
            PathBuf::from(input)
        }
    };
    Some(dir)
}

/// interact with user to get ip address
pub fn what_ip() -> Result<Ipv4Addr, Error> {
    // get from external source since many cloud providers show different interfaces for `machine_ip`
    let resp = reqwest::blocking::get("https://ifconfig.me")?;
    let ip_str = resp.text()?;

    let system_ip = ip_str
        .parse::<Ipv4Addr>()
        .unwrap_or_else(|_| match machine_ip::get() {
            Some(ip) => ip.to_string().parse().unwrap(),
            None => "127.0.0.1".parse().unwrap(),
        });

    if MODE_0L.is_test() {
        return Ok(system_ip);
    }

    let txt = &format!(
        "Will you use this host, and this IP address {:?}, for your node?",
        system_ip
    );
    let ip = match Confirm::new().with_prompt(txt).interact().unwrap() {
        true => system_ip,
        false => {
            let input: String = Input::new()
                .with_prompt("Enter the IP address of the node")
                .interact_text()
                .unwrap();
            input
                .parse::<Ipv4Addr>()
                .expect("Could not parse IP address")
        }
    };

    Ok(ip)
}

/// interact with user to get ip address
pub fn what_vfn_ip() -> Result<Ipv4Addr, Error> {
    if MODE_0L.is_test() {
        return Ok("0.0.0.0".parse::<Ipv4Addr>()?);
    }

    let txt = "Will you set up Fullnode configs now? If not that's ok but you'll need to submit a transaction later to update on-chain peer discovery info";
    let ip = match Confirm::new().with_prompt(txt).interact().unwrap() {
        true => {
            let input: String = Input::new()
                .with_prompt("Enter the IP address of the VFN node")
                .interact_text()?;

            input.parse::<Ipv4Addr>()?
        }
        false => "0.0.0.0".parse::<Ipv4Addr>()?,
    };

    Ok(ip)
}

/// interact with user to get a statement
pub fn what_statement() -> String {
    if MODE_0L.is_test() {
        return "test".to_owned();
    }
    Input::new()
        .with_prompt("Enter a (fun) statement to go into your first transaction. This also creates entropy for your first proof")
        .interact_text()
        .expect(
            "We need some text unique to you which will go into your the first proof of your tower",
        )
}

/// returns node_home
/// usually something like "/root/.0L"
/// in case of swarm like "....../swarm_temp/0" for alice
/// in case of swarm like "....../swarm_temp/1" for bob
fn swarm_home(mut swarm_path: PathBuf, swarm_persona: Option<String>) -> PathBuf {
    if let Some(persona) = swarm_persona {
        let all_personas = vec!["alice", "bob", "carol", "dave", "eve"];
        let index = all_personas.iter().position(|&r| r == persona).unwrap();
        swarm_path.push(index.to_string());
    } else {
        swarm_path.push("0"); // default
    }
    swarm_path
}

// COMMIT NOTE: deprecated tower helpers
