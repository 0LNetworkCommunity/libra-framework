//! MinerApp delay module
#![forbid(unsafe_code)]
use anyhow::{anyhow, Error};
use libra_types::{exports::NamedChain, legacy_types::mode_ol::MODE_0L};
use vdf::{PietrzakVDFParams, VDFParams, WesolowskiVDFParams, VDF};

/// Runs the VDF
pub fn do_delay(preimage: &[u8], difficulty: u64, security: u64) -> Result<Vec<u8>, Error> {
    // Functions for running the VDF.

    // TODO(Wiri): we need new fixtures so that we're not switching algorithms.
    // let vdf = if
    //   MODE_0L.clone() == NamedChain::TESTNET ||
    //   MODE_0L.clone() == NamedChain::TESTING {
    //   let vdf = PietrzakVDFParams(security as u16).new();
    //   vdf.solve(preimage, difficulty)
    // } else {
    //   let vdf = WesolowskiVDFParams(security as u16).new();
    //   vdf.solve(preimage, difficulty)
    // };
    let params = WesolowskiVDFParams(security as u16).new();
    let vdf = params.solve(preimage, difficulty);

    vdf.map_err(|e| anyhow!("ERROR: cannot solve VDF, message: {:?}", &e))
}

/// Verifies a proof
pub fn verify(preimage: &[u8], proof: &[u8], difficulty: u64, security: u16) -> bool {
    // TODO(Wiri): we need new fixtures so that we're not switching algorithms.
    let verifies =
        if MODE_0L.clone() == NamedChain::TESTNET || MODE_0L.clone() == NamedChain::TESTING {
            let vdf = PietrzakVDFParams(security as u16).new();
            vdf.verify(preimage, difficulty, proof)
        } else {
            let vdf = WesolowskiVDFParams(security as u16).new();
            vdf.verify(preimage, difficulty, proof)
        };

    match verifies {
        Ok(_) => true,
        Err(e) => {
            eprintln!("Proof is not valid. {:?}", &e);
            false
        }
    }
}
