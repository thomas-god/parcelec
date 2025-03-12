use std::env;

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub allow_origin: String,
    pub domain: String,
    pub port: u16,
}

impl AppConfig {
    pub fn from_env() -> AppConfig {
        AppConfig {
            allow_origin: env::var("ALLOW_ORIGIN").expect("No ALLOW_ORIGIN environnement variable"),
            domain: env::var("DOMAIN").expect("No DOMAIN environnement variable"),
            port: env::var("API_PORT")
                .unwrap_or("9002".to_owned())
                .parse::<u16>()
                .unwrap_or(9002),
        }
    }
}

#[cfg(test)]
impl Default for AppConfig {
    fn default() -> Self {
        AppConfig {
            allow_origin: String::from("http://127.0.0.1:5173"),
            port: 9003,
            domain: String::from("127.0.0.1"),
        }
    }
}
