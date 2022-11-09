use serde_derive::Deserialize;
use std::fs::{read_to_string, File};
use std::io::Write;
use toml::from_str;

#[derive(Deserialize, Debug, Clone)]
pub struct Config {
    pub username: String,
    pub password: String,
    pub smtp: Option<String>,
    pub subject: String,
}

impl Config {
    fn default() {
        let mut config_file = match File::create("./config.toml") {
            Ok(v) => v,
            Err(e) => {
                eprintln!("failed to create new config file due to: {e}! exiting");
                std::process::exit(1);
            }
        };
        writeln!(
            &mut config_file,
            "username = ''
password = ''
#stmp = 'uncomment and replace with custom mail server, otherwise gmail is assumed'"
        )
        .expect("failed to write default config");
    }
    pub fn load() -> Config {
        let config_contents = match read_to_string("./config.toml") {
            Ok(v) => v,
            Err(e) => {
                eprintln!("failed to read config file due to {e}");
                Config::default();
                eprintln!("fill out ./config.toml");
                std::process::exit(0);
            }
        };
        match from_str(&config_contents) {
            Ok(v) => v,
            Err(e) => {
                eprintln!("invalid config!\n{e}");
                std::process::exit(1);
            }
        }
    }
}
