//! Proof block datastructure

use crate::core::{
  next_proof::{self, NextProof},
  backlog, delay::*, proof_preimage::genesis_preimage
};

use anyhow::Error;
use libra_types::legacy_types::block::{VDFProof, GENESIS_VDF_SECURITY_PARAM, GENESIS_VDF_ITERATIONS};
use libra_types::{
  legacy_types::app_cfg::AppCfg,
  exports::Client,
  type_extensions::client_ext::ClientExt,
};

use std::{fs, path::PathBuf, time::Instant};

// writes a JSON file with the first vdf proof
fn mine_genesis(config: &AppCfg, difficulty: u64, security: u64) -> anyhow::Result<VDFProof> {
    println!("Mining Genesis Proof"); // TODO: use logger
    let preimage = genesis_preimage(&config)?;
    let now = Instant::now();

    let proof = do_delay(&preimage, difficulty, security).unwrap(); // Todo: make mine_genesis return a result.
    let elapsed_secs = now.elapsed().as_secs();
    println!("Delay: {:?} seconds", elapsed_secs);
    let block = VDFProof {
        height: 0u64,
        elapsed_secs,
        preimage,
        proof,
        difficulty: Some(difficulty),
        security: Some(security),
    };

    Ok(block)
}

/// Mines genesis and writes the file
pub fn write_genesis(config: &AppCfg) -> anyhow::Result<VDFProof> {
    let difficulty = GENESIS_VDF_ITERATIONS.clone();
    let security = GENESIS_VDF_SECURITY_PARAM.clone();
    let block = mine_genesis(config, difficulty, security)?;
    //TODO: check for overwriting file...
    // write_json(&block, &config.get_block_dir())?;
    let path = block.write_json(&config.get_block_dir(None)?)?;
    // let genesis_proof_filename = &format!("{}_0.json", FILENAME);
    println!(
        "proof zero mined, file saved to: {:?}",
        &path
    );
    Ok(block)
}
/// Mine one block
pub fn mine_once(config: &AppCfg, next: NextProof) -> Result<VDFProof, Error> {
    let now = Instant::now();
    let data = do_delay(&next.preimage, next.diff.difficulty, next.diff.security)?;
    let elapsed_secs = now.elapsed().as_secs();
    println!("Delay: {:?} seconds", elapsed_secs);

    let block = VDFProof {
        height: next.next_height,
        elapsed_secs,
        preimage: next.preimage,
        proof: data.clone(),
        difficulty: Some(next.diff.difficulty),
        security: Some(next.diff.security),
    };

    block.write_json( &config.get_block_dir(None)?)?;
    Ok(block)
}

/// Write block to file
pub async fn mine_and_submit(
    config: &mut AppCfg,
    // tx_params: TxParams,
    local_mode: bool,
    // _swarm_path: Option<PathBuf>,
) -> Result<(), Error> {
    // get the location of this miner's blocks
    let mut blocks_dir = config.workspace.node_home.clone();
    blocks_dir.push(&config.get_block_dir(None)?);
    let (client, _) = Client::from_libra_config(config, None).await?;

    loop {
        get_next_and_mine(config, &client, local_mode).await?;
        // // the default behavior is to fetch info from the chain to produce the next proof, including dynamic params for VDF difficulty.
        // // if the user is offline, they must use local mode
        // // however the user may end up using stale config proofs if the epoch changes and the params are different now.

        // let next = match local_mode {
        //     true => next_proof::get_next_proof_params_from_local(config)?,
        //     false => {
        //         // let client = client::find_a_remote_jsonrpc(
        //         //     &config,
        //         //     // config.get_waypoint(swarm_path.clone())?, // 0L todo
        //         // )?;
        //         match next_proof::get_next_proof_from_chain(config, &client).await {
        //             Ok(n) => n,
        //             // failover to local mode, if no onchain data can be found.
        //             // TODO: this is important for migrating to the new protocol.
        //             // in future versions we should remove this since we may be producing bad proofs, and users should explicitly choose to use local mode.
        //             Err(_) => next_proof::get_next_proof_params_from_local(config)?,
        //         }
        //     }
        // };

        // println!("Mining VDF Proof # {}", next.next_height);
        // println!(
        //     "difficulty: {}, security: {}",
        //     next.diff.difficulty, next.diff.security
        // );

        // let block = mine_once(&config, next)?;

        // submits backlog to client
        match backlog::process_backlog(&config).await {
            Ok(()) => println!("Success: Proof committed to chain"),
            Err(e) => {
                // don't stop on tx errors
                println!("ERROR: Failed processing backlog, message: {:?}", e);
            }
        }
    }
}

pub async fn get_next_and_mine(config: &AppCfg, client: &Client, local_mode: bool) -> anyhow::Result<VDFProof>{
        // the default behavior is to fetch info from the chain to produce the next proof, including dynamic params for VDF difficulty.
        // if the user is offline, they must use local mode
        // however the user may end up using stale config proofs if the epoch changes and the params are different now.

        let next = match local_mode {
            true => next_proof::get_next_proof_params_from_local(config)?,
            false => {
                match next_proof::get_next_proof_from_chain(config, &client).await {
                    Ok(n) => n,
                    // failover to local mode, if no onchain data can be found.
                    // TODO: this is important for migrating to the new protocol.
                    // in future versions we should remove this since we may be producing bad proofs, and users should explicitly choose to use local mode.
                    Err(_) => next_proof::get_next_proof_params_from_local(config)?,
                }
            }
        };

        println!("Mining VDF Proof # {}", next.next_height);
        println!(
            "difficulty: {}, security: {}",
            next.diff.difficulty, next.diff.security
        );

        let block = mine_once(&config, next)?;

        println!(
            "Proof mined: proof_{}.json created.",
            block.height.to_string()
        );

        Ok(block)
}




/* ////////////// */
/* / Unit tests / */
/* ////////////// */

// Tests generate side-effects. For now run sequentially with `cargo test -- --test-threads 1`
#[allow(dead_code)]
fn test_helper_clear_block_dir(blocks_dir: &PathBuf) {
    // delete the temporary test file and directory.
    // remove_dir_all is scary: be careful with this.
    if blocks_dir.exists() {
        fs::remove_dir_all(blocks_dir).unwrap();
    }
}
#[test]
#[ignore]
//Not really a test, just a way to generate fixtures.
fn create_fixtures() {
    use std::io::Write;
    use toml;

    // if no file is found, the block height is 0
    //let blocks_dir = Path::new("./test_blocks");
    for i in 0..6 {
        let ns = i.to_string();

        // let mnemonic_string = wallet.mnemonic(); //wallet.mnemonic()
        let save_to = format!("./test_fixtures_{}/", ns);
        fs::create_dir_all(save_to.clone()).unwrap();
        let configs_fixture = test_make_configs_fixture();
        // configs_fixture.workspace.block_dir = save_to.clone();

        // mine to save_to path
        write_genesis(&configs_fixture).unwrap();

        // create miner.toml
        //rename the path for actual fixtures
        // configs_fixture.workspace.block_dir = "vdf_proofs".to_string();
        let toml = toml::to_string(&configs_fixture).unwrap();
        let mut toml_path = PathBuf::from(save_to);
        toml_path.push("miner.toml");
        let file = fs::File::create(&toml_path);
        file.unwrap()
            .write(&toml.as_bytes())
            .expect("Could not write toml");
    }
}

#[test]
fn test_mine_once() {
    use libra_types::legacy_types::{
      block::VDFProof,
    };
    use zapatos_sdk::crypto::HashValue;
    use libra_types::legacy_types::vdf_difficulty::VDFDifficulty;
    use hex::decode;

    // if no file is found, the block height is 0
    let configs_fixture = test_make_configs_fixture();
    // configs_fixture.workspace.block_dir = "test_blocks_temp_2".to_owned();

    // Clear at start. Clearing at end can pollute the path when tests fail.
    test_helper_clear_block_dir(&configs_fixture.get_block_dir(None).unwrap());

    let fixture_previous_proof = decode("0016f43606b957ab9d93046cdffa73a1e6be4f21f3848eb7b55b81756f7d31919affef388c0d92ca7d68232de4fea46884186c23ef1d6c86f63f5c586000048bce05").unwrap();

    let fixture_block = VDFProof {
        height: 0u64, // Tower height
        elapsed_secs: 0u64,
        preimage: Vec::new(),
        proof: fixture_previous_proof,
        difficulty: Some(100),
        security: Some(512),
    };

    fixture_block.write_json(&configs_fixture.get_block_dir(None).unwrap()).unwrap();

    let next = NextProof {
        next_height: fixture_block.height + 1,
        preimage: HashValue::sha3_256_of(&fixture_block.proof).to_vec(),
        diff: VDFDifficulty {
            difficulty: 100,
            security: 512,
            prev_diff: 100,
            prev_sec: 512,
        },
    };

    mine_once(&configs_fixture, next).unwrap();
    // confirm this file was written to disk.
    let block_file = fs::read_to_string("./test_blocks_temp_2/proof_1.json")
        .expect("Could not read latest block");
    let latest_block: VDFProof =
        serde_json::from_str(&block_file).expect("could not deserialize latest block");
    // Test the file is read, and blockheight is 0
    assert_eq!(latest_block.height, 1, "Not the droid you are looking for.");

    // Test the expected proof is writtent to file correctly.
    let correct_proof = "006036397bd5c35644e2b20f2334a5343911de7cf29588654c322c0fc063c1a2c50000bc9923bdb96a97beaf67f3530ad00f735b7a795ea651f6a88cfd4deeb5aa29";
    assert_eq!(
        hex::encode(&latest_block.proof),
        correct_proof,
        "Not the proof of the new block created"
    );

    test_helper_clear_block_dir(&configs_fixture.get_block_dir(None).unwrap());
}

#[test]
fn test_mine_genesis() {
    // if no file is found, the block height is 0
    //let blocks_dir = Path::new("./test_blocks");
    let configs_fixture = test_make_configs_fixture();

    //clear from sideffects.
    test_helper_clear_block_dir(&configs_fixture.get_block_dir(None).unwrap());

    // mine
    write_genesis(&configs_fixture).unwrap();
    // read file
    let block_file =
        // TODO: make this work: let latest_block_path = &configs_fixture.chain_info.block_dir.to_string().push(format!("proof_0.json"));
        fs::read_to_string("./test_blocks_temp_1/proof_0.json").expect("Could not read latest block");

    let latest_block: VDFProof =
        serde_json::from_str(&block_file).expect("could not deserialize latest block");

    // Test the file is read, and blockheight is 0
    assert_eq!(latest_block.height, 0, "test");

    // Test the expected proof is writtent to file correctly.
    let correct_proof = "035117e66d23e3db4198ef29b37181a542f5a71cbde6fcbace201c2023b7cf561d762a04799605da5734f291";
    assert_eq!(hex::encode(&latest_block.proof), correct_proof, "test");

    test_helper_clear_block_dir(&configs_fixture.get_block_dir(None).unwrap());
}

#[test]
fn test_parse_no_files() {
    // if no file is found, the block height is 0
    let blocks_dir = PathBuf::from(".");

    match VDFProof::get_highest_block(&blocks_dir) {
        Ok(_) => assert!(false),
        Err(_) => assert!(true),
    }
}

#[test]
fn test_parse_one_file() {
    use std::io::Write;
    // create a file temporarily in ./test_blocks with height 33
    let current_block_number = 33;
    let block = VDFProof {
        height: current_block_number,
        elapsed_secs: 0u64,
        preimage: Vec::new(),
        proof: Vec::new(),
        difficulty: Some(100),
        security: Some(2048),
    };

    // write the file temporarilty
    let blocks_dir = PathBuf::from("./test_blocks_temp_3");
    // Clear at start. Clearing at end can pollute the path when tests fail.
    test_helper_clear_block_dir(&blocks_dir);

    fs::create_dir(&blocks_dir).unwrap();
    let mut latest_block_path = blocks_dir.clone();
    latest_block_path.push(format!("proof_{}.json", current_block_number));
    let mut file = fs::File::create(&latest_block_path).unwrap();
    file.write_all(serde_json::to_string(&block).unwrap().as_bytes())
        .expect("Could not write block");

    // block height
    assert_eq!(VDFProof::get_highest_block(&blocks_dir).unwrap().0.height, 33);

    test_helper_clear_block_dir(&blocks_dir)
}

#[cfg(test)]
/// make fixtures for file
pub fn test_make_configs_fixture() -> AppCfg {
    // use libra_types::exports::NamedChain;

    let cfg = AppCfg::default();
    let mut profile = cfg.get_profile(None).unwrap();
    // cfg.workspace.node_home = PathBuf::from(".");
    // cfg.workspace.block_dir = "test_blocks_temp_1".to_owned();
    // cfg.chain_info.chain_id = NamedChain::TESTNET;
    profile.auth_key = "3e4629ba1e63114b59a161e89ad4a083b3a31b5fd59e39757c493e96398e4df2"
        .parse()
        .unwrap();
    cfg
}
