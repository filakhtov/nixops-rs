use std::ffi::{OsStr, OsString};
use std::io::{Error, ErrorKind, Result};
use std::path::Path;
use std::process::Command;

pub fn execute<P: AsRef<Path>, A: AsRef<OsStr>, I: IntoIterator<Item = A>>(
    chroot: P,
    args: I,
) -> Result<()> {
    let mut args: Vec<OsString> = args.into_iter().map(|a| a.as_ref().to_owned()).collect();
    let mut args_vec: Vec<OsString> = Vec::with_capacity(args.len() + 3);
    args_vec.push(chroot.as_ref().as_os_str().to_owned());
    args_vec.push("/usr/bin/env".into());
    args_vec.push("TMPDIR=/tmp".into());
    args_vec.append(&mut args);

    let result = match Command::new("chroot").args(&args_vec).output() {
        Ok(o) => o,
        Err(e) => return Err(Error::new(e.kind(), format!("{}", e))),
    };

    if !result.status.success() {
        return Err(Error::new(
            ErrorKind::Other,
            format!(
                "= stdout:\n{}\n= stderr:\n{}",
                String::from_utf8_lossy(&result.stdout),
                String::from_utf8_lossy(&result.stderr)
            ),
        ));
    }

    eprint!(" chroot");
    for i in args_vec {
        eprint!(" {}", i.to_string_lossy());
    }
    eprintln!("");

    eprintln!(
        "= stdout:\n{}\n= stderr:\n{}",
        String::from_utf8_lossy(&result.stdout),
        String::from_utf8_lossy(&result.stderr)
    );

    Ok({})
}
