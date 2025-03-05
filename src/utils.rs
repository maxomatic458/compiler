use semver::Version;
use thiserror::Error;

#[derive(Error, Debug, Clone, PartialEq)]
pub enum ClangError {
    #[error("clang version {0} is not supported")]
    UnsupportedVersion(Version),
    #[error("clang is not installed")]
    NotInstalled,
    #[error("clang error: {0}")]
    ClangErrorCode(i32),
}

pub fn get_clang_version() -> Option<Version> {
    let output = std::process::Command::new("clang")
        .arg("--version")
        .output()
        .ok()?;

    let output = String::from_utf8(output.stdout).ok()?;
    let version = output.lines().next()?.split(' ').nth(2)?;

    Version::parse(version).ok()
}
