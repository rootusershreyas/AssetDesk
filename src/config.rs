use std::{
    env,
    error::Error,
    net::{IpAddr, SocketAddr},
};

pub struct Config {
    pub address: SocketAddr,
    pub database_url: String,
    pub database_max_connections: u32,
}

impl Config {
    pub fn from_env() -> Result<Self, Box<dyn Error>> {
        let host = env::var("HOST").map_err(|_| "HOST is required and must contain valid text")?;

        let host = host
            .parse::<IpAddr>()
            .map_err(|_| "HOST must be a valid IP address")?;

        let port = env::var("PORT").map_err(|_| "PORT is required and must contain valid text")?;

        let port = port
            .parse::<u16>()
            .map_err(|_| "PORT must be between 1 and 65535")?;

        if port == 0 {
            return Err("PORT must be between 1 and 65535".into());
        }
        let database_url = env::var("DATABASE_URL").map_err(|_| "DATABASE_URL is required")?;

        if database_url.trim().is_empty() {
            return Err("DATABASE_URL must not be empty".into());
        }

        let database_max_connections = env::var("DATABASE_MAX_CONNECTIONS")
            .map_err(|_| "DATABASE_MAX_CONNECTIONS is required")?
            .parse::<u32>()
            .map_err(|_| "DATABASE_MAX_CONNECTIONS must be a positive integer")?;

        if database_max_connections == 0 {
            return Err("DATABASE_MAX_CONNECTIONS must be a positive integer".into());
        }

        Ok(Self {
            address: SocketAddr::new(host, port),
            database_url,
            database_max_connections,
        })
    }
}
