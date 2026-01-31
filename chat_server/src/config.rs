use std::{env, fs::File, path::PathBuf};

use serde::{Serialize, Deserialize};
use anyhow::{Result, bail};
#[derive(Serialize, Deserialize, Debug)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub auth: AuthConfig,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct ServerConfig {
    pub port: u16,
    pub db_url: String,
    pub base_dir: PathBuf,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct AuthConfig {
    pub sk: String,
    pub pk: String, 
}

impl AppConfig {
    pub fn load() -> Result<Self> {
        let ret = match (
            File::open("app.yml"),
            File::open("/etc/config/app.yml"),
            env::var("CHAT_CONFIG"),
        ) {
            (Ok(file), _, _) => serde_yaml::from_reader(file),
            (_,Ok(file),_) => serde_yaml::from_reader(file),
            (_,_,Ok(path)) => serde_yaml::from_reader(File::open(path)?),
            _ => bail!("Cannot find configuration file"),
        };
        Ok(ret?)
    }
}
