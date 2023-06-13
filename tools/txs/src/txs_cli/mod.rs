use anyhow::Result;
use clap::Parser;

use indoc::indoc;

use libra_wallet::legacy::{get_keys_from_prompt, get_keys_from_mnem};

use zapatos_sdk::types::AccountKey;
use zapatos_sdk::types::account_address::AccountAddress;
use zapatos_sdk::crypto::ValidCryptoMaterialStringExt;
use zapatos_sdk::crypto::ed25519::Ed25519PrivateKey;

use zapatos_types::chain_id::ChainId;
use self::submit_transaction::Sender;


mod create_account;
// mod demo;
mod generate_transaction;
mod submit_transaction;
// mod transfer_coin;
mod view;
mod transfer;

#[derive(Parser)]
#[clap(name = env!("CARGO_PKG_NAME"), author, version, about, long_about = None, arg_required_else_help = true)]
pub struct TxsCli {
      #[clap(subcommand)]
      subcommand: Option<Subcommand>,

      /// Optional mnemonic to pass at runtime. Otherwise this will prompt for mnemonic.
      #[clap(short, long)]
      mnemonic: Option<String>,

      /// Private key of the account. Otherwise this will prompt for mnemonic
      #[clap(short, long)]
      private_key: Option<String>,

      /// Maximum number of gas units to be used to send this transaction
      #[clap(short, long)]
      max_gas: Option<u64>,

      /// The amount of coins to pay for 1 gas unit. The higher the price is, the higher priority your transaction will be executed with
      #[clap(short, long)]
      gas_unit_price: Option<u64>,
}

#[derive(clap::Subcommand)]
enum Subcommand {

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

        let pri_key = if self.private_key.is_none() && self.mnemonic.is_none() {
          let legacy = get_keys_from_prompt()?;
          legacy.child_0_owner.pri_key
        } else if self.mnemonic.is_some() {
          let legacy = get_keys_from_mnem(self.mnemonic.as_ref().unwrap().to_owned())?;
          legacy.child_0_owner.pri_key
        } else {
          Ed25519PrivateKey::from_encoded_string(&self.private_key.as_ref().unwrap())?
        };

        let mut send = Sender::new(AccountKey::from_private_key(pri_key), ChainId::test()).await?; // TODO: change this from test.
        

        match &self.subcommand {
            // Some(Subcommand::Test) => transfer::run().await,

            // Some(Subcommand::Demo) => demo::run().await,
            Some(Subcommand::CreateAccount {
                account_address,
                coins,
            }) => create_account::run(account_address, coins.unwrap_or_default()).await,
            Some(Subcommand::Transfer {
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
            Some(Subcommand::View {
                function_id,
                type_args,
                args,
            }) => {
                println!("====================");
                println!(
                    "{}",
                    view::run(function_id, type_args.to_owned(), args.to_owned()).await?
                );
                Ok(())
            }
            _ => Ok(()),
        }
    }
}
