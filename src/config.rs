use serde::{Deserialize, Serialize};

use std::io::Write;
use std::{fs, path::PathBuf};

use crate::error::Result;

// TODO add some sort of keychain support
#[derive(Serialize, Deserialize)]
pub struct Config {
    pub username: String,
    pub password: String,
    pub data_dir: PathBuf,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            username: "".to_string(),
            password: "".to_string(),
            data_dir: dirs::cache_dir().unwrap().join("attia"),
        }
    }
}

impl Config {
    pub fn load() -> Result<Config> {
        // TODO: Allow this to be customized, e.g. through CLI param
        let path = dirs::config_dir()
            .unwrap()
            .join("attia")
            .join("config.toml");

        if !path.exists() {
            tracing::warn!(path = %path.display(), "no config file found, writing a default one, please fill in the username & password");
            fs::create_dir_all(&path.parent().unwrap())?;
            fs::File::create(&path)?
                .write_all(&toml::to_string_pretty(&Config::default())?.as_bytes())?;
            std::process::exit(1); // FIXME: a big ugly
        }

        Ok(toml::from_str(&fs::read_to_string(path)?)?)
    }
}
