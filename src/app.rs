use crate::{
    alpine::BaseSystemDownloader,
    arch::Arch,
    extractor::extract,
    fs::{create_work_dir, path_to_string},
    http,
    mount::mount_kernel_filesystems,
    nix,
};
use std::path::Path;

macro_rules! err {
    ($($msg:expr),+) => {
        return Err(Error::new(format!($($msg),+)))
    };
}

type Result<T> = core::result::Result<T, Error>;

pub struct Error {
    error: String,
}

impl From<http::Error> for Error {
    fn from(e: http::Error) -> Self {
        Self {
            error: format!("{}", e),
        }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.error)
    }
}

impl Error {
    fn new<E: AsRef<str>>(error: E) -> Self {
        Self {
            error: error.as_ref().into(),
        }
    }
}

pub fn init_app() -> Result<App> {
    let client_builder = http::Client::builder()
        .request_timeout(None)
        .connect_timeout(None);
    let bsd = BaseSystemDownloader::new(client_builder.build()?);

    let app = App::new(bsd)?;

    Ok(app)
}

pub struct App {
    arch: Arch,
    bsd: BaseSystemDownloader,
}

impl Drop for App {
    fn drop(&mut self) {
        // remove destination directory
        println!("TODO: implement crate::app::App::drop()");
    }
}

impl App {
    pub fn new(base_system_downloader: BaseSystemDownloader) -> Result<Self> {
        check_platform()?;
        let arch = get_architecture()?;

        Ok(Self {
            arch: arch,
            bsd: base_system_downloader,
        })
    }

    pub fn build(&self) -> Result<()> {
        // TODO: parse arguments
        // TODO: parse config

        let wd = std::path::PathBuf::from("./workdir/");
        println!("Creating working directory...");
        match create_work_dir(&wd) {
            Ok(_) => println!("... OK: `{}` was successfully created", path_to_string(&wd)),
            Err(e) => err!("... ERROR: {}", e),
        };

        let mut tarball_path = wd.clone();
        tarball_path.push("base.txz");
        println!("Downloading base system tarball...");
        match self.bsd.download(&self.arch, &tarball_path) {
            Ok(_) => println!(
                "... OK: `{}` was successfully downloaded",
                path_to_string(&tarball_path)
            ),
            Err(e) => err!("... ERROR: {}", e),
        };

        println!("Extracting base system tarball...");
        match extract(&tarball_path) {
            Ok(_) => println!(
                "... OK: `{}` was successfully extracted",
                path_to_string(&tarball_path)
            ),
            Err(e) => err!("... ERROR: {}", e),
        }

        println!("Configure DNS resolution in the chroot environment...");
        match fix_resolv_conf(&wd) {
            Ok(_) => {
                println!("... OK: successfully created resolv.conf");
            }
            Err(e) => {
                eprintln!("... ERROR: {}", e);
            }
        }

        println!("Mounting Virtual Kernel File Systems...");
        let _mounts = match mount_kernel_filesystems(&wd) {
            Ok(mts) => {
                println!("... OK: devtmpfs, procfs, sysfs were successfully mounted");

                mts
            }
            Err(e) => err!("... ERROR: {}", e),
        };

        println!("Installing Nix package manager...");
        match nix::install_nix(&wd) {
            Ok(_) => {
                println!("... OK: Nix package manager was succefully installed");
            }
            Err(e) => err!("... ERROR: {}", e),
        }

        println!("Installing the `nixos-generators` package using Nix...");
        match nix::install_nixos_generators(&wd) {
            Ok(_) => {
                println!("... OK: `nixos-generators` package was successfully installed");
            }
            Err(e) => err!("... ERROR: {}", e),
        }

        // TODO:
        // from chroot:
        // - Run `nixos-generate -f lxc`
        // - Copy resulting tarball image to the destination
        // run clean-up:
        // - Unmount
        // - remove destination directory
        // upload tarball image to Proxmox
        // create container using uploaded image
        // - figure out how to specify various parameters
        // figure out how to tweak LXC image to contain necessary tools

        Ok({})
    }
}

fn check_platform() -> Result<()> {
    if let "linux" = std::env::consts::OS {
        return Ok({});
    }

    err!(
        "Unsupported platform. Only `linux` is supported, but `{}` is detected.",
        std::env::consts::OS
    )
}

fn get_architecture() -> Result<Arch> {
    match Arch::new(std::env::consts::ARCH) {
        Ok(a) => Ok(a),
        _ => err!(
            "Unsupported architecture. Only `x86`, `x86_64` and `aarch64` are supported, but `{}` is detected.",
            std::env::consts::ARCH
        ),
    }
}

fn fix_resolv_conf(p: &Path) -> Result<()> {
    let mut resolv_conf_path = p.to_owned();
    resolv_conf_path.push("etc");
    resolv_conf_path.push("resolv.conf");

    match std::fs::write(&resolv_conf_path, "nameserver 8.8.8.8") {
        Ok(_) => Ok({}),
        Err(e) => err!(
            "Unable to create `{}` file: {}",
            path_to_string(&resolv_conf_path),
            e
        ),
    }
}
