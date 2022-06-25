pub enum Arch {
    AMD64,
    X86,
    AARCH64,
}

impl Arch {
    pub fn new<S: AsRef<str>>(architecture: S) -> Result<Self, ()> {
        Ok(match architecture.as_ref() {
            "x86" => Self::X86,
            "x86_64" => Self::AMD64,
            "aarch64" => Self::AARCH64,
            _ => return Err({}),
        })
    }
}
