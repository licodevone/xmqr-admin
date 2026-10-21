// SPDX-License-Identifier: MIT
//! Private NTFS files. ACLs are checked on open handles, never inferred from readonly.
use std::{
    fs::{File, OpenOptions},
    io,
    os::windows::fs::{MetadataExt, OpenOptionsExt},
    path::{Component, Path, Prefix},
};
use windows_permissions::{
    LocalBox, SecurityDescriptor,
    constants::{AceType, SeObjectType, SecurityInformation},
    wrappers,
};

const REPARSE: u32 = 0x400;
const OPEN_REPARSE: u32 = 0x0020_0000;
const BACKUP_SEMANTICS: u32 = 0x0200_0000;
const READ_CONTROL: u32 = 0x0002_0000;
const WRITE_DAC: u32 = 0x0004_0000;

fn denied() -> io::Error {
    io::Error::new(
        io::ErrorKind::PermissionDenied,
        "private Windows file requires a trusted owner, restrictive DACL and no reparse points",
    )
}

fn trusted(sid: &str, current: &str) -> bool {
    sid == current || matches!(sid, "S-1-5-18" | "S-1-5-19" | "S-1-5-32-544")
}

pub fn validate(file: &File) -> io::Result<()> {
    let identity = current_sid()?;
    let metadata = file.metadata()?;
    if metadata.file_attributes() & REPARSE != 0 {
        return Err(denied());
    }
    let sd = wrappers::GetSecurityInfo(
        file,
        SeObjectType::SE_FILE_OBJECT,
        SecurityInformation::Owner | SecurityInformation::Dacl,
    )?;
    validate_descriptor(&sd, &identity)
}

fn validate_descriptor(sd: &SecurityDescriptor, identity: &str) -> io::Result<()> {
    if !sd
        .owner()
        .is_some_and(|owner| trusted(&owner.to_string(), identity))
    {
        return Err(denied());
    }
    let acl = sd.dacl().ok_or_else(denied)?;
    if acl.len() == 0 {
        return Err(denied());
    }
    for index in 0..acl.len() {
        let ace = acl.get_ace(index).ok_or_else(denied)?;
        // Unknown, callback and object ACE semantics fail closed. Broad grants
        // are rejected even when a deny ACE might currently mask them.
        match ace.ace_type() {
            AceType::ACCESS_DENIED_ACE_TYPE => {}
            AceType::ACCESS_ALLOWED_ACE_TYPE
                if ace
                    .sid()
                    .is_some_and(|sid| trusted(&sid.to_string(), identity)) => {}
            _ => return Err(denied()),
        }
    }
    Ok(())
}

pub fn pin_parent(path: &Path) -> io::Result<Vec<File>> {
    if !path.is_absolute()
        || !matches!(path.components().next(), Some(Component::Prefix(prefix)) if matches!(prefix.kind(), Prefix::Disk(_) | Prefix::VerbatimDisk(_)))
        || path.components().any(|c| match c {
            Component::ParentDir | Component::CurDir => true,
            Component::Normal(name) => {
                let name = name.to_string_lossy();
                let base = name
                    .split('.')
                    .next()
                    .unwrap_or_default()
                    .to_ascii_uppercase();
                name.contains(':')
                    || name.ends_with(['.', ' '])
                    || matches!(base.as_str(), "CON" | "PRN" | "AUX" | "NUL")
                    || (base.len() == 4
                        && (base.starts_with("COM") || base.starts_with("LPT"))
                        && base.as_bytes()[3].is_ascii_digit())
            }
            _ => false,
        })
    {
        return Err(denied());
    }
    let mut chain: Vec<_> = path.parent().ok_or_else(denied)?.ancestors().collect();
    chain.reverse();
    let mut handles = Vec::new();
    for ancestor in chain {
        let handle = OpenOptions::new()
            .access_mode(READ_CONTROL)
            .share_mode(3)
            .custom_flags(BACKUP_SEMANTICS | OPEN_REPARSE)
            .open(ancestor)?;
        let metadata = handle.metadata()?;
        if !metadata.is_dir() || metadata.file_attributes() & REPARSE != 0 {
            return Err(denied());
        }
        handles.push(handle);
    }
    validate(handles.last().ok_or_else(denied)?)?;
    Ok(handles)
}

pub fn parent(path: &Path) -> io::Result<()> {
    pin_parent(path).map(|_| ())
}

pub fn open(path: &Path, write: bool, create_new: bool) -> io::Result<File> {
    let _parents = pin_parent(path)?;
    let mut options = OpenOptions::new();
    options
        .read(true)
        .write(write)
        .share_mode(3)
        .custom_flags(OPEN_REPARSE);
    if create_new {
        options
            .create_new(true)
            .access_mode(0x8000_0000 | 0x4000_0000 | READ_CONTROL | WRITE_DAC);
    }
    let mut file = options.open(path)?;
    if !file.metadata()?.is_file() {
        return Err(denied());
    }
    if create_new {
        // No secret is written until this protected DACL is applied and read back.
        restrict_new(&mut file)?;
    }
    validate(&file)?;
    Ok(file)
}

/// Apply to a freshly created, empty file before writing private material.
pub fn restrict_new(file: &mut File) -> io::Result<()> {
    if file.metadata()?.len() != 0 {
        return Err(denied());
    }
    let identity = current_sid()?;
    let sd: LocalBox<SecurityDescriptor> =
        format!("D:P(A;;FA;;;{identity})(A;;FA;;;SY)(A;;FA;;;BA)(A;;FA;;;LS)").parse()?;
    wrappers::SetSecurityInfo(
        file,
        SeObjectType::SE_FILE_OBJECT,
        SecurityInformation::Dacl | SecurityInformation::ProtectedDacl,
        None,
        None,
        sd.dacl(),
        None,
    )?;
    validate(file)
}

pub fn open_or_create(path: &Path) -> io::Result<File> {
    match open(path, true, true) {
        Ok(file) => Ok(file),
        Err(error) if error.kind() == io::ErrorKind::AlreadyExists => open(path, true, false),
        Err(error) => Err(error),
    }
}

pub fn current_sid() -> io::Result<String> {
    use winsafe::{HPROCESS, TokenInfo, co};
    let token = HPROCESS::GetCurrentProcess()
        .OpenProcessToken(co::TOKEN::QUERY)
        .map_err(io::Error::other)?;
    match token
        .GetTokenInformation(co::TOKEN_INFORMATION_CLASS::User)
        .map_err(io::Error::other)?
    {
        TokenInfo::User(user) => user
            .User
            .Sid()
            .map(|sid| sid.to_string())
            .ok_or_else(denied),
        _ => Err(denied()),
    }
}

#[cfg(test)]
pub fn private_directory(path: &Path) {
    let identity = current_sid().unwrap();
    let sd: LocalBox<SecurityDescriptor> =
        format!("D:P(A;OICI;FA;;;{identity})(A;OICI;FA;;;SY)(A;OICI;FA;;;BA)(A;OICI;FA;;;LS)")
            .parse()
            .unwrap();
    let mut handle = OpenOptions::new()
        .access_mode(READ_CONTROL | WRITE_DAC)
        .share_mode(3)
        .custom_flags(BACKUP_SEMANTICS | OPEN_REPARSE)
        .open(path)
        .unwrap();
    wrappers::SetSecurityInfo(
        &mut handle,
        SeObjectType::SE_FILE_OBJECT,
        SecurityInformation::Dacl | SecurityInformation::ProtectedDacl,
        None,
        None,
        sd.dacl(),
        None,
    )
    .unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    #[test]
    fn dacl_rejects_public_null_empty_and_untrusted_owner() {
        let current = current_sid().unwrap();
        for text in [
            format!("O:{current}D:(A;;FA;;;WD)"),
            format!("O:{current}D:(A;;FR;;;BU)"),
            format!("O:{current}D:(D;;FA;;;WD)(A;;FR;;;WD)"),
            format!("O:{current}D:"),
            format!("O:{current}"),
            "O:S-1-5-21-1-2-3-1000D:(A;;FA;;;SY)".into(),
        ] {
            let sd: LocalBox<SecurityDescriptor> = text.parse().unwrap();
            assert!(validate_descriptor(&sd, &current).is_err(), "{text}");
        }
    }

    #[test]
    fn local_service_accepts_privileged_provisioning_and_rejects_operator_grants() {
        let service = "S-1-5-19";
        let normalized: LocalBox<SecurityDescriptor> =
            "O:BAD:P(A;;FA;;;SY)(A;;FA;;;BA)(A;;FR;;;LS)"
                .parse()
                .unwrap();
        assert!(validate_descriptor(&normalized, service).is_ok());
        let operator_owned: LocalBox<SecurityDescriptor> =
            "O:S-1-5-21-1-2-3-1000D:P(A;;FA;;;SY)(A;;FA;;;BA)(A;;FR;;;LS)"
                .parse()
                .unwrap();
        assert!(validate_descriptor(&operator_owned, service).is_err());
        let operator_grant: LocalBox<SecurityDescriptor> =
            "O:BAD:P(A;;FA;;;SY)(A;;FA;;;BA)(A;;FR;;;LS)(A;;FR;;;S-1-5-21-1-2-3-1000)"
                .parse()
                .unwrap();
        assert!(validate_descriptor(&operator_grant, service).is_err());
    }
    #[test]
    fn token_roundtrip_refuses_overwrite_public_acl_and_reparse_parent() {
        let directory = tempfile::tempdir().unwrap();
        private_directory(directory.path());
        let token = directory.path().join("token");
        crate::token::create(&token).unwrap();
        let original = fs::read(&token).unwrap();
        assert!(crate::token::read(&token).unwrap().matches(&original[..64]));
        assert!(crate::token::create(&token).is_err());
        assert_eq!(fs::read(&token).unwrap(), original);
        let mut file = OpenOptions::new()
            .access_mode(READ_CONTROL | WRITE_DAC)
            .open(&token)
            .unwrap();
        let public: LocalBox<SecurityDescriptor> = "D:P(A;;FA;;;WD)".parse().unwrap();
        wrappers::SetSecurityInfo(
            &mut file,
            SeObjectType::SE_FILE_OBJECT,
            SecurityInformation::Dacl | SecurityInformation::ProtectedDacl,
            None,
            None,
            public.dacl(),
            None,
        )
        .unwrap();
        drop(file);
        assert!(crate::token::read(&token).is_err());
        for name in ["token:stream", "token.", "token ", "NUL", "COM1.txt"] {
            assert!(open(&directory.path().join(name), true, true).is_err());
        }
        let mut parent = OpenOptions::new()
            .access_mode(READ_CONTROL | WRITE_DAC)
            .custom_flags(BACKUP_SEMANTICS | OPEN_REPARSE)
            .open(directory.path())
            .unwrap();
        wrappers::SetSecurityInfo(
            &mut parent,
            SeObjectType::SE_FILE_OBJECT,
            SecurityInformation::Dacl | SecurityInformation::ProtectedDacl,
            None,
            None,
            public.dacl(),
            None,
        )
        .unwrap();
        drop(parent);
        assert!(open(&directory.path().join("new-token"), true, true).is_err());
        assert!(!directory.path().join("new-token").exists());
    }

    #[test]
    fn real_system_junction_ancestor_is_rejected_without_following() {
        // Reuse Windows' compatibility junction because this sandbox forbids
        // creating reparse points. An explicit fixture can be used by CI.
        let junction = std::env::var_os("XMQR_TEST_REPARSE_DIRECTORY").map_or_else(
            || {
                let drive = std::env::var_os("SystemDrive").unwrap_or_else(|| "C:".into());
                std::path::PathBuf::from(format!(
                    "{}\\Documents and Settings",
                    drive.to_string_lossy()
                ))
            },
            std::path::PathBuf::from,
        );
        assert_ne!(
            fs::symlink_metadata(&junction).unwrap().file_attributes() & REPARSE,
            0
        );
        let error = open(&junction.join("xmqr-test-token"), false, false).unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::PermissionDenied);
        assert_eq!(error.to_string(), denied().to_string());
    }
}
