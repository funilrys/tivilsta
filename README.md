# Tivilsta - A different whitelisting mechanism for blocklist maintainers

Tivilsta aims to provide a different and in our view better whitelisting mechanism
for blocklist maintainers.

# Table of Content

- [Usage & Examples](#usage--examples)
  - [Library](#library)
  - [CLI](#cli)
    - [Overview](#overview)
    - [Help Output](#help-output)
    - [Simple whitelisting example](#simple-whitelisting-example)
- [License](#license)

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
    tivilsta [OPTIONS] --source <SOURCE> --whitelist <WHITELIST>

OPTIONS:
        --all <ALL>                A whitelisting schema/file to read. Each rule will be
                                   automatically prefixed with the `ALL ` flag while parsing
        --allow-complements        Whether we shouls consider complements while parsing rules. Note:
                                   Complements are `www.example.org` if `examplr.org` os given - and
                                   vice-versa
    -h, --help                     Print help information
    -o, --output <OUTPUT>          The output file
        --reg <REG>                A whitelisting schema/file to read. Each rule will be
                                   automatically prefixed with the `REG ` flag while parsing
        --rzd <RZD>                A whitelisting schema/file to read. Each rule will be
                                   automatically prefixed with the `RZD ` flag while parsing
    -s, --source <SOURCE>          The file to cleanup
    -V, --version                  Print version information
    -w, --whitelist <WHITELIST>    A whitelisting schema/file. Each rules will be parsed as-it-is
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
Copyright (c) 2022 Nissar Chababy

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
