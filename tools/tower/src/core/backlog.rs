//! Miner resubmit backlog transactions module
#![forbid(unsafe_code)]

use crate::core::{
  garbage_collection::gc_failed_proof,
  tower_error,
};

use anyhow::{anyhow, bail, Error, Result};
use std::path::PathBuf;


use libra_txs::submit_transaction::Sender;
use libra_query::{account_queries, get_client};
// use diem_client::BlockingClient as DiemClient;
// use diem_logger::prelude::*;
use libra_types::legacy_types::{
  app_cfg::AppCfg,
  block::VDFProof,
  tx_error::TxError,
};


const EPOCH_MINING_THRES_UPPER: u64 = 72;
/// Submit a backlog of blocks that may have been mined while network is offline.
/// Likely not more than 1.
pub async fn process_backlog(config: &AppCfg) -> anyhow::Result<()> {
    // Getting local state height
    let mut blocks_dir = config.workspace.node_home.clone();
    blocks_dir.push(&config.workspace.block_dir);

    let (current_local_proof, current_block_path) = VDFProof::get_highest_block(&blocks_dir)?;

    let current_proof_number = current_local_proof.height;

    println!("Local tower height: {:?}", current_proof_number);

    if current_proof_number == 0 { // if we are at genesis
      return submit_or_delete(config, current_local_proof, current_block_path).await
    }


    let mut i = 0;
    let mut remaining_in_epoch = EPOCH_MINING_THRES_UPPER;

    // Getting remote miner state
    // there may not be any onchain state.
    if let Some((remote_height, proofs_in_epoch)) = get_remote_tower_height(config).await.ok() {
        println!("Remote tower height: {}", remote_height);
        println!("Proofs already submitted in epoch: {}", proofs_in_epoch);

        if remote_height < 1 || current_proof_number > remote_height {
            i = remote_height as u64 + 1;

            // use i64 for safety
            if !(proofs_in_epoch < EPOCH_MINING_THRES_UPPER) {
                println!(
                    "Backlog: Maximum number of proofs sent this epoch {}, exiting.",
                    EPOCH_MINING_THRES_UPPER
                );
                return Err(anyhow!(
                    "cannot submit more proofs than allowed in epoch, aborting backlog."
                )
                .into());
            }

            if proofs_in_epoch > 0 {
                remaining_in_epoch = EPOCH_MINING_THRES_UPPER - proofs_in_epoch as u64
            }
        }
    }

    let mut submitted_now = 1u64;

    println!("Backlog: resubmitting missing proofs. Remaining in epoch: {}, already submitted in this backlog: {}", remaining_in_epoch, submitted_now);

    while i <= current_proof_number && submitted_now <= remaining_in_epoch {
        println!("submitting proof {}, in this backlog: {}", i, submitted_now);

        let (block, path) = VDFProof::get_proof_number(i, &blocks_dir)?;
        
        submit_or_delete(config, block, path).await?;


        i = i + 1;
        submitted_now = submitted_now + 1;
    }
    Ok(())
}

async fn submit_or_delete(config: &AppCfg, block: VDFProof, path: PathBuf) -> Result<()>{
        let mut sender = Sender::from_app_cfg(config, None).await?;

        sender.commit_proof(
          block.clone()
        ).await?;

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
pub async fn show_backlog(config: &AppCfg) -> Result<(), TxError> {
    // Getting remote miner state
    // there may not be any onchain state.
    match get_remote_tower_height(config).await {
        Ok((remote_height, _proofs_in_epoch)) => {
            println!("Remote tower height: {}", remote_height);
        },
        _ => {
            println!("Remote tower state no initialized");
        },
    }

    // Getting local state height
    let mut blocks_dir = config.workspace.node_home.clone();
    blocks_dir.push(&config.workspace.block_dir);
    let (current_local_proof, _current_block_path) = VDFProof::get_highest_block(&blocks_dir)?;
    // if let Some(current_proof_number) = current_local_proof.height {
    println!("Local tower height: {:?}", current_local_proof.height);
    // } else {
    // println!("Local tower height: 0");
    // }
    Ok(())
}

/// returns remote tower height and current proofs in epoch
pub async fn get_remote_tower_height(app_cfg: &AppCfg) -> Result<(u64, u64), Error> {
    let (client, _) = get_client::find_good_upstream(app_cfg).await?;
    let ts = account_queries::get_tower_state(&client, app_cfg.profile.account.to_owned()).await?;

    Ok((ts.verified_tower_height, ts.count_proofs_in_epoch))
}
