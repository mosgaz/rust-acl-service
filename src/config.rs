use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub http_addr: String,
    pub metrics_addr: String,
    pub database_url: String,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        let http_addr = env::var("HTTP_ADDR").unwrap_or_else(|_| "0.0.0.0:8080".to_string());
        let metrics_addr = env::var("METRICS_ADDR").unwrap_or_else(|_| "0.0.0.0:9000".to_string());
        let database_url = env::var("DATABASE_URL")?;

        Ok(Self {
            http_addr,
            metrics_addr,
            database_url,
        })
    }
}
