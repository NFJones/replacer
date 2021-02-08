/*
*   Copyright (c) 2021 Neil F Jones
*   All rights reserved.

*   Permission is hereby granted, free of charge, to any person obtaining a copy
*   of this software and associated documentation files (the "Software"), to deal
*   in the Software without restriction, including without limitation the rights
*   to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
*   copies of the Software, and to permit persons to whom the Software is
*   furnished to do so, subject to the following conditions:

*   The above copyright notice and this permission notice shall be included in all
*   copies or substantial portions of the Software.

*   THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
*   IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
*   FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
*   AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
*   LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
*   OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
*   SOFTWARE.
*/
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
    ($($arg:tt)*) => ({std::io::stderr().write_all(format!($($arg)*).as_bytes()).ok();})
}

#[macro_export]
macro_rules! errorln {
    () => (crate::error!("\n"));
    ($($arg:tt)*) => (crate::error!("{}\n", format!($($arg)*)));
}

#[macro_export]
macro_rules! debug {
    ($($arg:tt)*) => (if super::error::get_debug() { crate::error!($($arg)*);})
}

#[macro_export]
macro_rules! debugln {
    () => (crate::debug!("\n"));
    ($($arg:tt)*) => (crate::debug!("{}\n", format!($($arg)*)));
}
