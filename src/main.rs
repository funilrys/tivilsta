// Tivilsta - A different whitelisting mechanism
//
// Author:
//      Nissar Chababy, @funilrys, contactTATAfunilrysTODTODcom
//
// License:
//      Copyright (c) 2022, 2023 Nissar Chababy
//
//      Licensed under the Apache License, Version 2.0 (the "License");
//      you may not use this file except in compliance with the License.
//      You may obtain a copy of the License at
//
//          http://www.apache.org/licenses/LICENSE-2.0
//
//      Unless required by applicable law or agreed to in writing, software
//      distributed under the License is distributed on an "AS IS" BASIS,
//      WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
//      See the License for the specific language governing permissions and
//      limitations under the License.

mod cli;
mod data;
mod utils;

use std::path::PathBuf;

use clap::Parser;

use cli::CLIHandler;

#[derive(Parser, Default, Debug)]
#[clap(author = "Nissar Chababy (@funilrys)", version, about)]
/// A tool to compute whitelist lists against your lists or hosts files.
pub struct Arguments {
    #[clap(short, long, parse(from_os_str), required = true)]
    /// The file to cleanup.
    source: PathBuf,

    #[clap(short, long, parse(from_os_str), required = false)]
    /// The output file.
    output: Option<PathBuf>,

    #[clap(short, long, min_values = 1, required = true)]
    /// One or multiple space separated whitelisting schema in form of a file path or URL.
    /// Each rule/line will be parsed as-it-is.
    /// Note: When using a URL, the file will be downloaded and stored in a
    /// temporary file that will be deleted when the program exits.
    whitelist: Vec<String>,

    #[clap(long, min_values = 1, required = false)]
    /// One or multiple space separated whitelisting schema in form of a file path or URL to read.
    /// Each rule/line will be automatically prefixed with the `ALL ` flag while parsing.
    /// Note: When using a URL, the file will be downloaded and stored in a
    /// temporary file that will be deleted when the program exits.
    all: Vec<String>,

    #[clap(long, min_values = 1, required = false)]
    /// One or multiple space separated whitelisting schema in form of a file path or URL to read.
    /// Each rule/line will be automatically prefixed with the `REG ` flag while parsing.
    /// Note: When using a URL, the file will be downloaded and stored in a
    /// temporary file that will be deleted when the program exits.
    reg: Vec<String>,

    // #[clap(long, parse(from_os_str), required = false)]
    #[clap(long, min_values = 1, required = false)]
    /// One or multiple space separated whitelisting schema in form of a file path or URL to read.
    /// Each rule/line will be automatically prefixed with the `RZD ` flag while parsing.
    /// Note: When using a URL, the file will be downloaded and stored in a
    /// temporary file that will be deleted when the program exits.
    rzd: Vec<String>,

    #[clap(long)]
    /// Whether we consider complements while parsing rules.
    /// Note: Complements are `www.example.org` if `example.org` is given - and
    /// vice-versa.
    allow_complements: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Arguments::parse();
    let mut handler = CLIHandler::new(args);

    handler.cleanup();

    Ok(())
}
