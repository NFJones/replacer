# Replacer

replacer is a simple regex find and replace utility. It supports multiline patterns and group referencing.

## Usage

```
>$ rp --help
rp 0.1.0
Neil Jones
A multiline regex find/replace utility.

USAGE:
    rp [FLAGS] [OPTIONS] [files]...

ARGS:
    <files>...

FLAGS:
    -e, --escape-pattern    Print the pattern with regex characters escaped.
    -h, --help              Prints help information
    -i, --inplace           Write to file instead of stdout.
    -V, --version           Prints version information

OPTIONS:
    -p, --pattern <pattern>                           The regex pattern to match.
    -P, --pattern-file <pattern-file-path>            The file to read the regex pattern from.
    -r, --replacement <replacement>
            The replacement text to write. Supports groups (${1}, ${named_group}, etc.)

    -R, --replacement-file <replacement-file-path>    The file to read the replacement text from.
```
