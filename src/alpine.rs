use crate::{arch::Arch, http};
use serde::Deserialize;
use serde_yaml;
use sha2::{Digest, Sha512};
use std::fs::File;
use std::io::{copy, Read, Result as IoResult};
use std::path::Path;

type Result<T> = core::result::Result<T, Error>;

macro_rules! err {
    ($($args:expr),+) => {
        return Err(Error::new(format!($($args,)+)))
    };
}

pub struct BaseSystemDownloader {
    client: http::Client,
}

impl BaseSystemDownloader {
    pub fn new(client: http::Client) -> Self {
        Self { client }
    }

    pub fn download<P: AsRef<Path>>(&self, architecture: &Arch, destination_path: P) -> Result<()> {
        Ok(
            match self.download_impl(architecture, destination_path.as_ref()) {
                Ok(_) => {}
                Err(e) => err!(
                    "Unable to download and verify Alpine base system tarball: {}",
                    e
                ),
            },
        )
    }

    fn download_impl(&self, a: &Arch, p: &Path) -> Result<()> {
        let a = get_architecture(a);
        let version_file = self.download_version_file(a)?;
        let release_info = parse_release_info(&version_file)?;
        let downloaded_size = self.download_tarball(a, &release_info.file, p)?;
        verify_tarball_size(downloaded_size, release_info.size)?;
        verify_checksum(p, &release_info.sha512)?;
        Ok({})
    }

    fn download_version_file(&self, a: &str) -> Result<String> {
        Ok(match self.download_verion_file_impl(a) {
            Ok(v) => v,
            Err(e) => err!("Failed to download version file: {}", e),
        })
    }

    fn download_verion_file_impl(&self, a: &str) -> http::Result<String> {
        let url = format!(
            "https://dl-cdn.alpinelinux.org/alpine/latest-stable/releases/{}/latest-releases.yaml",
            a
        );

        let req = http::GetRequest::new(url)?;
        let response = self.client.get(req)?.as_text()?;

        Ok(response)
    }

    fn download_tarball(&self, a: &str, t: &str, p: &Path) -> Result<u64> {
        let reader = match self.download_tarball_impl(a, t) {
            Ok(r) => r,
            Err(e) => err!("Failed to download tarball file: {}", e),
        };

        Ok(match write_tarball(reader, p) {
            Ok(s) => s,
            Err(e) => err!("Failed to write tarball file: {}", e),
        })
    }

    fn download_tarball_impl(&self, a: &str, t: &str) -> http::Result<impl Read> {
        let url = format!(
            "https://dl-cdn.alpinelinux.org/alpine/latest-stable/releases/{}/{}",
            a, t
        );

        let req = http::GetRequest::new(url)?;
        let response = self.client.get(req)?.as_reader()?;

        Ok(response)
    }
}

#[derive(Deserialize)]
struct VersionFile {
    flavor: String,
    file: String,
    size: u64,
    sha512: String,
}

pub struct Error {
    error: String,
}

impl Error {
    pub fn new<S: AsRef<str>>(error: S) -> Self {
        let error = error.as_ref().to_owned();

        Self { error }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.error)
    }
}

fn parse_release_info(f: &str) -> Result<VersionFile> {
    let vf: Vec<VersionFile> = match serde_yaml::from_str(f) {
        Ok(f) => f,
        Err(e) => err!("Failed to parse version file: {}", e),
    };

    for rel in vf {
        if rel.flavor == "alpine-minirootfs" {
            return Ok(rel);
        }
    }

    err!("Unable to find `alpine-minirootfs` release in a version file")
}

fn verify_checksum(p: &Path, c: &str) -> Result<()> {
    match verify_checksum_impl(p, c) {
        Ok(_) => Ok({}),
        Err(e) => err!("Failed to verify downloaded base system tarball: {}", e),
    }
}

fn verify_checksum_impl(p: &Path, c: &str) -> IoResult<()> {
    let mut hasher = Sha512::new();
    let mut file = File::open(p)?;

    copy(&mut file, &mut hasher)?;
    let actual_checksum = format!("{:x}", hasher.finalize());

    if actual_checksum == c {
        return Ok({});
    }

    Err(std::io::Error::new(
        std::io::ErrorKind::Other,
        format!(
            "SHA-512 checksum doesn't match. Expected `{}`, got `{}`",
            c, actual_checksum
        ),
    ))
}

fn verify_tarball_size(download_size: u64, expected_size: u64) -> Result<()> {
    if download_size == expected_size {
        return Ok({});
    }

    err!(
        "Downloaded Gentoo stage3 tarball size mismatch: expected {}, but got {}",
        expected_size,
        download_size
    )
}

fn write_tarball(r: impl Read, p: &Path) -> IoResult<u64> {
    let mut r = r;
    let mut file = File::create(p)?;
    copy(&mut r, &mut file)
}

fn get_architecture(a: &Arch) -> &'static str {
    match a {
        Arch::AMD64 => "x86_64",
        Arch::X86 => "x86",
        Arch::AARCH64 => "aarch64",
    }
}
