// SPDX-License-Identifier: MIT
use std::{env, io, net::SocketAddr, path::PathBuf};

#[derive(Clone)]
pub struct Config {
    pub bind: SocketAddr,
    pub monitor: SocketAddr,
    pub token_file: PathBuf,
}

impl Config {
    pub fn from_env() -> io::Result<Self> {
        let token_file = env::var_os("XMQR_ADMIN_TOKEN_FILE")
            .ok_or_else(|| invalid("XMQR_ADMIN_TOKEN_FILE is required"))?;
        Self::parse(
            &setting("XMQR_ADMIN_BIND", "127.0.0.1:9080")?,
            &setting("XMQR_ADMIN_MONITOR_BIND", "127.0.0.1:9090")?,
            token_file.into(),
        )
    }

    pub fn parse(bind: &str, monitor: &str, token_file: PathBuf) -> io::Result<Self> {
        Ok(Self {
            bind: loopback(bind)?,
            monitor: loopback(monitor)?,
            token_file,
        })
    }

    pub fn allows_host(&self, host: &str) -> bool {
        host == self.bind.to_string() || host == format!("localhost:{}", self.bind.port())
    }

    pub fn allows_origin(&self, origin: &str) -> bool {
        origin
            .strip_prefix("http://")
            .is_some_and(|host| self.allows_host(host))
    }
}

fn loopback(value: &str) -> io::Result<SocketAddr> {
    let address: SocketAddr = value
        .parse()
        .map_err(|_| invalid("invalid local address"))?;
    if !address.ip().is_loopback() || address.port() == 0 {
        return Err(invalid(
            "only loopback addresses with a nonzero port are allowed",
        ));
    }
    Ok(address)
}

fn setting(name: &'static str, default: &'static str) -> io::Result<String> {
    match env::var(name) {
        Ok(value) => Ok(value),
        Err(env::VarError::NotPresent) => Ok(default.into()),
        Err(env::VarError::NotUnicode(_)) => Err(invalid("local address must be UTF-8")),
    }
}

fn invalid(message: &'static str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidInput, message)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn remote_wildcard_hostname_port_zero_and_urls_are_refused() {
        for value in [
            "0.0.0.0:9080",
            "[::]:9080",
            "192.0.2.1:9080",
            "localhost:9080",
            "127.0.0.1:0",
            "http://127.0.0.1:9080",
        ] {
            assert!(Config::parse(value, "127.0.0.1:9090", "unused".into()).is_err());
            assert!(Config::parse("127.0.0.1:9080", value, "unused".into()).is_err());
        }
    }

    #[test]
    fn host_and_origin_are_exact_no_rebinding_or_suffixes() {
        let config = Config::parse("127.0.0.1:9080", "[::1]:9090", "unused".into()).unwrap();
        for host in ["127.0.0.1:9080", "localhost:9080"] {
            assert!(config.allows_host(host));
        }
        for host in [
            "evil.test:9080",
            "localhost.evil.test:9080",
            "127.0.0.1:1",
            "localhost",
            "user@localhost:9080",
        ] {
            assert!(!config.allows_host(host));
        }
        assert!(config.allows_origin("http://localhost:9080"));
        for origin in [
            "null",
            "https://localhost:9080",
            "http://localhost:9080/",
            "http://evil.test:9080",
        ] {
            assert!(!config.allows_origin(origin));
        }
    }
}
