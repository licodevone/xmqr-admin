// SPDX-License-Identifier: MIT
use serde::Deserialize;
use std::{
    env,
    io::{self, Read},
    net::SocketAddr,
    path::{Path, PathBuf},
};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Document {
    version: u32,
    bind: String,
    monitor: String,
    token_file: PathBuf,
    #[serde(default)]
    users_file: Option<PathBuf>,
}

#[derive(Clone)]
pub struct Config {
    pub bind: SocketAddr,
    pub monitor: SocketAddr,
    pub token_file: PathBuf,
    pub users_file: Option<PathBuf>,
}

impl Config {
    pub fn from_file(path: &Path) -> io::Result<Self> {
        if !path.is_absolute() {
            return Err(invalid("configuration path must be absolute"));
        }
        let file = std::fs::File::open(path)?;
        if !file.metadata()?.is_file() {
            return Err(invalid("configuration must be a regular file"));
        }
        let mut text = String::new();
        file.take(65_537).read_to_string(&mut text)?;
        if text.len() > 65_536 {
            return Err(invalid("configuration exceeds 64 KiB"));
        }
        let document: Document =
            toml::from_str(&text).map_err(|_| invalid("invalid configuration schema"))?;
        if document.version != 1 || document.token_file.as_os_str().is_empty() {
            return Err(invalid("unsupported configuration"));
        }
        let token_file = if document.token_file.is_absolute() {
            document.token_file
        } else {
            path.parent()
                .ok_or_else(|| invalid("configuration needs parent directory"))?
                .join(document.token_file)
        };
        let mut config = Self::parse(&document.bind, &document.monitor, token_file)?;
        config.users_file = document
            .users_file
            .map(|users| {
                if users.as_os_str().is_empty() {
                    return Err(invalid("users file path must not be empty"));
                }
                Ok(if users.is_absolute() {
                    users
                } else {
                    path.parent()
                        .ok_or_else(|| invalid("configuration needs parent directory"))?
                        .join(users)
                })
            })
            .transpose()?;
        Ok(config)
    }

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
            users_file: None,
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
    fn config_file_is_bounded_strict_and_preserves_loopback() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("admin.toml");
        let text = "version=1\nbind='127.0.0.1:9080'\nmonitor='127.0.0.1:9090'\ntoken_file='token'";
        std::fs::write(&path, text).unwrap();
        assert_eq!(
            Config::from_file(&path).unwrap().token_file,
            dir.path().join("token")
        );
        std::fs::write(&path, format!("{text}\nusers_file='state/users.json'")).unwrap();
        assert_eq!(
            Config::from_file(&path).unwrap().users_file,
            Some(dir.path().join("state/users.json"))
        );
        for invalid_text in [
            text.replace("version=1", "version=2"),
            text.replace("127.0.0.1:9080", "0.0.0.0:9080"),
            format!("{text}\npassword='no'"),
            " ".repeat(65_537),
        ] {
            std::fs::write(&path, invalid_text).unwrap();
            assert!(Config::from_file(&path).is_err());
        }
        assert!(Config::from_file(Path::new("relative.toml")).is_err());
    }

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
