use diem_genesis::config::HostAndPort;
use libra_types::core_types::fixtures::TestPersona;
use std::path::PathBuf;

#[derive(clap::Parser)]
pub struct TestnetConfigOpts {
    /// sensible defaults for testnet, does not need a genesis repo
    /// accounts are created from fixture mnemonics for alice, bob, carol, dave
    /// which persona is this machine going to register as
    #[clap(short, long)]
    me: TestPersona,
    /// ordered list of dns/ip with port for alice..dave
    /// use 6180 for production validator port
    #[clap(long)]
    host_list: Vec<HostAndPort>,
    /// path to the Move framework file, usually ./framework/releases/head.mrb
    #[clap(short, long)]
    framework_mrb_path: PathBuf,
    /// path to file for legacy migration file
    #[clap(short, long)]
    json_legacy: Option<PathBuf>,
}
