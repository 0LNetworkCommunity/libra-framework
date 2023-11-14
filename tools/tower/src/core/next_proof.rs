//! next proof

use crate::core::proof_preimage;

use anyhow::{bail, Error};
use serde::{Deserialize, Serialize};

use libra_types::{
    exports::Client,
    legacy_types::{
        app_cfg::AppCfg,
        block::{VDFProof, GENESIS_VDF_ITERATIONS, GENESIS_VDF_SECURITY_PARAM},
        vdf_difficulty::VDFDifficulty,
    },
};

use libra_query::{account_queries, chain_queries};

use diem_sdk::crypto::HashValue;

/// container for the next proof parameters to be fed to VDF prover.
#[derive(Clone, Debug, Deserialize, Serialize)]

pub struct NextProof {
    /// VDF difficulty and security params
    pub diff: VDFDifficulty,
    /// height of next proof
    pub next_height: u64,
    /// preimage to solve
    #[serde(with = "hex")]
    pub preimage: Vec<u8>,
}

impl NextProof {
    /// create a genesis proof
    pub fn genesis_proof(config: &AppCfg) -> anyhow::Result<Self> {
        // NOTE: can't set defautlsin VDFDifficulty::default() because of circular dependency
        let diff = VDFDifficulty {
            difficulty: *GENESIS_VDF_ITERATIONS,
            security: *GENESIS_VDF_SECURITY_PARAM,
            prev_diff: *GENESIS_VDF_ITERATIONS,
            prev_sec: *GENESIS_VDF_SECURITY_PARAM,
        };

        Ok(NextProof {
            diff,
            next_height: 0,
            preimage: proof_preimage::genesis_preimage(config)?,
        })
    }
}
/// return the VDF difficulty expected and the next tower height
pub fn get_next_proof_params_from_local(config: &AppCfg) -> Result<NextProof, Error> {
    // get the location of this miner's blocks
    let mut blocks_dir = config.workspace.node_home.clone();
    blocks_dir.push(&config.get_block_dir(None)?);
    let (current_local_block, _) = VDFProof::get_highest_block(&blocks_dir)?;
    let diff = VDFDifficulty {
        difficulty: current_local_block.difficulty(),
        security: current_local_block.security.unwrap(),
        prev_diff: current_local_block.difficulty(),
        prev_sec: current_local_block.security.unwrap(),
    };
    Ok(NextProof {
        diff,
        next_height: current_local_block.height + 1,
        preimage: HashValue::sha3_256_of(&current_local_block.proof).to_vec(),
    })
}

/// query the chain for parameters to use in the next VDF proof.
/// includes global parameters for difficulty
/// and individual parameters like tower height and the preimage (previous proof hash)
pub async fn get_next_proof_from_chain(
    app_cfg: &AppCfg,
    client: &Client,
) -> Result<NextProof, Error> {
    let (difficulty, security) = chain_queries::get_tower_difficulty(client).await?;

    let profile = app_cfg.get_profile(None)?;
    // get user's state
    let p = match account_queries::get_tower_state(client, profile.account).await {
        Ok(ts) => {
            NextProof {
                diff: VDFDifficulty {
                    difficulty,
                    security,
                    prev_diff: 0, // not relevant off chain
                    prev_sec: 0,  // not relevant off chain
                },
                next_height: ts.verified_tower_height + 1, // add one for next
                preimage: ts.previous_proof_hash,
            }
        }
        _ => bail!("cannot get tower resource for account"),
    };

    Ok(p)
}
