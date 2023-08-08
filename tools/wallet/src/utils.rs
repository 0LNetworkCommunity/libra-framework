//! file and directory utilities
// TODO: this file is copied from vendor/crates/aptos/src/common/utils.rs
// It's not being imported because of build issues when we try to import that module. So it's a copy paste hack for now. But should be reviewed.

use anyhow::{anyhow, bail};
use dialoguer::Confirm;
use serde::{de::DeserializeOwned, Serialize};
use std::{
    env::current_dir,
    fs::OpenOptions,
    io::Write,
    // os::unix::fs::OpenOptionsExt,
    path::{Path, PathBuf},
};
#[cfg(unix)]
use std::os::unix::fs::OpenOptionsExt;
use zapatos_genesis::keys::PublicIdentity;

/// A common result to be returned to users
pub type CliResult = Result<String, String>;

/// A common result to remove need for typing `Result<T, CliError>`
pub type CliTypedResult<T> = Result<T, anyhow::Error>;

/// Checks if a file exists, being overridden by `PromptOptions`
pub fn check_if_file_exists(file: &Path) -> CliTypedResult<()> {
    if file.exists() {
        let o: Option<&str> = option_env!("LIBRA_CI");
        if o.is_some() {
            // TODO: how to make tests always overwrite?
            println!("LIBRA_CI is set, overwriting {:?}", file.as_os_str());
            return Ok(());
        }

        prompt_yes_with_override(&format!(
            "{:?} already exists, are you sure you want to overwrite it?",
            file.as_os_str(),
        ))?
    }

    Ok(())
}

/// Write a User only read / write file
pub fn write_to_user_only_file(path: &Path, name: &str, bytes: &[u8]) -> anyhow::Result<()> {
    let mut opts = OpenOptions::new();
    #[cfg(unix)]
    opts.mode(0o600);
    write_to_file_with_opts(path, name, bytes, &mut opts)
}

/// Write a `&[u8]` to a file with the given options
pub fn write_to_file_with_opts(
    path: &Path,
    name: &str,
    bytes: &[u8],
    opts: &mut OpenOptions,
) -> anyhow::Result<()> {
    let mut file = opts
        .write(true)
        .create(true)
        .truncate(true)
        .open(path)
        .map_err(|e| anyhow!("cannot write file: {}, message: {}", name, e))?;

    file.write_all(bytes)
        .map_err(|e| anyhow!("cannot write file: {}, message: {}", name, e))
}

pub fn dir_default_to_current(maybe_dir: &Option<PathBuf>) -> CliTypedResult<PathBuf> {
    if let Some(dir) = maybe_dir {
        Ok(dir.to_owned())
    } else {
        current_dir().map_err(|e| anyhow!(e))
    }
}

pub fn create_dir_if_not_exist(dir: &Path) -> CliTypedResult<()> {
    // Check if the directory exists, if it's not a dir, it will also fail here
    if !dir.exists() || !dir.is_dir() {
        std::fs::create_dir_all(dir).map_err(|e| anyhow!(e))?;
        println!("Created {} folder", dir.display());
    } else {
        println!("{} folder already exists", dir.display());
    }
    Ok(())
}

/// Note: We removed PromptOptions because it was not used
pub fn prompt_yes_with_override(prompt: &str) -> CliTypedResult<()> {
    if prompt_yes(prompt) {
        Ok(())
    } else {
        bail!("Aborted by user")
    }
}

// dialoguer is cleaner
pub fn prompt_yes(prompt: &str) -> bool {
    let t = format!("{} [yes/no] >", prompt);
    Confirm::new().with_prompt(t).interact().unwrap()
}

pub fn to_yaml<T: Serialize + ?Sized>(input: &T) -> CliTypedResult<String> {
    Ok(serde_yaml::to_string(input)?)
}

pub fn from_yaml<T: DeserializeOwned>(input: &str) -> CliTypedResult<T> {
    Ok(serde_yaml::from_str(input)?)
}

pub fn read_from_file(path: &Path) -> CliTypedResult<Vec<u8>> {
    std::fs::read(path).map_err(|e| anyhow!(e.to_string()))
}

pub fn read_public_identity_file(public_identity_file: &Path) -> CliTypedResult<PublicIdentity> {
    let bytes = read_from_file(public_identity_file)?;
    from_yaml(&String::from_utf8(bytes).map_err(|e| anyhow!(e))?)
}
