# Tivilsta - A different whitelisting mechanism for blocklist maintainers

Tivilsta aims to provide a different and in our view better whitelisting mechanism
for blocklist maintainers.

# Table of Content

- [Installation](#installation)
- [The Format](#the-format)
  - [Introduction](#introduction)
  - [The flags](#the-flags)
    - [No Flag: The purest rule](#no-flag-the-purest-rule)
    - [`ALL `: The "ends-with" rule](#all--the-ends-with-rule)
    - [`REG `: The regular expression rule](#reg--the-regular-expression-rule)
    - [`RZD `: The broad and powerful rule](#rzd--the-broad-and-powerful-rule)
- [Usage & Examples](#usage--examples)
  - [Library](#library)
  - [CLI](#cli)
    - [Overview](#overview)
    - [Help Output](#help-output)
    - [Simple whitelisting example](#simple-whitelisting-example)
- [License](#license)

# Installation

You can install the tivilsta CLI or library through [crates.io](https://crates.io/crates/tivilsta).

```shell
$ cargo install tivilsta

$ tivilsta -V  ## Assuming that it is in your ${PATH}
```

# The Format

## Introduction

In a world where blocklists and whitelist lists are getting bigger and bigger,
the whitelisting mechanism we all use is still the same: list all whitelisted
domains and use some kind of shell magic to processed the whitelisting.

What if we want more ? That's what Tivilsta tries to provide: A better way of
writing whitelist list.

With Tivilsta you still have 1 domain per line but you also get some nice features
like for example regular expression _(regex)_. In fact, the Tivilsta project
the same set of of "pure" rule that you know but also some flags like `ALL `,
`REG ` or `RDZ ` to fulfill many possible use cases that list maintainer may
need during the whitelisting process.

## The flags

Tivilsta provides a set of flags to make whitelist maintenance easier.

### No Flag: The purest rule

This is the purest of all rules. It is what we all know an cherish. The single
line without any flag.

```
example.org
```

In this example, any subject of your source file that literally matches `example.org`
will be whitelisted.

### `ALL `: The "ends-with" rule

Sometime when working with highly volatile dataset, you may want to whitelist
every subjects that ends with for example `gov.uk`.

With Tivilsta you can do that through the `ALL ` flag.

```
ALL .gov.uk
```

In this example, any subject of your source file that ends with `.gov.uk` -
`gov.uk` included - will be whitelisted.

### `REG `: The regular expression rule

You are a fan of regex ? We are too! When working with highly volatile dataset,
we want to simply use a regular expression (short regex) to do the task.

With Tivilsta you can do that through the `REG ` flag.

```
REG ^(?!.*\.?(watchdog\.ohio|dap\.digitalgov|stats\.ssa|adgallery\.whitehousedrugpolicy)).*\.gov$
```

In this example, any subject of your source file that ends with `.gov` will be
whitelisted except the following:

- `watchdog.ohio.gov`
- `dap.digitalgov.gov`
- `stats.ssa.gov`
- `adgallery.whiteshousedrugpolicy.gov`

### `RZD `: The broad and powerful rule

Have you ever wondered if it is possible to somehow whitelist all combination of
a company name with all possible Top Level Domain ?

With Tivilsta you can do that through the `RDZ ` flag. This flag is extremely
broad and powerful as it will fetch the
[IANA Root Zone Database](https://www.iana.org/domains/root/db) and the
[Public Suffix List](https://publicsuffix.org/)
to build a set of rules with all possible gTLDs or extensions - if you prefer.

```
RZD example
```

In this example, any subject matching `example.[gTLD]` will be whitelisted.


# Usage & Examples

## Library

```rust
use tivilsta::Ruler;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let my_subjects: Vec<String> = vec![
        String::from("example.com"),
        String::from("example.org"),
        String::from("api.example.org"),
        String::from("test.example.com"),
    ];

    let whitelisting_rules: Vec<String> = vec![
        String::from("api.example.org"),
        String::from("ALL .com"),
    ];

    let mut ruler = Ruler::new(false);
    ruler.parse_vec(&whitelisting_rules);

    for subject in my_subjects {
        if ruler.is_whitelisted(&subject) {
            println!("{} is WHITELISTED", subject)
        } else {
            println!("{} is still BLOCKLISTED", subject)
        }
    }

    Ok(())
}
```

**Output:**

```
example.com is WHITELISTED
example.org is still BLOCKLISTED
api.example.org is WHITELISTED
test.example.com is WHITELISTED
```

## CLI

### Overview

Example for argument with multiple values or files:

- `tivilsta -s test.list -w whitelist1.list -w  whitelist2.list`
- `tivilsta -s test.list --reg reg1.list --reg reg2.list`


| Argument              | Required           | Multiple Values Allowed | Description                                                                                                                             |
| --------------------- | ------------------ | ----------------------- | --------------------------------------------------------------------------------------------------------------------------------------- |
| `--source` \| `-s`    | :white_check_mark: | :x:                     | The source file. In other word the block list to process.                                                                               |
| `--whitelist` \| `-w` | :white_check_mark: | :white_check_mark:      | The whitelist schema file-s. Each line will be processed **AS IT IS.**                                                                  |
| `--all`               | :x:                | :white_check_mark:      | The whitelist schema file-s. Each line will be prefixed with the `ALL ` flag.                                                           |
| `--allow-complements` | :x:                | :x:                     | Whether we should consider complements when whitelisting. A complement is `www.example.org` when `example.org` is given and vice-versa. |
| `--help` \| `-h`      | :x:                | :x:                     | Prints the help message and exit.                                                                                                       |
| `--output` \| `-o`    | :x:                | :x:                     | The output file. By default the tool will output to `stdout`. You can use this argument to explicitly set the output file.              |
| `--reg`               | :x:                | :white_check_mark:      | The whitelist schema file-s. Each line will be prefixed with the `REG ` flag.                                                           |
| `--rzd`               | :x:                | :white_check_mark:      | The whitelist schema file-s. Each line will be prefixed with the `RDZ ` flag.                                                           |
| `--version` \| `-V`   | :x:                | :x:                     | Prints the version and exit.                                                                                                            |

### Help Output

```
A different whitelisting mechanism for blocklist maintainers.

USAGE:
    tivilsta [OPTIONS] --source <SOURCE> --whitelist <WHITELIST>...

OPTIONS:
        --all <ALL>...                One or multiple space separated whitelisting schema in form of
                                      a file path or URL to read. Each rule/line will be
                                      automatically prefixed with the `ALL ` flag while parsing.
                                      Note: When using a URL, the file will be downloaded and stored
                                      in a temporary file that will be deleted when the program
                                      exits
        --allow-complements           Whether we consider complements while parsing rules. Note:
                                      Complements are `www.example.org` if `example.org` is given -
                                      and vice-versa
    -h, --help                        Print help information
    -o, --output <OUTPUT>             The output file
        --reg <REG>...                One or multiple space separated whitelisting schema in form of
                                      a file path or URL to read. Each rule/line will be
                                      automatically prefixed with the `REG ` flag while parsing.
                                      Note: When using a URL, the file will be downloaded and stored
                                      in a temporary file that will be deleted when the program
                                      exits
        --rzd <RZD>...                One or multiple space separated whitelisting schema in form of
                                      a file path or URL to read. Each rule/line will be
                                      automatically prefixed with the `RZD ` flag while parsing.
                                      Note: When using a URL, the file will be downloaded and stored
                                      in a temporary file that will be deleted when the program
                                      exits
    -s, --source <SOURCE>             The file to cleanup
    -V, --version                     Print version information
    -w, --whitelist <WHITELIST>...    One or multiple space separated whitelisting schema in form of
                                      a file path or URL. Each rule/line will be parsed as-it-is.
                                      Note: When using a URL, the file will be downloaded and stored
                                      in a temporary file that will be deleted when the program
                                      exits
```


### Simple whitelisting example

```shell
$ cat test.list
example.org
example.com
api.example.org
test.example.com

$ cat whitelist.list
api.example.org
ALL .com

$ tivilsta -s test.list -w whitelist.list
example.org
```

# License

```
Copyright (c) 2022, 2023 Nissar Chababy

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

    http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.
```
