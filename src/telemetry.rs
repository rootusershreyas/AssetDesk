use std::{env, io};

use tracing_subscriber::EnvFilter;

pub fn init() -> io::Result<()> {
    let directives = match env::var("RUST_LOG") {
        Ok(value) => value,
        Err(env::VarError::NotPresent) => "info,tower_http=debug".to_owned(),
        Err(_) => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "RUST_LOG must contain valid text",
            ));
        }
    };

    let filter = EnvFilter::try_new(directives).map_err(|_| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            "RUST_LOG contains invalid filter directives",
        )
    })?;

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .json()
        .with_ansi(false)
        .try_init()
        .map_err(io::Error::other)
}
