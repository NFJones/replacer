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
    ($($arg:tt)*) => (std::io::stderr().write_all(format!($($arg)*)))
}

#[macro_export]
macro_rules! errorln {
    () => (print!("\n"));
    ($($arg:tt)*) => ({
        std::io::stderr().write_all(format!("{}\n", format!($($arg)*)).as_bytes()).expect("Failed to write to stderr. Aborting.");
    })
}
