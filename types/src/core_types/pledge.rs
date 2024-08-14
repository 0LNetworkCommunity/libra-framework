//! User code of conduct pledges

use diem::common::utils::prompt_yes;
use diem_crypto::HashValue;
use diem_types::chain_id::NamedChain;
use serde::{self, Deserialize, Serialize};

#[cfg(test)]
use crate::core_types::app_cfg::{AppCfg, Profile};
use crate::core_types::mode_ol::MODE_0L;

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Pledge {
    /// the canonical id of this pledge
    pub id: u8,
    /// nickname of pledge
    pub name: String,
    /// textual versions of the pledge
    pub version: u8,
    /// hash of the textual version
    #[serde(with = "hex::serde")]
    pub hash: Vec<u8>,
    /// Text question
    pub question: String,
    /// Text preamble
    pub preamble: String,
    /// if this pledge been committed to chain
    pub on_chain: bool,
}

pub enum CanonicalPledges {
    /// protect the game
    Game = 0,
    /// operate in good faith
    Validator = 1,
}

impl Pledge {
    /// make the unique hex encoding of the text.
    pub fn to_hash(&mut self) {
        let mut concat = self.question.clone();
        concat.push_str(&self.preamble);
        self.hash = HashValue::sha3_256_of(&concat.into_bytes()).to_vec();
    }

    /// check pledge hash
    pub fn check_pledge_hash(pledge_idx: u8, bytes: &[u8]) -> bool {
        if pledge_idx == 0 {
            return bytes == Self::pledge_protect_the_game().hash;
        } else if pledge_idx == 1 {
            return bytes == Self::pledge_validator().hash;
        } else {
            assert!(pledge_idx < 2, "pledge index not found");
        }
        false
    }

    /// interact with user to get basic pledges, validator pledge optional on default setup
    pub fn pledge_dialogue(&self) -> bool {
        println!("PLEDGE #{}: {}\n\n{}", &self.id, &self.name, &self.preamble);

        if *MODE_0L != NamedChain::MAINNET {
            println!("seems you are using CI or testnet settings, pledges default to yes");
            return true;
        };
        if prompt_yes(&format!("\n{}", &self.question)) {
            return true;
        }
        false
    }

    /// #0 Protect the Game Pledge
    /// Reference: Docs from Discord 0L Contributors circa June 2024
    pub fn pledge_protect_the_game() -> Pledge {
        let mut p = Pledge {
            id: 0,
            name: "Protect the Game".to_string(),
            version: 0,
            question: "Do you pledge to not damage the game and never cheat other users?".to_string(),
            preamble: "Code is not law at Open Libra. The law is law. The law comes from history.\n\nI understand written and unwritten laws come from social norms. I will refer to the expectations of this community based on canonical instructions, code documentation, and common sense to know when I'm cheating at the game, or otherwise unlawfully disadvantaging someone for my benefit.\n\nCheating can include, but is not limited to: gaining an advantage in a way that would be impossible unless it was covert, dishonest, untrue, or otherwise using an expected common courtesy others have extended to me which I'm not willing to return.".to_string(),
            hash: vec![],
            on_chain: false,
        };

        p.to_hash();

        p
    }

    /// #1 Validator pledge
    /// Reference: Docs from Discord 0L Contributors circa June 2024
    pub fn pledge_validator() -> Pledge {
        let mut p = Pledge {
            id: 1,
            name: "Operate in Good Faith".to_string(),
            version: 0,
            question: "Do you pledge to be a validator that acts in good faith to secure the network?".to_string(),
            preamble: "When taking this pledge you are also taking the Protect the Game pledge:\n'I pledge to not damage the game and never cheat other users'.\n\nAdditionally you pledge to: obey the blockchain's policies as intended, some of which may be encoded as smart contracts, not pretend to be multiple people (sybil), not change the blockchain settings or software without consulting the community, run the blockchain security software (e.g validator, and fullnode software) as intended and in its entirety.".to_string(),
            hash: vec![],
            on_chain: false,
        };

        p.to_hash();

        p
    }
}

#[tokio::test]
async fn test_pledge() {
    let mut a = AppCfg {
        user_profiles: vec![Profile::default()],
        ..Default::default()
    };
    let p = a.get_profile_mut(None).unwrap();
    assert!(p.pledges.is_none());
    let zero = Pledge::pledge_protect_the_game();
    p.pledges = Some(vec![zero]);
    assert!(p.pledges.is_some());
}
