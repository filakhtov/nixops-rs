use std::io::{Error, Result};
use std::path::{Path, PathBuf};
use sys_mount::{Mount, MountFlags, Unmount, UnmountDrop, UnmountFlags};

pub fn mount_kernel_filesystems<P: AsRef<Path>>(path: P) -> Result<Vec<UnmountDrop<Mount>>> {
    let path = path.as_ref();
    let mut mounts = vec![];

    let mut dev_path = path.to_owned();
    dev_path.push("dev");
    match mount_dev(dev_path) {
        Ok(m) => mounts.push(m.into_unmount_drop(UnmountFlags::DETACH)),
        Err(e) => {
            return Err(Error::new(
                e.kind(),
                format!("Failed to mount devtmpfs filesystem: {}", e),
            ))
        }
    }

    let mut devpts_path = path.to_owned();
    devpts_path.push("dev");
    devpts_path.push("pts");
    match mount_devpts(devpts_path) {
        Ok(m) => mounts.push(m.into_unmount_drop(UnmountFlags::DETACH)),
        Err(e) => {
            return Err(Error::new(
                e.kind(),
                format!("Failed to mount devpts filesystem: {}", e),
            ))
        }
    }

    let mut proc_path = path.to_owned();
    proc_path.push("proc");
    match mount_proc(proc_path) {
        Ok(m) => mounts.push(m.into_unmount_drop(UnmountFlags::DETACH)),
        Err(e) => {
            return Err(Error::new(
                e.kind(),
                format!("Failed to mount procfs filesystem: {}", e),
            ))
        }
    }

    let mut sys_path = path.to_owned();
    sys_path.push("sys");
    match mount_sys(sys_path) {
        Ok(m) => mounts.push(m.into_unmount_drop(UnmountFlags::DETACH)),
        Err(e) => {
            return Err(Error::new(
                e.kind(),
                format!("Failed to mount sysfs filesystem: {}", e),
            ))
        }
    }

    Ok(mounts)
}

fn mount_dev(p: PathBuf) -> Result<Mount> {
    Mount::new("devtmpfs", p, "devtmpfs", MountFlags::empty(), None)
}

fn mount_devpts(p: PathBuf) -> Result<Mount> {
    Mount::new("devpts", p, "devpts", MountFlags::empty(), Some("gid=5"))
}

fn mount_proc(p: PathBuf) -> Result<Mount> {
    Mount::new("proc", p, "proc", MountFlags::empty(), None)
}

fn mount_sys(p: PathBuf) -> Result<Mount> {
    Mount::new("sysfs", p, "sysfs", MountFlags::empty(), None)
}
