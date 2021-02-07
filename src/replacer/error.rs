use std::sync::atomic::{AtomicBool, Ordering};

static DEBUG: AtomicBool = AtomicBool::new(false);

pub fn set_debug(debug: bool) {
    DEBUG.store(debug, Ordering::SeqCst);
}

pub fn get_debug() -> bool {
    return DEBUG.load(Ordering::SeqCst);
}

#[derive(Debug, Clone)]
pub struct CliError {
    msg: String,
}

impl From<std::io::Error> for CliError {
    fn from(error: std::io::Error) -> CliError {
        let msg = String::from(format!("{}", error));
        return CliError { msg };
    }
}

impl From<String> for CliError {
    fn from(error: String) -> CliError {
        let msg = String::from(format!("{}", error));
        return CliError { msg };
    }
}

impl From<&str> for CliError {
    fn from(error: &str) -> CliError {
        let msg = String::from(format!("{}", error));
        return CliError { msg };
    }
}

impl From<regex::Error> for CliError {
    fn from(error: regex::Error) -> CliError {
        let msg = String::from(format!("{}", error));
        return CliError { msg };
    }
}

impl std::fmt::Display for CliError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.msg)
    }
}

#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => (std::io::stderr().write_all(format!($($arg)*).as_bytes()).ok())
}

#[macro_export]
macro_rules! errorln {
    () => (crate::error!("\n"));
    ($($arg:tt)*) => (crate::error!("{}\n", format!($($arg)*)));
}

#[macro_export]
macro_rules! debug {
    ($($arg:tt)*) => (if replacer::error::get_debug() { crate::error!($($arg)*);})
}

#[macro_export]
macro_rules! debugln {
    () => (crate::debug!("\n"));
    ($($arg:tt)*) => (crate::debug!("{}\n", format!($($arg)*)));
}
