use std::io::{Error, ErrorKind, Result};
use std::path::Path;

pub fn create_work_dir<P: AsRef<Path>>(path: P) -> Result<()> {
    let wd = path.as_ref();

    if !wd.exists() {
        match std::fs::create_dir_all(&wd) {
            Ok(_) => {}
            Err(e) => {
                return Err(Error::new(
                    e.kind(),
                    format!(
                        "Unable to create working directory `{}`: {}",
                        path_to_string(wd),
                        e
                    ),
                ))
            }
        }
    }

    if !wd.is_dir() {
        return Err(Error::new(
            // TODO: ErrorKind::NotADirectory,
            ErrorKind::Other,
            format!("`{}` is not a directory", path_to_string(wd)),
        ));
    }

    match is_empty_dir(wd) {
        Ok(true) => Ok({}),
        Ok(false) => {
            return Err(Error::new(
                // TODO: ErrorKind::DirectoryNotEmpty,
                ErrorKind::Other,
                format!("Working directory `{}` is not empty", path_to_string(wd)),
            ));
        }
        Err(e) => Err(e),
    }
}

fn is_empty_dir(path: &Path) -> Result<bool> {
    match path.read_dir() {
        Ok(mut dir) => Ok(dir.next().is_none()),
        Err(e) => {
            return Err(Error::new(
                e.kind(),
                format!(
                    "Unable to access directory `{}`: {}",
                    path_to_string(path),
                    e
                ),
            ))
        }
    }
}

pub fn path_to_string<P: AsRef<Path>>(path: P) -> String {
    let path = path.as_ref();

    match path.to_str() {
        Some(p) => p.to_owned(),
        None => path.to_string_lossy().into(),
    }
}
