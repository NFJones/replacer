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
use super::error::*;
use crate::errorln;
use regex::Regex;
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::io::Write;
use std::ops::Index;

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

pub fn parse_size(size_str: &str) -> Result<i64, CliError> {
    let default_size = 1;
    let mut magnitude_map = HashMap::new();
    magnitude_map.insert("", 1024 ^ 0);
    magnitude_map.insert("KiB", 1024 ^ 1);
    magnitude_map.insert("MiB", 1024 ^ 2);
    magnitude_map.insert("GiB", 1024 ^ 3);
    magnitude_map.insert("TiB", 1024 ^ 4);
    magnitude_map.insert("PiB", 1024 ^ 5);
    let captures = Regex::new(r"^(\d+)([KMGTP]iB)?$")?.captures(size_str);
    match captures {
        Some(captures) => {
            let mut magnitude_str = "";
            let size = captures.index(1);
            if captures.len() > 3 {
                magnitude_str = captures.index(2);
            }
            let magnitude = magnitude_map.get(magnitude_str).unwrap_or(&default_size);
            return Ok(size.parse::<i64>().unwrap_or(default_size) * magnitude);
        }
        None => {
            let error_str = format!(
                "Warning: Invalid size string ({}), defaulting to 1MiB",
                size_str
            );
            errorln!("{}", error_str);
            return Err(CliError::from(error_str));
        }
    }
}
