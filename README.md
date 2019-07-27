# ILC
[![dependency status](https://deps.rs/repo/github/realcundo/ilc/status.svg)](https://deps.rs/repo/github/realcundo/ilc)
[![GitHub license](https://img.shields.io/github/license/realcundo/ilc.svg)](https://github.com/realcundo/ilc/blob/master/LICENSE)

Interactive Line Counter is a command line tool to read input lines from a stream and display most common lines together with their count. It can optionally extract portions of input lines using regular expressions.

It is similar to `wc -l` in unix-like systems except that it outputs counts as they're read. It can be used to count lines in real time in growing log files.

## Examples

```bash
# Read example log file and output most common lines.
ilc example.log
```
```bash
# Follow example.log and keep printing most common line up to date.
tail -f example.log | ilc
```
```bash
# Display most common commands on current Redis server.
# The regexp captures everything except the timestamp and client details.
redis-cli MONITOR | ilc -r "[0-9.]+\s+\[.*\]\s+(.*)"
```

## Usage

```bash
# ilc --help

Tool to interactively display most common matching lines.

The primary use case is to read stdin from a stream, filter the lines using
a regular expression and periodically display top most common lines.

Input files (and stdin) are opened and processed in sequence.

See https://docs.rs/regex/#syntax for regex syntax.

USAGE:
    ilc [OPTIONS] [FILE]...

FLAGS:
    -h, --help       
            Prints help information

    -V, --version    
            Prints version information


OPTIONS:
    -r, --regex <REGEX>    
            Only process lines matching REGEX. Non-matching files are ignored.
            If the REGEX contains a capture group it will be used to process
            the input instead of the whole line.

ARGS:
    <FILE>...    
            Files to process. If none specified stdin is used. To specify stdin
            explicitly pass in "-". Directories are not supported.
```

## Installation

The simplest method is to [install Cargo]((https://doc.rust-lang.org/cargo/getting-started/installation.html)) (Rust package manager) and then:
```bash
cargo install --git https://github.com/realcundo/ilc
```

## Copyright and License

Copyright 2019 Peter Cunderlik

Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file except in compliance with the License.
You may obtain a copy of the License at

    http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software distributed under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied. See the License for the specific language governing permissions and limitations under the License.