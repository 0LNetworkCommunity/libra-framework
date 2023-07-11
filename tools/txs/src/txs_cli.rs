use std::path::PathBuf;

use crate::submit_transaction::Sender;

use anyhow::Result;
use clap::Parser;
use indoc::indoc;
use libra_types::legacy_types::app_cfg::AppCfg;
use libra_wallet::account_keys::{get_keys_from_prompt, get_keys_from_mnem};
use url::Url;

use zapatos_sdk::rest_client::Client;
use zapatos_sdk::types::{AccountKey, account_address::AccountAddress};
use zapatos_sdk::crypto::{ValidCryptoMaterialStringExt, ed25519::Ed25519PrivateKey};
use zapatos_types::chain_id::{NamedChain, ChainId};

#[derive(Parser)]
#[clap(name = env!("CARGO_PKG_NAME"), author, version, about, long_about = None, arg_required_else_help = true)]
/// Submit a transaction to the blockchain.
pub struct TxsCli {
      #[clap(subcommand)]
      pub subcommand: Option<TxsSub>,

      /// optional, path to the config file
      #[clap(short, long)]
      pub config_path: Option<PathBuf>,

      /// optional, mnemonic to pass at runtime. Otherwise this will prompt for mnemonic.
      #[clap(short, long)]
      pub mnemonic: Option<String>,

      /// optional, private key of the account. Otherwise this will prompt for mnemonic. Warning: intended for testing.
      #[clap(short, long)]
      pub test_private_key: Option<String>,

      /// optional, id of chain as name. Will default to MAINNET;
      #[clap(short, long)]
      pub chain_id: Option<NamedChain>,

      /// optional, URL of the upstream node to send tx to, including port
      /// Otherwise will default to what is in the config file
      #[clap(short, long)]
      pub url: Option<Url>,

      /// optional, maximum number of gas units to be used to send this transaction
      #[clap(short, long)]
      pub max_gas: Option<u64>,

      /// optional, the amount of coins to pay for 1 gas unit. The higher the price is, the higher priority your transaction will be executed with
      #[clap(short, long)]
      pub gas_unit_price: Option<u64>,
}

#[derive(clap::Subcommand)]
pub enum TxsSub {

    /// Create onchain account by using Aptos faucet
    CreateAccount {
        /// Create onchain account with the given address
        #[clap(short, long)]
        account_address: String,

        /// The amount of coins to fund the new account
        #[clap(short, long)]
        coins: Option<u64>,
    },

    /// Transfer coins between accounts
    Transfer {
        /// Address of the recipient
        #[clap(short, long)]
        to_account: AccountAddress,

        /// The amount of coins to transfer
        #[clap(short, long)]
        amount: u64,
    },

    /// Generate a transaction that executes an Entry function on-chain
    GenerateTransaction {
        #[clap(
            short,
            long,
            help = indoc!{r#"
                Function identifier has the form <ADDRESS>::<MODULE_ID>::<FUNCTION_NAME>

                Example:
                0x1::coin::transfer
            "#}
        )]
        function_id: String,

        #[clap(
            short,
            long,
            help = indoc!{ r#"
                Type arguments separated by commas

                Example:
                'u8, u16, u32, u64, u128, u256, bool, address, vector<u8>, signer'
                '0x1::aptos_coin::AptosCoin'
            "#}
        )]
        type_args: Option<String>,

        #[clap(
            short,
            long,
            help = indoc!{ r#"
                Function arguments separated by commas

                Example:
                '0x1, true, 12, 24_u8, x"123456"'
            "#}
        )]
        args: Option<String>,

        /// Maximum amount of gas units to be used to send this transaction
        #[clap(short, long)]
        max_gas: Option<u64>,

        /// The amount of coins to pay for 1 gas unit. The higher the price is, the higher priority your transaction will be executed with
        #[clap(short, long)]
        gas_unit_price: Option<u64>,

        // / Private key to sign the transaction
        // #[clap(short, long)]
        // private_key: String,

        /// Submit the generated transaction to the blockchain
        #[clap(short, long)]
        submit: bool,
    },

    /// Execute a View function on-chain
    View {
        #[clap(
            short,
            long,
            help = indoc!{r#"
                Function identifier has the form <ADDRESS>::<MODULE_ID>::<FUNCTION_NAME>

                Example:
                0x1::coin::balance
            "#}
        )]
        function_id: String,

        #[clap(
            short,
            long,
            help = indoc!{ r#"
                Type arguments separated by commas

                Example:
                'u8, u16, u32, u64, u128, u256, bool, address, vector<u8>, signer'
                '0x1::aptos_coin::AptosCoin'
            "#}
        )]
        type_args: Option<String>,

        #[clap(
            short,
            long,
            help = indoc!{ r#"
                Function arguments separated by commas

                Example:
                '0x1, true, 12, 24_u8, x"123456"'
            "#}
        )]
        args: Option<String>,
    },
}

impl TxsCli {
    pub async fn run(&self) -> Result<()> {
        let pri_key = if self.test_private_key.is_none() && self.mnemonic.is_none() {
          let legacy = get_keys_from_prompt()?;
          legacy.child_0_owner.pri_key
        } else if self.mnemonic.is_some() {
          let legacy = get_keys_from_mnem(self.mnemonic.as_ref().unwrap().to_owned())?;
          legacy.child_0_owner.pri_key
        } else {
          Ed25519PrivateKey::from_encoded_string(&self.test_private_key.as_ref().unwrap())?
        };

        let chain_name = self.chain_id.unwrap_or(NamedChain::MAINNET);

        let url = if let Some(u) = self.url.as_ref() {
            u.to_owned()
        }else {
          AppCfg::load(self.config_path.clone())?.pick_url(Some(chain_name))?
        };

        let client = Client::new(url);

        // TODO: load profile from ConfigCli::load_ext
        let mut send = Sender::new(
          AccountKey::from_private_key(pri_key),
          ChainId::new(chain_name.id()),
          Some(client),
        ).await?;

        match &self.subcommand {
            Some(TxsSub::CreateAccount {
                account_address,
                coins,
            }) => crate::create_account::run(account_address, coins.unwrap_or_default()).await,
            Some(TxsSub::Transfer {
                to_account,
                amount,
            }) => {
                send.transfer(
                    to_account.to_owned(),
                    amount.to_owned(),
                )
                .await
            }
            // Some(Subcommand::GenerateTransaction {
            //     function_id,
            //     type_args,
            //     args,
            //     _max_gas,
            //     _gas_unit_price,
            //     // private_key,
            //     submit,
            // }) => {
            //     println!("====================");
            //     let signed_trans = send::generate_transaction(
            //         function_id,
            //         // &pri_key,
            //         type_args.to_owned(),
            //         args.to_owned(),
            //         // max_gas.to_owned(),
            //         // gas_unit_price.to_owned(),
            //     )
            //     .await?;

            //     println!("{}", format_signed_transaction(&signed_trans));

            //     if *submit {
            //         println!("{}", "Submitting transaction...".green().bold());
            //         send.submit(&signed_trans).await?;
            //         println!("Success!");
            //     }
            //     Ok(())
            // }
            Some(TxsSub::View {
                function_id,
                type_args,
                args,
            }) => {
                println!("====================");
                println!(
                    "{}",
                    crate::view::run(function_id, type_args.to_owned(), args.to_owned()).await?
                );
                Ok(())
            }
            _ => Ok(()),
        }
    }
}
