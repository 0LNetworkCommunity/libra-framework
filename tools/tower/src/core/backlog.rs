//! Miner resubmit backlog transactions module
#![forbid(unsafe_code)]

use crate::core::{garbage_collection::gc_failed_proof, tower_error};

use anyhow::{anyhow, bail, Error, Result};
use std::path::PathBuf;

use libra_query::account_queries;
use libra_txs::submit_transaction::Sender;

// use diem_client::BlockingClient as DiemClient;
// use diem_logger::prelude::*;
use libra_types::{
    exports::Client,
    legacy_types::{app_cfg::AppCfg, block::VDFProof},
    type_extensions::client_ext::ClientExt,
};

const EPOCH_MINING_THRES_UPPER: u64 = 72;
/// Submit a backlog of blocks that may have been mined while network is offline.
/// Likely not more than 1.
pub async fn process_backlog(config: &AppCfg) -> anyhow::Result<()> {
    // Getting local state height
    let mut blocks_dir = config.workspace.node_home.clone();
    blocks_dir.push(&config.get_block_dir(None)?);

    let (current_local_proof, current_block_path) = VDFProof::get_highest_block(&blocks_dir)?;

    let current_proof_number = current_local_proof.height;

    println!("Local tower height: {:?}", current_proof_number);
    if current_proof_number == 0 {
        // if we are at genesis
        return submit_or_delete(config, current_local_proof, current_block_path).await;
    }

    let mut i = 0;
    let mut remaining_in_epoch = EPOCH_MINING_THRES_UPPER;

    // Getting remote miner state
    // there may not be any onchain state.
    if let Ok((remote_height, proofs_in_epoch)) = get_remote_tower_height(config).await {
        println!("Remote tower height: {}", remote_height);
        println!("Proofs already submitted in epoch: {}", proofs_in_epoch);

        if remote_height < 1 || current_proof_number > remote_height {
            i = remote_height + 1;

            // use i64 for safety
            if proofs_in_epoch >= EPOCH_MINING_THRES_UPPER {
                println!(
                    "Backlog: Maximum number of proofs sent this epoch {}, exiting.",
                    EPOCH_MINING_THRES_UPPER
                );
                return Err(anyhow!(
                    "cannot submit more proofs than allowed in epoch, aborting backlog."
                ));
            }

            if proofs_in_epoch > 0 {
                remaining_in_epoch = EPOCH_MINING_THRES_UPPER - proofs_in_epoch
            }
        }
    }

    let mut submitted_now = 1u64;

    println!("Backlog: resubmitting missing proofs. Remaining in epoch: {}, already submitted in this backlog: {}", remaining_in_epoch, submitted_now);

    while i <= current_proof_number && submitted_now <= remaining_in_epoch {
        println!("submitting proof {}, in this backlog: {}", i, submitted_now);

        let (block, path) = VDFProof::get_proof_number(i, &blocks_dir)?;

        submit_or_delete(config, block, path).await?;

        i += 1;
        submitted_now += 1;
    }
    Ok(())
}

pub async fn submit_or_delete(config: &AppCfg, block: VDFProof, path: PathBuf) -> Result<()> {
    // TODO: allow user to set a profile
    dbg!("get sender");
    let mut sender = Sender::from_app_cfg(config, None).await?;

    dbg!("try commit proof");
    sender.commit_proof(block.clone()).await?;

    match sender.eval_response() {
        Ok(_) => Ok(()),
        Err(e) => {
            eprintln!(
                "WARN: could not fetch TX status, aborting. Message: {:?} ",
                &e
            );
            // evaluate type of error and maybe garbage collect
            match tower_error::parse_error(e.clone()) {
                tower_error::TowerError::WrongDifficulty => gc_failed_proof(config, path)?,
                tower_error::TowerError::Discontinuity => gc_failed_proof(config, path)?,
                tower_error::TowerError::Invalid => gc_failed_proof(config, path)?,
                _ => {}
            }
            bail!("Cannot submit tower proof: {:?}", e);
        }
    }
}
// /// submit an exact proof height
// pub fn submit_proof_by_number(
//     config: &AppCfg,
//     tx_params: &TxParams,
//     proof_to_submit: u64,
// ) -> Result<(), TxError> {
//     // Getting remote miner state
//     // there may not be any onchain state.
//     match get_remote_tower_height(tx_params)? {
//         Some((remote_height, _proofs_in_epoch)) => {
//             println!("Remote tower height: {}", remote_height);

//             if remote_height > 0 && proof_to_submit <= remote_height as u64 {
//                 warn!(
//                     "unable to submit proof - remote tower height is higher than or equal to {:?}",
//                     proof_to_submit
//                 );
//                 return Ok(());
//             }
//         },
//         None => {
//             if proof_to_submit != 0 {
//                 warn!(
//                     "unable to submit proof - remote tower state is not initiliazed. Sent proof 0 first"
//                 );
//                 return Ok(());
//             }
//         }
//     }

//     // Getting local state height
//     let mut blocks_dir = config.workspace.node_home.clone();
//     blocks_dir.push(&config.workspace.block_dir);
//     let (current_local_proof, _current_block_path) = VDFProof::get_highest_block(&blocks_dir)?;
//     let current_proof_number = current_local_proof.height;
//     // if let Some(current_proof_number) = current_local_proof {
//     println!("Local tower height: {:?}", current_proof_number);

//     if proof_to_submit > current_proof_number {
//         warn!(
//             "unable to submit proof - local tower height is smaller than {:?}",
//             proof_to_submit
//         );
//         return Ok(());
//     }

//     println!("Backlog: submitting proof {:?}", proof_to_submit);

//     let path = PathBuf::from(format!(
//         "{}/{}_{}.json",
//         blocks_dir.display(),
//         FILENAME,
//         proof_to_submit
//     ));
//     let file = File::open(&path).map_err(|e| Error::from(e))?;

//     let reader = BufReader::new(file);
//     let block: VDFProof = serde_json::from_reader(reader).map_err(|e| Error::from(e))?;

//     let view = commit_proof(&tx_params, block)?;
//     match eval_tx_status(view) {
//         Ok(_) => {}
//         Err(e) => {
//             warn!(
//                 "WARN: could not fetch TX status, aborting. Message: {:?} ",
//                 e
//             );
//             return Err(e);
//         } // };
//     }
//     Ok(())
// }

/// display the user's tower backlog
pub async fn show_backlog(config: &AppCfg) -> Result<()> {
    // Getting remote miner state
    // there may not be any onchain state.
    match get_remote_tower_height(config).await {
        Ok((remote_height, _proofs_in_epoch)) => {
            println!("Remote tower height: {}", remote_height);
        }
        _ => {
            println!("Remote tower state not initialized");
        }
    }

    // Getting local state height
    let mut blocks_dir = config.workspace.node_home.clone();
    blocks_dir.push(&config.get_block_dir(None)?);
    let (current_local_proof, _current_block_path) = VDFProof::get_highest_block(&blocks_dir)?;

    println!("Local tower height: {:?}", current_local_proof.height);

    Ok(())
}

/// returns the chain's height as a tuple of (total tower height, current proofs in epoch)
pub async fn get_remote_tower_height(app_cfg: &AppCfg) -> Result<(u64, u64), Error> {
    let (client, _) = Client::from_libra_config(app_cfg, None).await?;
    let profile = app_cfg.get_profile(None)?;
    let ts = account_queries::get_tower_state(&client, profile.account.to_owned()).await?;

    Ok((ts.verified_tower_height, ts.count_proofs_in_epoch))
}
