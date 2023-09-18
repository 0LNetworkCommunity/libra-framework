//! MinerApp delay module
#![forbid(unsafe_code)]
use anyhow::{anyhow, Error};

use vdf::{PietrzakVDFParams, VDFParams, WesolowskiVDFParams, VDF};

/// Runs the VDF
// NOTE from V7 the algorithm switched to Wesolowski
pub fn do_delay(preimage: &[u8], difficulty: u64, security: u64) -> Result<Vec<u8>, Error> {
    let params = WesolowskiVDFParams(security as u16).new();
    let vdf = params.solve(preimage, difficulty);

    vdf.map_err(|e| anyhow!("ERROR: cannot solve VDF, message: {:?}", &e))
}

/// Verifies a proof
pub fn verify(
    preimage: &[u8],
    proof: &[u8],
    difficulty: u64,
    security: u16,
    is_weso: bool,
) -> bool {
    let verifies = if is_weso {
        let vdf = WesolowskiVDFParams(security).new();
        vdf.verify(preimage, difficulty, proof)
    } else {
        let vdf = PietrzakVDFParams(security).new();
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
