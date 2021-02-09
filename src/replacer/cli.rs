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
use super::util::*;
use super::validators::*;
use crate::{debug, debugln, errorln};
use clap::Clap;
use regex::Regex;
use std::io::Read;
use std::io::Write;

#[clap(
    name = "rp",
    version = "0.2.0",
    author = "Neil F Jones",
    about = "A multiline regex find/replace utility."
)]
#[derive(Debug, Clone, Clap)]
struct Opts {
    #[clap(
        short('i'),
        long("inplace"),
        takes_value(false),
        about("The regex pattern to match.")
    )]
    inplace: bool,
    #[clap(
        short('p'),
        long("pattern"),
        takes_value(true),
        validator(validate_regex),
        conflicts_with("pattern-file"),
        about("Write to file instead of stdout.")
    )]
    pattern: Option<String>,
    #[clap(
        short('r'),
        long("replacement"),
        takes_value(true),
        conflicts_with("replacement-file"),
        about("The replacement text to write. Supports groups (${1}, ${named_group}, etc.)")
    )]
    replacement: Option<String>,
    #[clap(
        short('P'),
        long("pattern-file"),
        takes_value(true),
        validator(validate_regex_file),
        conflicts_with("pattern"),
        about("The file to read the regex pattern from.")
    )]
    pattern_file: Option<String>,
    #[clap(
        short('R'),
        long("replacement-file"),
        takes_value(true),
        conflicts_with("replacement"),
        about("The file to read the replacement text from.")
    )]
    replacement_file: Option<String>,
    #[clap(
        short('e'),
        long("escape"),
        takes_value(false),
        about("Print the pattern with regex characters escaped.")
    )]
    escape: bool,
    #[clap(
        short('v'),
        long("verbose"),
        takes_value(false),
        about("Print verbose output to stderr.")
    )]
    verbose: bool,
    #[clap(multiple(true), about("Print verbose output to stderr."))]
    files: Vec<String>,
}

#[derive(Debug, Clone)]
struct ParsedOpts {
    pattern: String,
    replacement: String,
}

#[derive(Debug, Clone)]
pub struct Cli {
    opts: Opts,
    parsed_opts: ParsedOpts,
}

impl Cli {
    pub fn new() -> Cli {
        let opts = Opts::parse();
        let parsed_opts = ParsedOpts {
            pattern: Cli::get_arg_or_file(opts.pattern.clone(), opts.pattern_file.clone()),
            replacement: Cli::get_arg_or_file(
                opts.replacement.clone(),
                opts.replacement_file.clone(),
            ),
        };
        return Cli { opts, parsed_opts };
    }

    fn get_arg_or_file(arg: Option<String>, path: Option<String>) -> String {
        match arg {
            Some(arg) => return arg,
            None => match path {
                Some(path) => return read_file(path.as_str()).unwrap(),
                None => return String::new(),
            },
        }
    }

    fn escape_pattern(&self) {
        let escaped = regex::escape(self.parsed_opts.pattern.as_str());
        print!("{}", escaped);
    }

    fn process_text(&self, text: String) -> Result<String, CliError> {
        return Ok(String::from(
            Regex::new(self.parsed_opts.pattern.as_str())?
                .replace_all(text.as_str(), self.parsed_opts.replacement.as_str())
                .as_ref(),
        ));
    }

    fn process_file(&self, path: &str) -> Result<(), CliError> {
        debug!("Processing: {} => ", path);
        return read_file(path).and_then(|text| -> Result<(), CliError> {
            let result = self.process_text(text);
            match result {
                Ok(result) => {
                    debugln!("replaced");
                    match self.opts.inplace {
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

    fn process_files(&self) -> Result<(), CliError> {
        for path in self.opts.files.iter() {
            match self.process_file(path.as_str()) {
                Ok(_) => (),
                Err(error) => errorln!("{}", error),
            }
        }
        return Ok(());
    }

    fn process_stdin(&self) -> Result<(), CliError> {
        debugln!("Reading stdin");
        let mut text = String::new();

        match std::io::stdin().read_to_string(&mut text) {
            Ok(_) => {
                let result = self.process_text(text.clone());
                match result {
                    Ok(result) => print!("{}", result),
                    Err(_) => (),
                }
            }
            Err(error) => return Err(CliError::from(error)),
        }
        return Ok(());
    }

    fn process_pattern(&self) -> Result<(), CliError> {
        match self.opts.files.len() > 0 {
            true => return self.process_files(),
            false => return self.process_stdin(),
        }
    }

    pub fn run(&self) -> Result<(), CliError> {
        set_debug(self.opts.verbose);
        match self.opts.escape {
            true => return Ok(self.escape_pattern()),
            false => return self.process_pattern(),
        }
    }
}
