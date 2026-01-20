use std::{env, fs::File};

use serde::{Serialize, Deserialize};
use anyhow::{Result, bail};
#[derive(Serialize, Deserialize, Debug)]
pub struct AppConfig {
    pub server: ServerConfig,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct ServerConfig {
    pub port: u16,
}

impl AppConfig {
    pub fn load() -> Result<Self> {
        let ret = match (
            File::open("app.yml"),
            File::open("/etc/config/app.yaml"),
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
