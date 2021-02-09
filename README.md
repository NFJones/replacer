# Replacer

replacer is a simple regex find and replace utility. It supports multiline patterns and group referencing.

## Usage

```
>$ rp --help
rp 0.2.0
Neil F Jones
A multiline regex find/replace utility.

USAGE:
    rp [FLAGS] [OPTIONS] [files]...

ARGS:
    <files>...    Print verbose output to stderr.

FLAGS:
    -e, --escape     Print the pattern with regex characters escaped.
    -h, --help       Prints help information
    -i, --inplace    The regex pattern to match.
    -v, --verbose    Print verbose output to stderr.
    -V, --version    Prints version information

OPTIONS:
    -p, --pattern <pattern>                      Write to file instead of stdout.
    -P, --pattern-file <pattern-file>            The file to read the regex pattern from.
    -r, --replacement <replacement>
            The replacement text to write. Supports groups (${1}, ${named_group}, etc.)

    -R, --replacement-file <replacement-file>    The file to read the replacement text from.
```
