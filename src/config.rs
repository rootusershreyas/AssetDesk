use std::net::{IpAddr, SocketAddr};

pub struct Config {
    pub database_url: String,
    pub database_max_connections: u32,
    pub address: SocketAddr,
}

impl Config {
    pub fn from_env() -> Result<Self, String> {
        Self::from_lookup(|key| std::env::var(key).ok())
    }

    fn from_lookup(lookup: impl Fn(&str) -> Option<String>) -> Result<Self, String> {
        let database_url = lookup("DATABASE_URL")
            .filter(|value| !value.trim().is_empty())
            .ok_or("DATABASE_URL must be set")?;
        let database_max_connections: u32 = lookup("DATABASE_MAX_CONNECTIONS")
            .unwrap_or_else(|| "10".into())
            .parse()
            .map_err(|_| "DATABASE_MAX_CONNECTIONS must be a positive integer")?;
        if database_max_connections == 0 {
            return Err("DATABASE_MAX_CONNECTIONS must be greater than zero".into());
        }
        let host: IpAddr = lookup("HOST")
            .unwrap_or_else(|| "127.0.0.1".into())
            .parse()
            .map_err(|_| "HOST must be an IP address")?;
        let port: u16 = lookup("PORT")
            .unwrap_or_else(|| "3000".into())
            .parse()
            .map_err(|_| "PORT must be an integer between 1 and 65535")?;
        if port == 0 {
            return Err("PORT must be an integer between 1 and 65535".into());
        }
        Ok(Self {
            database_url,
            database_max_connections,
            address: SocketAddr::new(host, port),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::Config;

    #[test]
    fn defaults_and_invalid_configuration() {
        let config =
            Config::from_lookup(|key| (key == "DATABASE_URL").then(|| "postgres://example".into()))
                .unwrap();
        assert_eq!(config.database_max_connections, 10);
        assert_eq!(config.address.to_string(), "127.0.0.1:3000");
        assert!(Config::from_lookup(|_| None).is_err());

        for (key, value) in [
            ("DATABASE_MAX_CONNECTIONS", "0"),
            ("DATABASE_MAX_CONNECTIONS", "bad"),
            ("HOST", "not-an-address"),
            ("PORT", "0"),
            ("PORT", "65536"),
        ] {
            assert!(
                Config::from_lookup(|name| match name {
                    "DATABASE_URL" => Some("postgres://example".into()),
                    name if name == key => Some(value.into()),
                    _ => None,
                })
                .is_err(),
                "invalid {key} was accepted"
            );
        }
    }
}
