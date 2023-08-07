//! Proof block datastructure

use crate::legacy_types::{app_cfg::AppCfg, mode_ol::MODE_0L};

use crate::exports::NamedChain;

use anyhow::{bail, Result, Context};
use glob::glob;
use hex;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::{fs, io::Write, path::{PathBuf, Path}};
// use std::

// TOWER DIFFICULTY SETTINGS
// What we call "difficulty", is the intersection of number of VDF iterations with the security parameter.

// V6: Difficulty updated in V6
// see benchmarking research summary here: ol/documentation/tower/difficulty_benchmarking.md

/// The VDF security parameter.
pub static GENESIS_VDF_SECURITY_PARAM: Lazy<u64> = Lazy::new(|| {
    match *MODE_0L {
        NamedChain::MAINNET => 350,
        NamedChain::TESTNET => 350, // TODO: Do we want a different one?
        _ => 350,
    }
});

/// name of the proof files
pub const FILENAME: &str = "proof";

/// The VDF iterations. Combined with security parameter we have the "difficulty".
pub static GENESIS_VDF_ITERATIONS: Lazy<u64> = Lazy::new(|| {
    match *MODE_0L {
        // Difficulty updated in V6
        // see ol/documentation/tower/difficulty_benchmarking.md
        NamedChain::MAINNET => 3_000_000_000, // 3 billion, ol/documentation/tower/difficulty_benchmarking.md
        NamedChain::TESTNET => 100,
        _ => 100,
    }
});

/// Data structure and serialization of 0L delay proof.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct VDFProof {
    /// Proof Height
    pub height: u64,
    /// Elapsed Time in seconds
    pub elapsed_secs: u64,
    /// VDF input preimage. AKA challenge
    #[serde(with = "hex")]
    pub preimage: Vec<u8>,
    /// VDF proof. AKA solution
    #[serde(with = "hex")]
    pub proof: Vec<u8>,
    /// The iterations of the circuit
    pub difficulty: Option<u64>, // option to make backwards compatible reads
    /// the security parameter of the proof.
    pub security: Option<u64>,
}

impl VDFProof {
    /// Extract the preimage and proof from a genesis proof proof_0.json
    pub fn get_genesis_tx_data(path: &PathBuf) -> Result<(Vec<u8>, Vec<u8>), std::io::Error> {
        let file = std::fs::File::open(path)?;
        let reader = std::io::BufReader::new(file);
        let block: VDFProof =
            serde_json::from_reader(reader).expect("Genesis block should deserialize");
        Ok((block.preimage, block.proof))
    }

    // /// new object deserialized from file
    // pub fn parse_block_file(path: PathBuf) -> Result<VDFProof, anyhow::Error> {
    //     let file = fs::File::open(&path)?;
    //     let reader = BufReader::new(file);
    //     Ok(serde_json::from_reader(reader)?)
    // }

    /// get the difficulty/iterations of the block, or assume legacy
    pub fn difficulty(&self) -> u64 {
        self.difficulty.unwrap() // if the block doesn't have this info, assume it's legacy block.
    }

    /// get the security param of the block, or assume legacy
    pub fn security(&self) -> u64 {
        self.security.unwrap() // if the block doesn't have this info, assume it's legacy block.
    }

    pub fn write_json(&self, blocks_dir: &PathBuf) -> Result<PathBuf, std::io::Error> {
        if !&blocks_dir.exists() {
            // first run, create the directory if there is none, or if the user changed the configs.
            // note: user may have blocks but they are in a different directory than what AppCfg says.
            fs::create_dir_all(blocks_dir)?;
        };
        // Write the file.
        let mut latest_block_path = blocks_dir.clone();
        latest_block_path.push(format!("{}_{}.json", FILENAME, self.height));
        let mut file = fs::File::create(&latest_block_path)?;
        file.write_all(serde_json::to_string(&self)?.as_bytes())?;
        Ok(latest_block_path)
    }

    /// Parse a proof_x.json file and return a VDFProof
    pub fn parse_block_file(path: &PathBuf, purge_if_bad: bool) -> Result<Self> {
        let block_file = fs::read_to_string(path)?;

        match serde_json::from_str(&block_file) {
            Ok(v) => Ok(v),
            Err(e) => {
                if purge_if_bad {
                    fs::remove_file(&block_file)?
                }
                bail!(
                    "Could not read latest block file in path {:?}, message: {:?}",
                    &path,
                    e
                )
            }
        }
    }

    /// Parse a proof_x.json file and return a VDFProof
    pub fn get_proof_number(num: u64, blocks_dir: &Path) -> Result<(Self, PathBuf)> {
        let file = PathBuf::from(&format!(
            "{}/{}_{}.json",
            blocks_dir.display(),
            FILENAME,
            num
        ));
        match Self::parse_block_file(&file, false) {
            Ok(p) => {
                if p.height == num {
                    Ok((p, file))
                } else {
                    bail!(
                        "file {} does not contain proof height {}, found {} instead",
                        file.to_str().unwrap(),
                        num,
                        p.height
                    );
                }
            }
            Err(e) => Err(e),
        }
    }

    /// find the most recent proof on disk
    pub fn get_latest_proof(config: &AppCfg, purge_if_bad: bool) -> Result<Self> {
        let (_current_block_number, current_block_path) =
            Self::get_highest_block(&config.get_block_dir(None)?)?;

        Self::parse_block_file(&current_block_path, purge_if_bad)
    }

    /// parse the existing blocks in the miner's path. This function receives any path. Note: the path is configured in miner.toml which abscissa Configurable parses, see commands.rs.
    pub fn get_highest_block(blocks_dir: &Path) -> Result<(Self, PathBuf)> {
        let mut max_block: Option<VDFProof> = None;
        let mut max_block_path: Option<PathBuf> = None;

        let file_list = glob(&format!("{}/{}_*.json", blocks_dir.display(), FILENAME))?;
        // iterate through all json files in the directory.
        // if file_list.last().is_none() {
        //   bail!("cannot find any VDF proof files in, {:?}", blocks_dir);
        // }

        for entry in file_list.flatten() {
            // if let Ok(entry) = entry {
                // let file = fs::File::open(&entry).expect("Could not open block file");
                // let reader = BufReader::new(file);
                let block = match VDFProof::parse_block_file(&entry, false) {
                    Ok(v) => v,
                    Err(e) => {
                        println!("could not parse the proof file: {}, skipping. Manually delete if this proof is not readable.", e);
                        continue;
                    }
                };

                let blocknumber = block.height;

                if let Some(b) = &max_block {
                    if blocknumber > b.height {
                        max_block = Some(block);
                        max_block_path = Some(entry);
                    }
                } else {
                    max_block = Some(block);
                    max_block_path = Some(entry);
                }
        }

        let err_msg = "cannot find a valid VDF proof in files to determine next proof's parameters. Exiting.";

        Ok((max_block.context(err_msg)?, max_block_path.context(err_msg)?))
    }
}
