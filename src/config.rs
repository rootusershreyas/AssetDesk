use std::{
    env,
    error::Error,
    net::{IpAddr, SocketAddr},
};

pub struct Config {
    pub address: SocketAddr,
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

        Ok(Self {
            address: SocketAddr::new(host, port),
        })
    }
}
