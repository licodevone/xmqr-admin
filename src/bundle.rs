//! Read-only validation using the immutable core dependency, without opening MQTT state.
use std::{io, path::Path};

pub fn validate(path: &Path) -> io::Result<()> {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?
        .block_on(mqtt_broker::auth::DynamicPolicy::load(
            path.to_path_buf(),
            true,
        ))
        .map_err(|_| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                "bundle seguro invalido ou arquivo sem protecao adequada",
            )
        })?;
    Ok(())
}
