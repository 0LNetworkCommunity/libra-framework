use anyhow::anyhow;
use clap::Parser;
use std::path::PathBuf;
use tokio::process::Command;
#[derive(Parser)]
/// Start a libra node
pub struct NodeCli {
    #[clap(short, long)]
    /// filepath to the validator or fullnode yaml config file.
    config_path: Option<PathBuf>,
}

impl NodeCli {
    pub async fn run(&self) -> anyhow::Result<()> {
        //Get the current executable path
        let current_exe = std::env::current_exe().unwrap();
        //Get the path to the libra-node executable
        let node_path = current_exe.parent().unwrap().join("./libra-node");
        // Convert the config path to a string for the command line, providing a default if none is set
        let config_path_arg = self
            .config_path
            .as_ref()
            .map_or_else(|| "".to_string(), |p| p.to_str().unwrap_or("").to_string());
        // Construct the command to run in the tmux session
        let command = format!("{} --config-path {}", node_path.display(), config_path_arg);
        // The session is named 'libra_node'
        Command::new("tmux")
            .args(["new-session", "-d", "-s", "libra-node"])
            .output()
            .await
            .map_err(|e| anyhow!("Failed to create new tmux session: {}", e))?;

        // Send the command to the created tmux session
        Command::new("tmux")
            .args(["send-keys", "-t", "libra-node", &command, "C-m"])
            .output()
            .await
            .map_err(|e| anyhow!("Failed to send keys to tmux session: {}", e))?;
        // Attach to the tmux session
        Command::new("tmux")
            .args(["attach-session", "-t", "libra-node"])
            .output()
            .await
            .map_err(|e| anyhow!("Failed to attach to tmux session: {}", e))?;
        Ok(())
    }
}
