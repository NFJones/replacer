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
mod error;

use clap::{App, Arg};
use error::{set_debug, CliError};
use regex::Regex;
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::io::Write;
use std::ops::Index;

fn parse_args() -> clap::ArgMatches {
    return App::new("rp")
        .author("Neil Jones")
        .about("A multiline regex find/replace utility.")
        .version("0.1.0")
        .args(&[
            Arg::new("inplace")
                .short('i')
                .long("inplace")
                .takes_value(false)
                .about("Write to file instead of stdout."),
            Arg::new("pattern")
                .short('p')
                .long("pattern")
                .takes_value(true)
                .about("The regex pattern to match."),
            Arg::new("replacement")
                .short('r')
                .long("replacement")
                .takes_value(true)
                .about(
                    "The replacement text to write. Supports groups (${1}, ${named_group}, etc.)",
                ),
            Arg::new("pattern-file-path")
                .short('P')
                .long("pattern-file")
                .takes_value(true)
                .about("The file to read the regex pattern from."),
            Arg::new("replacement-file-path")
                .short('R')
                .long("replacement-file")
                .takes_value(true)
                .about("The file to read the replacement text from."),
            Arg::new("escape")
                .short('e')
                .long("escape-pattern")
                .takes_value(false)
                .about("Print the pattern with regex characters escaped."),
            Arg::new("pump-limit")
                .short('l')
                .long("pump-limit")
                .takes_value(true)
                .default_value("1MiB")
                .about("The internal buffer size when making streaming replacements."),
            Arg::new("verbose")
                .short('v')
                .long("verbose")
                .takes_value(false)
                .about("Print verbose output to stderr."),
            Arg::new("files").multiple(true),
        ])
        .get_matches();
}

fn read_file(path: &str) -> Result<String, CliError> {
    let mut buf = String::new();
    match File::open(&path)?.read_to_string(&mut buf) {
        Ok(_) => Ok(buf),
        Err(error) => Err(CliError::from(error)),
    }
}

fn get_arg_or_file(name: &str, arg: Option<&str>, path: Option<&str>) -> Result<String, CliError> {
    match arg {
        Some(arg) => Ok(String::from(arg)),
        None => match path {
            Some(path) => read_file(path),
            None => Err(CliError::from(format!("No {} was supplied", name))),
        },
    }
}

fn write_file(path: &str, content: String) -> Result<(), CliError> {
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

fn process_text(pattern: String, replacement: String, text: String) -> Result<String, CliError> {
    return Ok(String::from(
        Regex::new(pattern.clone().as_str())?
            .replace_all(text.as_str(), replacement.as_str())
            .as_ref(),
    ));
}

fn process_stdin(pattern: String, replacement: String, pump_limit: i64) -> Result<(), CliError> {
    debugln!("Reading stdin");
    let mut text = String::new();
    let _pump_limit = pump_limit;
    match std::io::stdin().read_to_string(&mut text) {
        Ok(_) => {
            let result = process_text(pattern, replacement, text);
            match result {
                Ok(result) => print!("{}", result),
                Err(error) => return Err(error),
            }
        }
        Err(error) => return Err(CliError::from(error)),
    }
    return Ok(());
}

fn process_files(
    pattern: String,
    replacement: String,
    files: clap::Values,
    inplace: bool,
) -> Result<(), CliError> {
    for path in files {
        debug!("Processing: {} => ", path);
        read_file(path)
            .and_then(|text| -> Result<(), CliError> {
                let result = process_text(pattern.clone(), replacement.clone(), text);
                match result {
                    Ok(result) => {
                        debugln!("replaced");
                        match inplace {
                            true => return write_file(path, result),
                            false => {
                                print!("{}", result);
                                return Ok(());
                            }
                        }
                    }
                    Err(error) => {
                        debugln!("skipped");
                        return Err(error);
                    }
                }
            })
            .or_else(|error| -> Result<(), CliError> {
                debugln!("failed: ({})", error);
                return Err(error);
            })
            .ok();
    }
    return Ok(());
}

fn process_pattern(
    pattern: String,
    replacement: String,
    files: Option<clap::Values>,
    inplace: bool,
    pump_limit: i64,
) -> Result<(), CliError> {
    match files {
        Some(files) => return process_files(pattern, replacement, files, inplace),
        None => return process_stdin(pattern, replacement, pump_limit),
    }
}

fn escape_pattern(pattern: String) {
    let escaped = regex::escape(pattern.as_str());
    print!("{}", escaped);
}

fn parse_size(size_str: &str) -> Result<i64, CliError> {
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

fn parse_pump_limit(pump_limit_arg: Option<&str>) -> i64 {
    return pump_limit_arg
        .and_then(|pump_limit| -> Option<i64> {
            return Some(parse_size(pump_limit).unwrap_or(1024 ^ 2));
        })
        .unwrap_or(1024 ^ 2);
}

fn main() {
    let args = parse_args();
    set_debug(args.is_present("verbose"));

    let escape = args.is_present("escape");
    let inplace = args.is_present("inplace");
    let pattern = get_arg_or_file(
        "pattern",
        args.value_of("pattern"),
        args.value_of("pattern-file-path"),
    );
    let replacement = get_arg_or_file(
        "replacement",
        args.value_of("replacement"),
        args.value_of("replacement-file-path"),
    );
    let files: Option<clap::Values> = args.values_of("files");
    let pump_limit = parse_pump_limit(args.value_of("pump-limit"));

    pattern
        .and_then(|pattern| -> Result<(), CliError> {
            match escape {
                true => return Ok(escape_pattern(pattern.clone())),
                false => {
                    return replacement.and_then(|replacement| -> Result<(), CliError> {
                        return process_pattern(pattern, replacement, files, inplace, pump_limit);
                    })
                }
            }
        })
        .or_else(|error| -> Result<(), ()> {
            errorln!("\n{}", error);
            std::process::exit(1);
        })
        .ok();
}
