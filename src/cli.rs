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

use crate::Arguments;
use tivilsta::Ruler;
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::{fs::File, path::PathBuf};
use tempfile::NamedTempFile;

#[derive(Debug)]
struct CLIHandlerSettings {
    output_given: bool,
}

#[derive(Debug)]
struct CLIHandlerTmp {
    output: NamedTempFile,
}

#[derive(Debug)]
struct CLIHandlerPaths {
    source: PathBuf,
    output: PathBuf,
    whitelist: Vec<PathBuf>,
    all_prefixed: Vec<PathBuf>,
    reg_prefixed: Vec<PathBuf>,
    rzd_prefixed: Vec<PathBuf>,
}

#[derive(Debug)]
pub struct CLIHandler {
    source: File,
    whitelist: Vec<File>,
    all_prefixed: Vec<File>,
    reg_prefixed: Vec<File>,
    rzd_prefixed: Vec<File>,
    ruler: Ruler,
    settings: CLIHandlerSettings,
    tmp: CLIHandlerTmp,
    paths: CLIHandlerPaths,
}

impl CLIHandler {
    pub fn new(args: Arguments) -> CLIHandler {
        let mut paths = CLIHandlerPaths {
            source: PathBuf::new(),
            output: PathBuf::new(),
            whitelist: vec![],
            all_prefixed: vec![],
            reg_prefixed: vec![],
            rzd_prefixed: vec![],
        };
        let tmp = CLIHandlerTmp {
            output: NamedTempFile::new().unwrap(),
        };
        let mut settings = CLIHandlerSettings {
            output_given: false,
        };

        settings.output_given = !args.output.is_none();
        paths.source = args.source;
        paths.output = args.output.unwrap_or_default();

        let mut whitelist: Vec<File> = vec![];
        let mut all_prefixed: Vec<File> = vec![];
        let mut reg_prefixed: Vec<File> = vec![];
        let mut rzd_prefixed: Vec<File> = vec![];

        if !args.whitelist.is_empty() {
            for file in args.whitelist {
                whitelist.push(File::open(&file).unwrap());
                paths.whitelist.push(file.clone());
            }
        }

        if !args.all.is_empty() {
            for file in args.all {
                all_prefixed.push(File::open(&file).unwrap());
                paths.all_prefixed.push(file.clone())
            }
        }

        if !args.reg.is_empty() {
            for file in args.reg {
                reg_prefixed.push(File::open(&file).unwrap());
                paths.reg_prefixed.push(file.clone())
            }
        }

        if !args.rzd.is_empty() {
            for file in args.rzd {
                rzd_prefixed.push(File::open(&file).unwrap());
                paths.rzd_prefixed.push(file.clone())
            }
        }

        let mut result = CLIHandler {
            source: File::open(&paths.source).unwrap(),
            whitelist: whitelist,
            all_prefixed: all_prefixed,
            reg_prefixed: reg_prefixed,
            rzd_prefixed: rzd_prefixed,
            ruler: Ruler::new(args.allow_complements),
            settings: settings,
            tmp: tmp,
            paths: paths,
        };

        result.load_all();
        result
    }

    fn load_whitelist(&mut self) -> bool {
        for file in &self.whitelist {
            let whitelist_file = BufReader::new(file);

            for line in whitelist_file.lines() {
                self.ruler.parse(&line.unwrap())
            }
        }

        for file in &self.all_prefixed {
            let whitelist_file = BufReader::new(file);

            for line in whitelist_file.lines() {
                self.ruler.parse(&format!("ALL {}", &line.unwrap()))
            }
        }

        for file in &self.reg_prefixed {
            let whitelist_file = BufReader::new(file);

            for line in whitelist_file.lines() {
                self.ruler.parse(&format!("REG {}", &line.unwrap()))
            }
        }

        for file in &self.rzd_prefixed {
            let whitelist_file = BufReader::new(file);

            for line in whitelist_file.lines() {
                self.ruler.parse(&format!("RZD {}", &line.unwrap()))
            }
        }

        true
    }

    pub fn load_all(&mut self) -> bool {
        self.load_whitelist()
    }

    pub fn cleanup(&mut self) -> bool {
        let src = BufReader::new(&self.source);

        for line in src.lines() {
            let line = line.unwrap();

            if self.ruler.is_whitelisted(&line) {
                continue;
            }

            let _ = self
                .tmp
                .output
                .write((line.to_string() + "\n").as_bytes())
                .unwrap();

            if !self.settings.output_given {
                println!("{}", &line)
            }
        }

        if self.settings.output_given {
            let _ = fs::copy(self.tmp.output.path(), &self.paths.output).unwrap();
        }

        true
    }
}
