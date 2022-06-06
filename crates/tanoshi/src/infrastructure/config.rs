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
pub struct GotifyConfig {
    pub base_url: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct MyAnimeListConfig {
    pub client_id: String,
    pub client_secret: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct AniListConfig {
    pub client_id: String,
    pub client_secret: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LocalFolder {
    pub name: String,
    pub path: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum LocalFolders {
    Single(String),
    Multiple(Vec<LocalFolder>),
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Config {
    #[serde(skip)]
    path: PathBuf,
    #[serde(skip, default = "default_extension_repository")]
    pub extension_repository: String,
    #[serde(default)]
    pub base_url: Option<String>,
    #[serde(default = "default_port")]
    pub port: u16,
    #[serde(default = "default_database_path")]
    pub database_path: String,
    #[serde(default = "default_create_database")]
    pub create_database: bool,
    #[serde(default = "default_secret")]
    pub secret: String,
    #[serde(default = "default_update_interval")]
    pub update_interval: u64,
    #[serde(default)]
    pub auto_download_chapters: bool,
    #[serde(default = "default_plugin_path")]
    pub plugin_path: String,
    #[serde(default = "default_local_folders")]
    pub local_path: LocalFolders,
    #[serde(default = "default_download_path")]
    pub download_path: String,
    #[serde(default = "default_cache_path")]
    pub cache_path: String,
    #[serde(default)]
    pub enable_playground: bool,
    pub telegram: Option<TelegramConfig>,
    pub pushover: Option<PushoverConfig>,
    pub gotify: Option<GotifyConfig>,
    pub myanimelist: Option<MyAnimeListConfig>,
    pub anilist: Option<AniListConfig>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            path: tanoshi_home().join("config.yml"),
            extension_repository: default_extension_repository(),
            base_url: None,
            port: default_port(),
            database_path: default_database_path(),
            create_database: default_create_database(),
            secret: default_secret(),
            update_interval: default_update_interval(),
            auto_download_chapters: false,
            plugin_path: default_plugin_path(),
            local_path: default_local_folders(),
            download_path: default_download_path(),
            cache_path: default_cache_path(),
            enable_playground: false,
            telegram: None,
            pushover: None,
            gotify: None,
            myanimelist: None,
            anilist: None,
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

fn default_extension_repository() -> String {
    "https://raw.githubusercontent.com/faldez/tanoshi-extensions/repository".to_string()
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
    path.join("tanoshi.db").display().to_string()
}

fn default_create_database() -> bool {
    true
}

fn default_plugin_path() -> String {
    let path = tanoshi_home().join("plugins");
    if !path.exists() {
        let _ = std::fs::create_dir_all(&path);
    }
    path.display().to_string()
}

fn default_local_folders() -> LocalFolders {
    LocalFolders::Single(default_local_path())
}

fn default_local_path() -> String {
    let path = tanoshi_home().join("manga");
    if !path.exists() {
        let _ = std::fs::create_dir_all(&path);
    }
    path.display().to_string()
}

fn default_download_path() -> String {
    let path = tanoshi_home().join("downloads");
    if !path.exists() {
        let _ = std::fs::create_dir_all(&path);
    }
    path.display().to_string()
}

fn default_cache_path() -> String {
    let path = tanoshi_home().join("cache");
    if !path.exists() {
        let _ = std::fs::create_dir_all(&path);
    }
    path.display().to_string()
}

impl Config {
    pub fn open<P: AsRef<Path>>(path: Option<P>) -> Result<Config, anyhow::Error> {
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

    pub fn save(&self) -> Result<(), anyhow::Error> {
        std::fs::write(&self.path, serde_yaml::to_string(&self)?)?;

        Ok(())
    }
}
