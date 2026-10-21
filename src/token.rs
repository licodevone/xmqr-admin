// SPDX-License-Identifier: MIT
#[cfg(unix)]
use std::fs;
use std::{
    fs::File,
    io::{self, Read},
    path::Path,
};
use subtle::ConstantTimeEq;
use zeroize::Zeroizing;

pub struct Token(Zeroizing<Vec<u8>>);

impl Token {
    #[cfg_attr(windows, allow(dead_code))]
    pub fn as_str(&self) -> &str {
        std::str::from_utf8(&self.0).expect("validated hexadecimal token")
    }

    pub fn matches(&self, candidate: &[u8]) -> bool {
        candidate.len() == 64 && bool::from(self.0.as_slice().ct_eq(candidate))
    }
}

pub fn read(path: &Path) -> io::Result<Token> {
    let file = private_file(path)?;
    let mut bytes = Zeroizing::new(Vec::new());
    file.take(129).read_to_end(&mut bytes)?;
    if bytes.len() > 128 {
        return Err(invalid("token file exceeds limit"));
    }
    while bytes.last().is_some_and(u8::is_ascii_whitespace) {
        bytes.pop();
    }
    if bytes.len() != 64 || !bytes.iter().all(u8::is_ascii_hexdigit) {
        return Err(invalid("token must be 64 hexadecimal bytes"));
    }
    Ok(Token(bytes))
}

#[cfg(unix)]
fn private_file(path: &Path) -> io::Result<File> {
    use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};
    let parent = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .ok_or_else(|| invalid("explicit private parent directory required"))?;
    let parent_metadata = fs::symlink_metadata(parent)?;
    if !parent_metadata.is_dir() || parent_metadata.permissions().mode() & 0o077 != 0 {
        return Err(invalid("token parent must be a private directory"));
    }
    let file = fs::OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK)
        .open(path)?;
    let metadata = file.metadata()?;
    if !metadata.is_file() || metadata.permissions().mode() & 0o777 != 0o600 {
        return Err(invalid(
            "token must be a regular private file with mode 0600",
        ));
    }
    Ok(file)
}

#[cfg(windows)]
fn private_file(path: &Path) -> io::Result<File> {
    crate::windows_private::open(path, false, false)
}

#[cfg(not(any(unix, windows)))]
fn private_file(_: &Path) -> io::Result<File> {
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "private token files require Unix",
    ))
}

#[cfg(unix)]
pub fn create(path: &Path) -> io::Result<()> {
    use rand_core::{OsRng, RngCore};
    use std::{
        fmt::Write as _,
        fs::OpenOptions,
        io::Write,
        os::unix::fs::{OpenOptionsExt, PermissionsExt},
    };
    let parent = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .ok_or_else(|| invalid("explicit private parent directory required"))?;
    let metadata = fs::symlink_metadata(parent)?;
    if !metadata.is_dir() || metadata.permissions().mode() & 0o077 != 0 {
        return Err(invalid("token parent must be a private directory"));
    }
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(path)?;
    let mut random = Zeroizing::new([0_u8; 32]);
    OsRng.fill_bytes(random.as_mut());
    let mut value = Zeroizing::new(String::with_capacity(65));
    for byte in random.iter() {
        write!(&mut *value, "{byte:02x}").expect("String formatting cannot fail");
    }
    value.push('\n');
    file.write_all(value.as_bytes())?;
    file.sync_all()?;
    File::open(parent)?.sync_all()
}

#[cfg(windows)]
pub fn create(path: &Path) -> io::Result<()> {
    use rand_core::{OsRng, RngCore};
    use std::{fmt::Write as _, io::Write};
    let mut file = crate::windows_private::open(path, true, true)?;
    let mut random = Zeroizing::new([0_u8; 32]);
    OsRng.fill_bytes(random.as_mut());
    let mut value = Zeroizing::new(String::with_capacity(65));
    for byte in random.iter() {
        write!(&mut *value, "{byte:02x}").expect("String formatting cannot fail");
    }
    value.push('\n');
    file.write_all(value.as_bytes())?;
    file.sync_all()
}

#[cfg(not(any(unix, windows)))]
pub fn create(_: &Path) -> io::Result<()> {
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "token creation requires Unix",
    ))
}

fn invalid(message: &'static str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidInput, message)
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    use std::os::unix::fs::{PermissionsExt, symlink};

    #[test]
    fn random_token_private_file_matches_exactly_and_never_overwrites() {
        let directory = tempfile::tempdir().unwrap();
        fs::set_permissions(directory.path(), fs::Permissions::from_mode(0o700)).unwrap();
        let path = directory.path().join("access.token");
        create(&path).unwrap();
        let bytes = fs::read(&path).unwrap();
        let token = read(&path).unwrap();
        assert!(token.matches(&bytes[..64]));
        assert!(!token.matches(&[]));
        assert!(!token.matches(&[b'x'; 64]));
        assert!(create(&path).is_err());
        assert_eq!(fs::read(&path).unwrap(), bytes);
    }

    #[test]
    fn permissions_symlinks_directories_and_invalid_values_are_refused() {
        let directory = tempfile::tempdir().unwrap();
        fs::set_permissions(directory.path(), fs::Permissions::from_mode(0o700)).unwrap();
        let path = directory.path().join("access.token");
        fs::write(&path, [b'a'; 64]).unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).unwrap();
        assert!(read(&path).unwrap().matches(&[b'a'; 64]));
        fs::set_permissions(&path, fs::Permissions::from_mode(0o644)).unwrap();
        assert!(read(&path).is_err());
        fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).unwrap();
        let link = directory.path().join("link.token");
        symlink(&path, &link).unwrap();
        assert!(read(&link).is_err());
        assert!(read(directory.path()).is_err());
        for bytes in [vec![b'a'; 63], vec![b'g'; 64], vec![b'a'; 129]] {
            fs::write(&path, bytes).unwrap();
            assert!(read(&path).is_err());
        }
    }

    #[test]
    fn public_parent_and_fifo_are_refused_without_blocking() {
        let directory = tempfile::tempdir().unwrap();
        fs::set_permissions(directory.path(), fs::Permissions::from_mode(0o700)).unwrap();
        let path = directory.path().join("fifo.token");
        assert!(
            std::process::Command::new("mkfifo")
                .arg(&path)
                .status()
                .unwrap()
                .success()
        );
        assert!(read(&path).is_err());
        let path = directory.path().join("access.token");
        fs::write(&path, [b'a'; 64]).unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).unwrap();
        assert!(read(&path).unwrap().matches(&[b'a'; 64]));
        fs::set_permissions(directory.path(), fs::Permissions::from_mode(0o755)).unwrap();
        assert!(read(&path).is_err());
    }
}
