use color_eyre::{eyre::Context, Result};
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub port: u32,
    pub bind_addr: String,
    pub db_url: String,
    pub registration_key: String
}

impl Config {
    /// Parses a new configuration from the default file ``config.ron``
    pub fn new() -> Result<Self> {
        ron::from_str(
            &std::fs::read_to_string("./config.ron").context("Failed to open configuration file.")?,
        ).context("Configuration is malformed.")
    }
}