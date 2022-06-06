// Tivilsta - A different whitelisting mechanism
//
// Author:
//      Nissar Chababy, @funilrys, contactTATAfunilrysTODTODcom
//
// License:
//      Copyright (c) 2022 Nissar Chababy
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

    #[clap(short, long, parse(from_os_str), required = true)]
    /// A whitelisting schema/file. Each rules will be parsed as-it-is.
    whitelist: Vec<PathBuf>,

    #[clap(long, parse(from_os_str), required = false)]
    /// A whitelisting schema/file to read. Each rule will be automatically prefixed
    /// with the `ALL ` flag while parsing.
    all: Vec<PathBuf>,

    #[clap(long, parse(from_os_str), required = false)]
    /// A whitelisting schema/file to read. Each rule will be automatically prefixed
    /// with the `REG ` flag while parsing.
    reg: Vec<PathBuf>,

    #[clap(long, parse(from_os_str), required = false)]
    /// A whitelisting schema/file to read. Each rule will be automatically prefixed
    /// with the `RZD ` flag while parsing.
    rzd: Vec<PathBuf>,

    #[clap(long)]
    /// Whether we d consider complements while parsing rules.
    /// Note: Complements are `www.example.org` if `example.org` os given - and
    /// vice-versa.
    allow_complements: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Arguments::parse();
    let mut handler = CLIHandler::new(args);

    handler.cleanup();

    Ok(())
}
