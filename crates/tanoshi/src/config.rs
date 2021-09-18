use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::{iter, path::PathBuf};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct TelegramConfig {
    pub name: String,
    pub token: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct PushoverConfig {
    pub application_key: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Config {
    #[serde(skip)]
    path: PathBuf,
    #[serde(default = "default_port")]
    pub port: u16,
    #[serde(default = "default_database_path")]
    pub database_path: String,
    #[serde(default = "default_secret")]
    pub secret: String,
    #[serde(default = "default_update_interval")]
    pub update_interval: u64,
    #[serde(default = "default_plugin_path")]
    pub plugin_path: String,
    #[serde(default = "default_local_path")]
    pub local_path: String,
    #[serde(default)]
    pub enable_playground: bool,
    pub telegram: Option<TelegramConfig>,
    pub pushover: Option<PushoverConfig>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            path: tanoshi_home().join("config.yml"),
            port: default_port(),
            database_path: default_database_path(),
            secret: default_secret(),
            update_interval: default_update_interval(),
            plugin_path: default_plugin_path(),
            local_path: default_local_path(),
            enable_playground: false,
            telegram: None,
            pushover: None,
        }
    }
}

fn tanoshi_home() -> PathBuf {
    match std::env::var("TANOSHI_HOME") {
        Ok(path) => PathBuf::from(path),
        Err(_) => dirs::home_dir().expect("should have home").join(".tanoshi"),
    }
}

fn default_port() -> u16 {
    80
}

fn default_update_interval() -> u64 {
    3600
}

fn default_secret() -> String {
    let mut rng = thread_rng();
    let chars = iter::repeat(())
        .map(|()| rng.sample(Alphanumeric))
        .take(16)
        .collect();
    String::from_utf8(chars).unwrap()
}

fn default_database_path() -> String {
    let path = tanoshi_home();
    if !path.exists() {
        let _ = std::fs::create_dir_all(&path);
    }
    path.join("tanoshi.db").to_str().unwrap().to_string()
}

fn default_plugin_path() -> String {
    let path = tanoshi_home().join("plugins");
    if !path.exists() {
        let _ = std::fs::create_dir_all(&path);
    }
    path.to_str().unwrap().to_string()
}

fn default_local_path() -> String {
    let path = tanoshi_home().join("manga");
    if !path.exists() {
        let _ = std::fs::create_dir_all(&path);
    }
    path.to_str().unwrap().to_string()
}

impl Config {
    pub fn open<P: AsRef<Path>>(path: Option<P>) -> Result<Config, Box<dyn std::error::Error>> {
        let config_path = match path {
            Some(p) => PathBuf::new().join(p),
            None => tanoshi_home().join("config.yml"),
        };

        match std::fs::File::open(config_path.clone()) {
            Ok(file) => {
                info!("Open config from {:?}", config_path);
                let mut cfg: Self = serde_yaml::from_reader(file)?;
                cfg.path = config_path;
                Ok(cfg)
            }
            Err(_) => {
                let cfg = Config {
                    path: config_path,
                    ..Default::default()
                };
                cfg.save()?;
                info!("Write default config at {:?}", cfg.path);
                Ok(cfg)
            }
        }
    }

    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        std::fs::write(&self.path, serde_yaml::to_string(&self)?)?;

        Ok(())
    }
}
