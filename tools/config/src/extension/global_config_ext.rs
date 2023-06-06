use std::path::PathBuf;
use zapatos::{
    common::{
        types::{CliError, CliTypedResult, ConfigSearchMode},
        utils::{current_dir, read_from_file},
    },
    config::{ConfigType, GlobalConfig},
    genesis::git::from_yaml,
};
use libra_types::LIBRA_CONFIG_DIRECTORY;
// const LIBRA_CONFIG_DIRECTORY: &str = ".0L";
const GLOBAL_CONFIG_FILE: &str = "global_config.yaml";

pub trait GlobalConfigExt {
    fn load_ext() -> CliTypedResult<GlobalConfig>;
    fn get_config_location_ext(&self, mode: ConfigSearchMode) -> CliTypedResult<PathBuf>;
}

impl GlobalConfigExt for GlobalConfig {
    fn load_ext() -> CliTypedResult<GlobalConfig> {
        let path = global_folder()?.join(GLOBAL_CONFIG_FILE);
        if path.exists() {
            from_yaml(&String::from_utf8(read_from_file(path.as_path())?)?)
        } else {
            // If we don't have a config, let's load the default
            Ok(GlobalConfig::default())
        }
    }

    fn get_config_location_ext(&self, mode: ConfigSearchMode) -> CliTypedResult<PathBuf> {
        match self.config_type.unwrap_or_default() {
            ConfigType::Global => global_folder(),
            ConfigType::Workspace => find_workspace_config(current_dir()?, mode),
        }
    }
}

pub fn global_folder() -> CliTypedResult<PathBuf> {
    if let Some(dir) = dirs::home_dir() {
        Ok(dir.join(LIBRA_CONFIG_DIRECTORY))
    } else {
        Err(CliError::UnexpectedError(
            "Unable to retrieve home directory".to_string(),
        ))
    }
}

pub fn find_workspace_config(
    starting_path: PathBuf,
    mode: ConfigSearchMode,
) -> CliTypedResult<PathBuf> {
    match mode {
        ConfigSearchMode::CurrentDir => Ok(starting_path.join(LIBRA_CONFIG_DIRECTORY)),
        ConfigSearchMode::CurrentDirAndParents => {
            let mut current_path = starting_path.clone();
            loop {
                current_path.push(LIBRA_CONFIG_DIRECTORY);
                if current_path.is_dir() {
                    break Ok(current_path);
                } else if !(current_path.pop() && current_path.pop()) {
                    // If we aren't able to find the folder, we'll create a new one right here
                    break Ok(starting_path.join(LIBRA_CONFIG_DIRECTORY));
                }
            }
        }
    }
}
