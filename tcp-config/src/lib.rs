use std::{fs::File, ops::Deref};

use once_cell::sync::OnceCell;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct IpAddrV4 {
    pub ip: String,
    pub port: u16,
    pub tag: String,
}

#[derive(Debug, Deserialize)]
pub struct Config {
    nodes: Vec<IpAddrV4>,
}

impl Deref for Config {
    type Target = Vec<IpAddrV4>;

    fn deref(&self) -> &Self::Target {
        &self.nodes
    }
}

static INSTANCE: OnceCell<Config> = OnceCell::new();

impl Config {
    fn read_in_config() -> Result<Config, serde_json::Error> {
        let config_file = File::open("./config.json").unwrap();
        serde_json::from_reader(config_file)
    }

    pub fn get() -> &'static Config {
        &INSTANCE.get_or_try_init(Self::read_in_config).unwrap()
    }
}
