#![allow(dead_code)]
use anyhow::Result;
use clap::{Args, Parser, Subcommand};
use legacy::get_keys_from_prompt;
use std::path::PathBuf;

mod key_gen;
mod legacy;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Entry {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate keys and account address locally
    Keygen {
        /// Recover account from the given mnemonic
        #[clap(short, long)]
        mnemonic: Option<String>,

        /// Path of the directory to store yaml files
        #[clap(short, long)]
        output_dir: Option<String>,
    },
    /// Use the legacy key derivation scheme
    Legacy(LegArgs),
}

#[derive(Args, Debug)]
struct LegArgs {
    ///  display private keys and authentication keys
    #[arg(short, long)]
    display: bool,
    #[arg(short, long)]
    /// save legacy keyscheme private keys to file
    output_path: Option<PathBuf>,
    /// generate new keys and mnemonic in legacy format. It's not clear why you need this besides for testing. Note: these are not useful for creating a validator.
    #[arg(short, long)]
    keygen: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Entry::parse();

    // You can check for the existence of subcommands, and if found use their
    // matches just as you would the top level cmd
    match &cli.command {
        Commands::Legacy(args) => {
            if !args.display && args.output_path.is_none() {
                println!("pass --display to show keys and/or --output-path to save keys");
                return Ok(());
            }

            let l = if args.keygen {
                legacy::legacy_keygen()?
            } else {
                get_keys_from_prompt()?
            };

            if let Some(dir) = &args.output_path {
                l.save_keys(dir)?;
            }

            if args.display {
                l.display();
            }
        }
        Commands::Keygen {
            mnemonic,
            output_dir,
        } => {
            println!(
                "{}",
                key_gen::run(mnemonic.to_owned(), output_dir.as_ref().map(PathBuf::from)).await?
            );
        }
    }
    Ok(())
}
