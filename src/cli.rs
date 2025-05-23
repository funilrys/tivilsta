// Tivilsta - A different whitelisting mechanism
//
// Author:
//      Nissar Chababy, @funilrys, contactTATAfunilrysTODTODcom
//
// License:
//      Copyright (c) 2022, 2023, 2024, 2025 Nissar Chababy
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
use num_cpus;
use std::cmp::max;
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::sync::mpsc;
use std::thread;
use std::{fs::File, path::PathBuf};
use tempfile::NamedTempFile;
use tivilsta::Ruler;

use crate::utils;

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
    whitelist: Vec<String>,
    all_prefixed: Vec<String>,
    reg_prefixed: Vec<String>,
    rzd_prefixed: Vec<String>,
    tmps: Vec<String>,
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
    pub multithread: bool,
    max_threads: usize,
}

impl CLIHandler {
    /// Returns a person with the name given them
    ///
    /// # Arguments
    ///
    /// * `args` - A set of parsed arguments.
    ///
    /// # Examples
    ///
    /// ```
    /// use tivilsta::cli::CLIHandler;
    ///
    /// let args = Arguments::parse();
    /// let mut handler = CLIHandler::new(args);
    ///
    /// // handler already do this for you. But you can force it to reload all (new?) datasets by doing this.
    /// handler.load_all();
    ///
    /// // Process the whitelisting + output based on all inputs.
    /// handler.cleanup();
    /// ```
    pub fn new(args: Arguments) -> CLIHandler {
        let mut paths = CLIHandlerPaths {
            source: PathBuf::new(),
            output: PathBuf::new(),
            whitelist: vec![],
            all_prefixed: vec![],
            reg_prefixed: vec![],
            rzd_prefixed: vec![],
            tmps: vec![],
        };
        let tmp = CLIHandlerTmp {
            output: NamedTempFile::new().unwrap(),
        };
        let mut settings = CLIHandlerSettings {
            output_given: false,
        };

        settings.output_given = args.output.is_some();
        paths.source = args.source;
        paths.output = args.output.unwrap_or_default();

        let mut whitelist: Vec<File> = vec![];
        let mut all_prefixed: Vec<File> = vec![];
        let mut reg_prefixed: Vec<File> = vec![];
        let mut rzd_prefixed: Vec<File> = vec![];

        if !args.whitelist.is_empty() {
            for file in args.whitelist {
                let (path, downloaded) = utils::download_file(&file);

                if downloaded {
                    paths.tmps.push(path.clone())
                }

                whitelist.push(File::open(&path).unwrap());
                paths.whitelist.push(path.clone());
            }
        }

        if !args.all.is_empty() {
            for file in args.all {
                let (path, downloaded) = utils::download_file(&file);

                if downloaded {
                    paths.tmps.push(path.clone())
                }

                all_prefixed.push(File::open(&path).unwrap());
                paths.all_prefixed.push(path.clone())
            }
        }

        if !args.reg.is_empty() {
            for file in args.reg {
                let (path, downloaded) = utils::download_file(&file);

                if downloaded {
                    paths.tmps.push(path.clone())
                }

                reg_prefixed.push(File::open(&path).unwrap());
                paths.reg_prefixed.push(path.clone())
            }
        }

        if !args.rzd.is_empty() {
            for file in args.rzd {
                let (path, downloaded) = utils::download_file(&file);

                if downloaded {
                    paths.tmps.push(path.clone())
                }

                rzd_prefixed.push(File::open(&path).unwrap());
                paths.rzd_prefixed.push(path.clone())
            }
        }

        let mut result = CLIHandler {
            source: File::open(&paths.source).unwrap(),
            whitelist,
            all_prefixed,
            reg_prefixed,
            rzd_prefixed,
            ruler: Ruler::new(args.allow_complements),
            settings,
            tmp,
            paths,
            multithread: args.multithread,
            max_threads: args.max_threads.unwrap_or(max(1, num_cpus::get() - 2)),
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

    /// Loads all external datasets into the ruler.
    /// This is done automatically when the handler is created.
    ///
    /// However, if you - for example - add a new file you can call this method
    /// to force it to load and parse your newly added file.
    pub fn load_all(&mut self) -> bool {
        self.load_whitelist()
    }

    /// Writes the final destination of the output.
    /// Because we only write to the temporary file, we need to copy it to the final destination.
    fn write_final_destination(&self) {
        if self.settings.output_given {
            let _ = fs::copy(self.tmp.output.path(), &self.paths.output).unwrap();
        }
    }

    /// Proceed with the whitelisting and output based on all inputs.
    /// This is a single-threaded version.
    ///
    /// It will read the source file line by line, process each line using the `Ruler` instance,
    /// and write the result to the temporary output file.
    ///
    /// If the `output` argument is not given, it will print the result to stdout.
    pub fn cleanup(&mut self) -> bool {
        let source = self.source.try_clone().unwrap();
        let src = BufReader::new(source);

        for line in src.lines() {
            let line = self.ruler.idnaze_line(&line.unwrap());

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

        self.write_final_destination();

        true
    }

    /// Proceed with the whitelisting and output based on all inputs.
    /// This is a multithreaded version.
    pub fn multithreaded_cleanup(&mut self) -> bool {
        let source = self.source.try_clone().unwrap();
        let src = BufReader::new(source);

        let (line_sender, line_receiver) = mpsc::channel();
        let (output_sender, output_receiver) = mpsc::channel();

        // Wrap the receiver in an Arc<Mutex> to share it across threads
        let line_receiver = std::sync::Arc::new(std::sync::Mutex::new(line_receiver));

        // Spawn worker threads to process lines
        let mut worker_handles = vec![];

        // -2 because we need to keep one thread for the sender and one for the output
        // thread.
        // saturating_sub(2) is used to ensure we have at least one thread
        for _ in 0..max(1, self.max_threads.saturating_sub(2)) {
            let output_sender = output_sender.clone();
            // Ensure we have a new receiver for each thread
            let line_receiver = std::sync::Arc::clone(&line_receiver);

            let mut ruler = self.ruler.clone();

            let handle = thread::spawn(move || {
                while let Ok(line) = line_receiver.lock().unwrap().recv() {
                    // Process the line
                    let processed_line = ruler.idnaze_line(&line);
                    if !ruler.is_whitelisted(&processed_line) {
                        output_sender.send(processed_line).unwrap();
                    }
                }
            });

            worker_handles.push(handle);
        }

        // Spawn a thread to send lines to the line_sender
        let sender_handle = thread::spawn(move || {
            for line in src.lines() {
                let line = line.unwrap();
                line_sender.send(line).unwrap();
            }
            drop(line_sender); // Close the channel to signal workers to stop
        });

        // Drop the original output_sender to signal the output thread when done
        drop(output_sender);

        let output_path = self.tmp.output.path().to_path_buf();
        let stdout_output = !self.settings.output_given;

        // Spawn a thread to handle writing output
        let output_handle = thread::spawn(move || {
            let mut output_file = File::create(output_path).unwrap();

            for line in output_receiver {
                let _ = output_file
                    .write((line.to_string() + "\n").as_bytes())
                    .unwrap();

                if stdout_output {
                    println!("{}", &line)
                }
            }
        });

        // Wait for all threads to finish
        sender_handle.join().unwrap();
        for handle in worker_handles {
            handle.join().unwrap();
        }
        output_handle.join().unwrap();

        self.write_final_destination();

        true
    }
}

impl Drop for CLIHandler {
    /// Ensures that all temporary files or downloaded files are cleaned up.
    fn drop(&mut self) {
        for file in &self.paths.tmps {
            let _ = fs::remove_file(file);
        }
    }
}
