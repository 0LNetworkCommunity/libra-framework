//! garbage collection

use crate::core::next_proof;
use anyhow::bail;
use libra_types::{
    exports::Client,
    legacy_types::{app_cfg::AppCfg, block::VDFProof},
};

use std::{fs, path::PathBuf, time::SystemTime};
use zapatos_sdk::crypto::HashValue;
/// Start the GC for a proof that is known bad
pub fn gc_failed_proof(app_cfg: &AppCfg, bad_proof_path: PathBuf) -> anyhow::Result<()> {
    println!(
        "bad proof found at {}. Will collect subsequent proofs and move to vdf proof archive",
        bad_proof_path.to_str().unwrap()
    );

    // gets the profile from workspace default
    let block_dir = app_cfg.get_block_dir(None)?;
    if let Some(v) = collect_subsequent_proofs(bad_proof_path, block_dir.clone())? {
        put_in_trash(v, &block_dir)?;
    }
    Ok(())
}

/// collect all the proofs after a given height, inclusive of the given height
pub fn collect_subsequent_proofs(
    bad_proof_path: PathBuf,
    block_dir: PathBuf,
) -> anyhow::Result<Option<Vec<PathBuf>>> {
    let bad_proof = VDFProof::parse_block_file(&bad_proof_path, true)?;

    let highest_local = VDFProof::get_highest_block(&block_dir)?.0.height;

    // something is wrong with file list
    if highest_local < bad_proof.height {
        bail!("highest local proof is lower than bad proof, looks like a filename and height don't match for: {}", &bad_proof_path.to_str().unwrap())
    };
    // check if the next proof nonce that the chain expects has already been mined.
    let mut vec_trash: Vec<PathBuf> = vec![];
    let mut i = bad_proof.height;
    while i < highest_local {
        let (_, file) = VDFProof::get_proof_number(i, &block_dir)?;
        vec_trash.push(file);
        i += 1;
    }
    Ok(Some(vec_trash))
}

/// take list of proofs and save in garbage file
pub fn put_in_trash(to_trash: Vec<PathBuf>, blocks_dir: &PathBuf) -> anyhow::Result<()> {
    let now = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH)?;
    let new_dir = blocks_dir.join(now.as_secs().to_string());
    fs::create_dir_all(&new_dir)?;

    println!(
        "placing {} files in trash at {}",
        to_trash.len(),
        new_dir.to_str().unwrap()
    );

    to_trash.into_iter().for_each(|f| {
        fs::copy(&f, &new_dir).unwrap();
        fs::remove_file(&f).unwrap();
    });

    Ok(())
}

/// check remaining proofs in backlog.
/// if they all fail, move the list to a trash file
pub async fn find_first_discontinous_proof(
    cfg: AppCfg,
    client: &Client,
) -> anyhow::Result<Option<PathBuf>> {
    let block_dir = cfg.get_block_dir(None)?;
    let highest_local = VDFProof::get_highest_block(&block_dir)?.0.height;
    // start from last known proof on chain.
    let p = next_proof::get_next_proof_from_chain(&mut cfg.clone(), client).await?;

    if highest_local < p.next_height {
        return Ok(None);
    };
    // check if the next proof nonce that the chain expects has already been mined.

    let mut i = p.next_height;
    let mut preimage = p.preimage;
    while i < highest_local {
        let (proof, file) = VDFProof::get_proof_number(i, &block_dir)?;
        let next_preimage = HashValue::sha3_256_of(&proof.proof).to_vec();
        if preimage != next_preimage {
            return Ok(Some(file));
        }
        preimage = next_preimage;

        i += 1;
    }

    Ok(None)
}
