//! Helper module to get fixtures data from files in ol/fixtures folder.
//! This module includes functionalities to manage test personas and
//! retrieve mnemonic files associated with them.

use core::fmt;
use std::{fs, path::PathBuf, str::FromStr};

use anyhow::bail;

#[derive(Clone, Debug)]
pub enum TestPersona {
    Alice,
    Bob,
    Carol,
    Dave,
}


impl FromStr for TestPersona {
    type Err = &'static str;

    /// Convert a string to a TestPersona variant.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "alice" => Ok(TestPersona::Alice),
            "bob" => Ok(TestPersona::Bob),
            "carol" => Ok(TestPersona::Carol),
            "dave" => Ok(TestPersona::Dave),
            _ => Err("not found"),
        }
    }
}

impl fmt::Display for TestPersona {
    /// Format the TestPersona as a string.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match self {
            TestPersona::Alice => "alice",
            TestPersona::Bob => "bob",
            TestPersona::Carol => "carol",
            TestPersona::Dave => "dave",
        };
        write!(f, "{}", s)
    }
}

impl TestPersona {
    /// get persona from index. Used for testnet to assign persona to validator seat
    pub fn from(idx: usize) -> anyhow::Result<Self> {
        match idx {
            0 => Ok(TestPersona::Alice),
            1 => Ok(TestPersona::Bob),
            2 => Ok(TestPersona::Carol),
            3 => Ok(TestPersona::Dave),
            _ => bail!("no default persona at this index"),
        }
    }

    /// Get the index of the persona. Used for testnet to assign persona to validator seat
    pub fn idx(&self) -> usize {
        match self {
            TestPersona::Alice => 0,
            TestPersona::Bob => 1,
            TestPersona::Carol => 2,
            TestPersona::Dave => 3,
        }
    }
    pub fn get_persona_mnem(&self)-> String {
      let s = match self {
        TestPersona::Alice => "talent sunset lizard pill fame nuclear spy noodle basket okay critic grow sleep legend hurry pitch blanket clerk impose rough degree sock insane purse",
        TestPersona::Bob => "ring pumpkin cake build jungle cloth bronze aerobic mechanic baby love melt below sight cotton trophy inquiry sugar exhibit sure first match ten clarify",
        TestPersona::Carol => "giraffe tower toward rapid flower obey piano circle better announce castle when enlist inquiry arrive segment leave develop confirm avoid meat loud fit parent",
        TestPersona::Dave => "recall october regret kite undo choice outside season business wall quit arrest vacant arrow giggle vote ghost winter hawk soft cheap decide exhaust spare"
      };
      s.to_string()
    }
}

#[test]
fn test_block() {
    let mnem = TestPersona::Alice.get_persona_mnem();
    assert!(mnem.contains("talent"));
}
