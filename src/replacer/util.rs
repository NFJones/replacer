use super::error::CliError;
use std::fs::File;
use std::io::Read;
use std::io::Write;

pub fn read_file(path: &str) -> Result<String, CliError> {
    let mut buf = String::new();
    match File::open(&path)?.read_to_string(&mut buf) {
        Ok(_) => Ok(buf),
        Err(error) => Err(CliError::from(error)),
    }
}

pub fn write_file(path: &str, content: String) -> Result<(), CliError> {
    match std::fs::OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(&path)?
        .write_all(content.as_bytes())
    {
        Ok(_) => Ok(()),
        Err(error) => Err(CliError::from(error)),
    }
}
