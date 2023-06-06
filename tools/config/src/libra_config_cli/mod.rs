use anyhow::Result;
use clap::Parser;

mod init;

#[derive(Parser)]
#[clap(name = env!("CARGO_PKG_NAME"), author, version, about, long_about = None, arg_required_else_help = true)]
pub struct LibraConfigCli {
    #[clap(subcommand)]
    subcommand: Option<Subcommand>,
}

#[derive(clap::Subcommand)]
enum Subcommand {
    /// Generate config.yaml file that stores 0L configuration
    Init {
        /// Ed25519 public key
        #[clap(long)]
        public_key: String,

        /// Profile to use from the CLI config
        ///
        /// This will be used to override associated settings such as
        /// the REST URL, the Faucet URL, and the private key arguments.
        ///
        /// Defaults to "default"
        #[clap(long)]
        profile: Option<String>,

        /// Initializes configs in this directory
        /// You will only use this if you are developing apps
        /// and each app will have different public key configurations.
        #[clap(long)]
        init_workspace: bool,
    },
}

impl LibraConfigCli {
    pub async fn run(&self) -> Result<()> {
        match &self.subcommand {
            Some(Subcommand::Init {
                public_key,
                profile,
                init_workspace,
            }) => init::run(
              public_key, 
              profile.as_deref().to_owned(),
              *init_workspace,
            ).await,
            _ => Ok(()),
        }
    }
}
