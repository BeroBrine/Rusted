use std::{
    fs::{File, OpenOptions},
    io::Write,
    sync::OnceLock,
};

pub struct Logger {
    file: File,
}

pub static LOGGER: OnceLock<Logger> = OnceLock::new();

impl Logger {
    pub fn new(path: &str) -> anyhow::Result<Self> {
        let file = OpenOptions::new()
            .append(true)
            .write(true)
            .create(true)
            .open(path)?;

        Ok(Self { file })
    }
    pub fn log(&self, message: String) -> anyhow::Result<()> {
        (&self.file).write(message.as_bytes())?;
        Ok(())
    }
}

#[macro_export]
macro_rules! log {
    ($($args:tt)*) => {
        let _ = $crate::logger::_logger::LOGGER
            .get_or_init(|| $crate::logger::_logger::Logger::new("rusted.log").unwrap()).log(format!($($args)*));
    };
}
