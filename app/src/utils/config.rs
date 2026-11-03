use std::{env, path::PathBuf};

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub allow_origin: String,
    pub port: u16,
    pub static_dir: PathBuf,
}

impl AppConfig {
    pub fn from_env() -> AppConfig {
        AppConfig {
            allow_origin: env::var("ALLOW_ORIGIN").expect("No ALLOW_ORIGIN environnement variable"),
            port: env::var("API_PORT")
                .unwrap_or("9002".to_owned())
                .parse::<u16>()
                .unwrap_or(9002),
            static_dir: env::var("STATIC_DIR")
                .unwrap_or_else(|_| "/public".to_owned())
                .into(),
        }
    }
}

#[cfg(test)]
impl Default for AppConfig {
    fn default() -> Self {
        AppConfig {
            allow_origin: String::from("http://127.0.0.1:5173"),
            port: 9003,
            static_dir: PathBuf::from("../client/build"),
        }
    }
}
