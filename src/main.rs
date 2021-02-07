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
mod replacer;

use clap::{App, Arg};
use regex::Regex;
use replacer::error::{set_debug, CliError};
use replacer::scan_buffer::ScanBuffer;
use replacer::util::{parse_size, read_file, write_file};
use std::fs::File;
use std::io::Read;
use std::io::Write;
use std::iter::FromIterator;

#[derive(Debug, Clone)]
struct Cli {
    inplace: bool,
    pattern: Result<String, CliError>,
    replacement: Result<String, CliError>,
    escape: bool,
    pump_limit: i64,
    verbose: bool,
    files: Option<Vec<String>>,
}

fn regex_validator(val: &str) -> Result<String, CliError> {
    match Regex::new(val) {
        Ok(_) => return Ok(String::from(val)),
        Err(error) => return Err(CliError::from(error)),
    }
}

fn regex_file_validator(val: &str) -> Result<String, CliError> {
    let mut buffer = String::new();
    let file = File::open(val);
    match file {
        Ok(mut file) => match file.read_to_string(&mut buffer) {
            Ok(_) => match regex_validator(buffer.as_str()) {
                Ok(_) => return Ok(String::from(val)),
                Err(error) => return Err(CliError::from(error)),
            },
            Err(error) => return Err(CliError::from(error)),
        },
        Err(error) => return Err(CliError::from(error)),
    }
}

impl Cli {
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
                    .validator(regex_validator)
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
                    .validator(regex_file_validator)
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

    fn new() -> Cli {
        let args = Cli::parse_args();
        let files = args.values_of("files");
        let file_vec: Option<Vec<String>>;
        match files {
            Some(files) => {
                file_vec = Some(
                    files
                        .map(|path| -> String { return String::from(path) })
                        .collect(),
                )
            }
            None => file_vec = None,
        }

        let cli = Cli {
            inplace: args.is_present("inplace"),
            pattern: Cli::get_arg_or_file(
                "pattern",
                args.value_of("pattern"),
                args.value_of("pattern-file-path"),
            ),
            replacement: Cli::get_arg_or_file(
                "replacement",
                args.value_of("replacement"),
                args.value_of("replacement-file-path"),
            ),
            escape: args.is_present("escape"),
            pump_limit: Cli::parse_pump_limit(args.value_of("pump-limit")),
            verbose: args.is_present("verbose"),
            files: file_vec,
        };
        return cli;
    }

    fn get_arg_or_file(
        name: &str,
        arg: Option<&str>,
        path: Option<&str>,
    ) -> Result<String, CliError> {
        match arg {
            Some(arg) => Ok(String::from(arg)),
            None => match path {
                Some(path) => read_file(path),
                None => Err(CliError::from(format!("No {} was supplied", name))),
            },
        }
    }

    fn parse_pump_limit(pump_limit_arg: Option<&str>) -> i64 {
        return pump_limit_arg
            .and_then(|pump_limit| -> Option<i64> {
                return Some(parse_size(pump_limit).unwrap_or(1024 ^ 2));
            })
            .unwrap_or(1024 ^ 2);
    }

    fn escape_pattern(&self, pattern: String) {
        let escaped = regex::escape(pattern.as_str());
        print!("{}", escaped);
    }

    fn process_text(
        &self,
        pattern: &str,
        replacement: &str,
        text: String,
    ) -> Result<String, CliError> {
        return Ok(String::from(
            Regex::new(pattern)?
                .replace_all(text.as_str(), replacement)
                .as_ref(),
        ));
    }

    fn process_file(&self, pattern: &str, replacement: &str, path: &str) -> Result<(), CliError> {
        debug!("Processing: {} => ", path);
        return read_file(path).and_then(|text| -> Result<(), CliError> {
            let result = self.process_text(pattern, replacement, text);
            match result {
                Ok(result) => {
                    debugln!("replaced");
                    match self.inplace {
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
        });
    }

    fn process_files(
        &self,
        pattern: &str,
        replacement: &str,
        files: Vec<String>,
    ) -> Result<(), CliError> {
        for path in files {
            match self.process_file(pattern, replacement, path.as_str()) {
                Ok(_) => (),
                Err(error) => errorln!("{}", error),
            }
        }
        return Ok(());
    }

    fn process_stdin(&self, pattern: &str, replacement: &str) -> Result<(), CliError> {
        debugln!("Reading stdin");
        let mut handle = std::io::stdin();
        let mut buffer = ScanBuffer::new(
            self.pump_limit as usize,
            (self.pump_limit / 2) as usize,
            '\0',
        );

        while buffer.shift(&mut handle) > 0 {
            let text = buffer.process(|b: &Vec<char>| {
                return String::from_iter(b.iter());
            });
            let result = self.process_text(pattern, replacement, text);
            match result {
                Ok(result) => print!("{}", result),
                Err(_) => (),
            }
        }
        return Ok(());
    }

    fn process_pattern(&self, pattern: &str, replacement: &str) -> Result<(), CliError> {
        match self.files.clone() {
            Some(files) => return self.process_files(pattern, replacement, files),
            None => return self.process_stdin(pattern, replacement),
        }
    }

    fn run(&self) -> Result<(), CliError> {
        set_debug(self.verbose);
        match self.pattern.clone() {
            Ok(pattern) => match self.escape {
                true => return Ok(self.escape_pattern(pattern)),
                false => match self.replacement.clone() {
                    Ok(replacement) => {
                        return self.process_pattern(pattern.as_str(), replacement.as_str())
                    }
                    Err(error) => return Err(error),
                },
            },
            Err(error) => return Err(error),
        }
    }
}

fn main() {
    match Cli::new().run() {
        Ok(_) => (),
        Err(error) => {
            errorln!("{}", error);
            std::process::exit(1);
        }
    }
}
