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


use clap::{App, Arg};
use std::fs::File;
use std::io::Read;
use std::io::Write;

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
                .about("The replacement text to write. Supports groups (${1}, ${named_group}, etc.)"),
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
            Arg::new("files").multiple(true),
        ])
        .get_matches();
}

fn read_file(path: &str) -> Result<String, String> {
    fn file_error(path: &str, error: std::io::Error) -> String {
        return format!("File could not be read: {}, {}", path, error);
    }

    let mut buf = String::new();
    match File::open(&path) {
        Ok(mut file) => match file.read_to_string(&mut buf) {
            Ok(_) => (),
            Err(error) => return Err(file_error(path, error)),
        },
        Err(error) => return Err(file_error(path, error)),
    }
    return Ok(buf);
}

fn get_arg_or_file(
    name: &str,
    pattern: Option<&str>,
    path: Option<&str>,
) -> Result<String, String> {
    match pattern {
        Some(p) => return Ok(String::from(p)),
        None => match path {
            Some(path) => return read_file(path),
            None => return Err(format!("No {} was supplied", name)),
        },
    }
}

fn write_file(path: &str, content: String) -> Result<(), String> {
    match std::fs::OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(&path)
    {
        Ok(ref mut file) => match file.write_all(content.as_bytes()) {
            Ok(_) => return Ok(()),
            Err(error) => return Err(format!("Failed to write to file: {}, {}", path, error)),
        },
        Err(error) => return Err(format!("Failed to open file: {}, {}", path, error)),
    }
}

fn process_text(pattern: String, replacement: String, text: String) -> Result<String, String> {
    let re = regex::Regex::new(pattern.clone().as_str());
    match re {
        Ok(re) => {
            return Ok(String::from(
                re.replace_all(text.as_str(), replacement.as_str()),
            ));
        }
        Err(error) => return Err(format!("Failed to parse regex: {}", error)),
    }
}

fn process_pattern(
    pattern: String,
    replacement: String,
    files: Option<clap::Values>,
    inplace: bool,
) -> Result<String, String> {
    match files {
        Some(files) => {
            for path in files {
                let text = read_file(path);
                match text {
                    Ok(text) => {
                        let result = process_text(pattern.clone(), replacement.clone(), text);
                        match result {
                            Ok(result) => match inplace {
                                true => match write_file(path, result) {
                                    Ok(_) => (),
                                    Err(error) => return Err(error),
                                },
                                false => print!("{}", result),
                            },
                            Err(error) => return Err(error),
                        }
                    }
                    Err(error) => return Err(error),
                }
            }
        }
        None => {
            let mut text = String::new();
            match std::io::stdin().read_to_string(&mut text) {
                Ok(_) => {
                    let result = process_text(pattern, replacement, text);
                    match result {
                        Ok(result) => print!("{}", result),
                        Err(error) => return Err(error),
                    }
                }
                Err(error) => return Err(format!("Failed to read stdin: {}", error)),
            }
        }
    }
    return Ok(String::new());
}

fn escape_pattern(pattern: String) {
    let escaped = regex::escape(pattern.as_str());
    print!("{}", escaped);
}

fn main() {
    let mut error_occurred = false;
    let args = parse_args();
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

    match pattern {
        Ok(p) => match escape {
            true => escape_pattern(p),
            false => match replacement {
                Ok(r) => {
                    let result = process_pattern(p, r, files, inplace);
                    match result {
                        Ok(_) => (),
                        Err(error) => {
                            println!("{}", error);
                            error_occurred = true;
                        }
                    }
                }
                Err(error) => {
                    println!("{}", error);
                    error_occurred = true;
                }
            },
        },
        Err(error) => {
            println!("{}", error);
            error_occurred = true;
        }
    }

    if error_occurred {
        std::process::exit(1);
    }
}
