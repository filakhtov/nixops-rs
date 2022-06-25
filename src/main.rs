mod alpine;
mod app;
mod arch;
mod chroot;
mod extractor;
mod fs;
mod http;
mod mount;
mod nix;

macro_rules! abort {
    ($($msg:expr),+) => {
        abort(format!($($msg),+))
    };
}

fn abort<S: AsRef<str>>(message: S) -> ! {
    eprintln!("{}", message.as_ref());

    std::process::exit(1);
}

fn main() {
    let app = match crate::app::init_app() {
        Ok(app) => app,
        Err(e) => abort!("Failed to initialize the application: {}", e),
    };
    match app.build() {
        Ok(_) => {}
        Err(e) => abort!("{}", e),
    };
}
