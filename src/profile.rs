use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use snafu::prelude::*;

use crate::Result;

pub const CONFIG_DIR_NAME: &str = "menmos";

fn get_config_path() -> Result<PathBuf> {
    let root_config_dir =
        dirs::config_dir().with_whatever_context(|| "missing config directory")?;

    let cfg_dir_path = root_config_dir.join(CONFIG_DIR_NAME);
    if !cfg_dir_path.exists() {
        fs::create_dir_all(&cfg_dir_path)
            .with_whatever_context(|e| format!("failed to create config directory: {e}"))?;
    }

    Ok(cfg_dir_path.join("client").with_extension("toml"))
}

/// A client profile containing credentials to a menmos cluster.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Profile {
    pub host: String,
    pub username: String,
    pub password: String,
}

/// A client configuration, as stored on disk.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Config {
    /// The configuration profiles set by the user.
    pub profiles: HashMap<String, Profile>,
}

impl Config {
    pub fn load() -> Result<Self> {
        let config_file = get_config_path()?;

        let cfg = if config_file.exists() {
            let buf = fs::read(config_file)
                .with_whatever_context(|e| format!("failed to read config: {e}"))?;
            toml::from_slice(&buf)
                .with_whatever_context(|e| format!("failed to deserialize config: {e}"))?
        } else {
            Config::default()
        };

        Ok(cfg)
    }

    pub fn add<S: Into<String>>(&mut self, name: S, profile: Profile) -> Result<()> {
        self.profiles.insert(name.into(), profile);

        let config_file = get_config_path()?;
        let encoded = toml::to_vec(&self)
            .with_whatever_context(|e| format!("failed to serialize config: {e}"))?;
        let mut f = fs::File::create(config_file)
            .with_whatever_context(|e| format!("failed to create config file: {e}"))?;
        f.write_all(&encoded)
            .with_whatever_context(|e| format!("failed to write config: {e}"))?;
        Ok(())
    }
}
