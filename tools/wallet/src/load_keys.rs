//! Key generation
use crate::core::{mnemonic::Mnemonic, wallet_library::WalletLibrary};
use diem_types::chain_id::NamedChain;
use libra_types::exports::AccountAddress;
use libra_types::exports::AuthenticationKey;
use libra_types::core_types::mode_ol::MODE_0L;
use std::{env, process::exit};

/// Get authkey and account from mnemonic
pub fn get_account_from_mnem(
    mnemonic_string: String,
) -> Result<(AuthenticationKey, AccountAddress, WalletLibrary), anyhow::Error> {
    let mut wallet = WalletLibrary::new_from_mnemonic(Mnemonic::from(mnemonic_string.trim())?);
    let (auth_key, _) = wallet.new_address()?;
    let account = auth_key.derived_address();
    Ok((auth_key, account, wallet))
}

/// helper to return account tuple from wallet
pub fn get_account_from_wallet(
    wallet: &WalletLibrary,
) -> Result<(AuthenticationKey, AccountAddress, WalletLibrary), anyhow::Error> {
    get_account_from_mnem(wallet.mnemonic())
}

/// Prompts user to type mnemonic securely.
pub fn get_account_from_prompt() -> (AuthenticationKey, AccountAddress, WalletLibrary) {
    println!("Enter your 0L mnemonic:");

    let test_env_mnem = env::var("MNEM");
    // if we are in debugging or CI mode
    let mnem = match (*MODE_0L == NamedChain::TESTING) && test_env_mnem.is_ok() {
        true => {
            println!("Debugging mode, using mnemonic from env variable, $MNEM");
            test_env_mnem.unwrap().trim().to_string()
        }
        false => match rpassword::read_password_from_tty(Some("\u{1F511} ")) {
            Ok(read) => read.trim().to_owned(),
            Err(e) => {
                println!(
                    "ERROR: could not read mnemonic from prompt, message: {}",
                    &e.to_string()
                );
                exit(1);
            }
        },
    };

    match get_account_from_mnem(mnem) {
        Ok(a) => a,
        Err(e) => {
            println!(
                "ERROR: could not get account from mnemonic, message: {}",
                &e.to_string()
            );
            exit(1);
        }
    }
}

#[test]
fn wallet() {
    // use diem_wallet::Mnemonic;
    let mut wallet = WalletLibrary::new();

    let (auth_key, child_number) = wallet.new_address().expect("Could not generate address");
    let mnemonic_string = wallet.mnemonic(); //wallet

    println!("auth_key:\n{:?}", auth_key.to_string());
    println!("child_number:\n{:?}", child_number);
    println!("mnemonic:\n{}", mnemonic_string);

    let mut wallet = WalletLibrary::new_from_mnemonic(Mnemonic::from(&mnemonic_string).unwrap());

    let (main_addr, child_number) = wallet.new_address().unwrap();
    println!("wallet\n:{:?} === {:x}", child_number, main_addr);

    let vec_addresses = wallet.get_addresses().unwrap();

    println!("vec_addresses\n:{:?}", vec_addresses);

    // Expect this to be zero before we haven't populated the address map in the repo
    assert_eq!(vec_addresses.len(), 1);
}

#[test]
fn fixture_wallet() {
    use crate::account_keys::get_ol_legacy_address;

    // alice
    let mnemonic_string = "talent sunset lizard pill fame nuclear spy noodle basket okay critic grow sleep legend hurry pitch blanket clerk impose rough degree sock insane purse";

    let mut wallet = WalletLibrary::new_from_mnemonic(Mnemonic::from(mnemonic_string).unwrap());

    let (main_addr, child_number) = wallet.new_address().unwrap();
    println!("wallet\n:{:?} === {:x}", child_number, main_addr);

    let (_, acc, _) = get_account_from_mnem(mnemonic_string.to_owned()).unwrap();

    // expect the same address for alice
    assert_eq!(
        &acc.to_string(),
        "87515d94a244235a1433d7117bc0cb154c613c2f4b1e67ca8d98a542ee3f59f5"
    );
    assert_eq!(
        get_ol_legacy_address(acc)
            .unwrap()
            .to_string()
            .to_uppercase(),
        "000000000000000000000000000000004C613C2F4B1E67CA8D98A542EE3F59F5"
    );
}
