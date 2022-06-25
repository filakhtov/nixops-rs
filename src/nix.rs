use crate::chroot;
use crate::fs::path_to_string;
use std::io::prelude::*;
use std::path::Path;

type Result<T> = core::result::Result<T, Error>;

pub struct Error {
    error: String,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.error)
    }
}

macro_rules! err {
    ($($args:expr),+) => {
        return Err(Error { error: format!($($args),+) })
    };
}

pub fn install_nix<R: AsRef<Path>>(chroot: R) -> Result<()> {
    let chroot = chroot.as_ref();

    configure_repositories(chroot)?;
    update_repositories(chroot)?;
    install_nix_package(chroot)?;
    configure_nix(chroot)?;
    update_channels(chroot)?;

    Ok({})
}

fn configure_repositories(chroot: &Path) -> Result<()> {
    let mut repo_path = chroot.to_owned();
    repo_path.push("etc");
    repo_path.push("apk");
    repo_path.push("repositories");
    let repositories = format!(
        "https://dl-cdn.alpinelinux.org/alpine/edge/main/\n\
        https://dl-cdn.alpinelinux.org/alpine/edge/community/\n\
        https://dl-cdn.alpinelinux.org/alpine/edge/testing/\n"
    );
    match std::fs::write(&repo_path, repositories) {
        Ok(_) => {}
        Err(e) => err!(
            "Failed to update `{}` file: {}",
            path_to_string(repo_path),
            e
        ),
    }

    Ok({})
}

fn update_repositories(chroot: &Path) -> Result<()> {
    match chroot::execute(chroot, ["apk", "update"]) {
        Ok(_) => Ok({}),
        Err(e) => err!("Failed to install the `nix` package:\n{}", e),
    }
}

fn install_nix_package(chroot: &Path) -> Result<()> {
    match chroot::execute(chroot, ["apk", "add", "bash", "tar", "xz", "nix"]) {
        Ok(_) => Ok({}),
        Err(e) => err!("Failed to install the `nix` package:\n{}", e),
    }
}

fn configure_nix(chroot: &Path) -> Result<()> {
    let mut nix_conf_path = chroot.to_owned();
    nix_conf_path.push("etc");
    nix_conf_path.push("nix");
    nix_conf_path.push("nix.conf");

    let mut config = match std::fs::OpenOptions::new()
        .write(true)
        .append(true)
        .open(&nix_conf_path)
    {
        Ok(f) => f,
        Err(e) => err!(
            "Unable to read Nix configuration file `{}`: {}",
            path_to_string(&nix_conf_path),
            e
        ),
    };

    match writeln!(config, "sandbox = false\n") {
        Ok(_) => Ok({}),
        Err(e) => err!(
            "Unable to update Nix configuration file `{}`: {}",
            path_to_string(&nix_conf_path),
            e
        ),
    }
}

fn update_channels(chroot: &Path) -> Result<()> {
    if let Err(e) = chroot::execute(
        &chroot,
        [
            "nix-channel",
            "--add",
            "https://nixos.org/channels/nixpkgs-unstable",
        ],
    ) {
        err!("Failed to subscribe to nixpkgs channel:\n{}", e);
    }

    let mut profile_dir = chroot.to_owned();
    profile_dir.push("nix");
    profile_dir.push("var");
    profile_dir.push("nix");
    profile_dir.push("profiles");
    profile_dir.push("default");
    if let Err(e) = std::fs::remove_dir(&profile_dir) {
        err!(
            "Failed to remove default profile directory `{}`: {}",
            path_to_string(&profile_dir),
            e
        )
    }

    match chroot::execute(&chroot, ["nix-channel", "--update"]) {
        Ok(_) => Ok({}),
        Err(e) => err!("Failed to update Nix channels:\n{}", e),
    }
}

pub fn install_nixos_generators<R: AsRef<Path>>(chroot: R) -> Result<()> {
    match chroot::execute(&chroot, ["nix-env", "-iA", "nixpkgs.nixos-generators"]) {
        Ok(_) => Ok({}),
        Err(e) => err!("Failed to install `nixos-generators`:\n{}", e),
    }
}

// TODO: actually figure this out
pub fn generate_lxc_image<P: AsRef<Path>>(chroot: P) -> Result<()> {
    match chroot::execute(&chroot, ["nixos-generate", "-f", "lxc", "-c", "/lxc.nix"]) {
        Ok(_) => Ok({}),
        Err(e) => err!("Failed to install `nixos-generators`:\n{}", e),
    }
}
