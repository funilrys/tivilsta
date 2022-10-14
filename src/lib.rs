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

mod data;
mod utils;

use crate::data::iana;
use crate::data::psl;
use fancy_regex::Regex;
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::collections::HashSet;
use std::fs;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Debug)]
pub struct RulerSettings {
    handle_complement: bool,
    extensions: Vec<String>,
}

#[derive(Debug)]
pub struct RulerTmps {
    downloaded_files: Vec<String>,
}

#[derive(Debug)]
pub struct Ruler {
    strict: HashMap<String, HashSet<String>>,
    ends: HashMap<String, HashSet<String>>,
    present: HashMap<String, HashSet<String>>,
    regex: String,
    compiled_regex: Regex,
    settings: RulerSettings,
    tmps: RulerTmps,
}

impl Ruler {
    /// Creates a new empty Ruler object.
    ///
    /// # Arguments
    ///
    /// * `handle_complement` - Whether we should follow and cleanup complements.
    /// A complement is `www.example.org` if `example.org` has been given - and vice-versa.
    ///
    /// # Returns
    ///
    /// A new Ruler object.
    ///
    /// # Example
    ///
    /// ### Parsing a vector
    ///
    /// ```rust
    /// use tivilsta::Ruler;
    ///
    /// let mut ruler = Ruler::new(false);
    ///
    /// let my_subjects: Vec<String> = vec![
    ///     String::from("example.com"),
    ///     String::from("example.org"),
    ///     String::from("api.example.org"),
    ///     String::from("test.example.com"),
    /// ];
    ///
    /// let whitelisting_rules: Vec<String> = vec![
    ///     String::from("api.example.org"),
    ///     String::from("ALL .com"),
    /// ];
    ///
    /// // Check that no rule is loaded.
    /// assert_eq!(ruler.is_whitelisted(&String::from("example.com")), false);
    /// assert_eq!(ruler.is_whitelisted(&String::from("example.org")), false);
    /// assert_eq!(ruler.is_whitelisted(&String::from("api.example.com")), false);
    /// assert_eq!(ruler.is_whitelisted(&String::from("test.example.com")), false);
    ///
    /// // Let's parse our rules.
    /// ruler.parse_vec(&whitelisting_rules);
    ///
    /// assert_eq!(ruler.is_whitelisted(&String::from("example.com")), true);
    /// assert_eq!(ruler.is_whitelisted(&String::from("example.org")), false);
    /// assert_eq!(ruler.is_whitelisted(&String::from("api.example.com")), true);
    /// assert_eq!(ruler.is_whitelisted(&String::from("test.example.com")), true);
    ///
    /// // Let's unparse our rules.
    /// ruler.unparse_vec(&whitelisting_rules);
    ///
    /// assert_eq!(ruler.is_whitelisted(&String::from("example.com")), false);
    /// assert_eq!(ruler.is_whitelisted(&String::from("example.org")), false);
    /// assert_eq!(ruler.is_whitelisted(&String::from("api.example.com")), false);
    /// assert_eq!(ruler.is_whitelisted(&String::from("test.example.com")), false);
    /// ```
    pub fn new(handle_complement: bool) -> Ruler {
        Ruler {
            strict: HashMap::new(),
            ends: HashMap::new(),
            present: HashMap::new(),
            regex: String::from(""),
            compiled_regex: Regex::new("").unwrap(),
            settings: RulerSettings {
                handle_complement,
                extensions: vec![],
            },
            tmps: RulerTmps {
                downloaded_files: vec![],
            },
        }
    }

    fn reduce(&self, element: &String) -> String {
        let result;

        if element.starts_with("www.") {
            result = element[4..].to_string()
        } else {
            result = element.to_string();
        }

        result
    }

    fn extensions() -> Vec<String> {
        let mut extensions: Vec<String> = Vec::new();

        let mut iana_extensions = iana::extensions().unwrap();
        let mut psl_suffixes = psl::suffixes().unwrap();

        extensions.append(&mut iana_extensions);
        extensions.append(&mut psl_suffixes);

        extensions
    }

    fn search_keys(&mut self, record: &String) -> (String, String) {
        let common_search_key = record.chars().take(4).collect::<String>();
        let ends_search_key = record
            .chars()
            .rev()
            .take(3)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect::<String>();

        (common_search_key, ends_search_key)
    }

    fn push_strict(&mut self, record: &String) {
        let (search_key, _) = self.search_keys(&self.reduce(record));

        match self.strict.entry(search_key) {
            Entry::Occupied(mut entry) => {
                entry.get_mut().insert(record.to_string());
            }
            Entry::Vacant(entry) => {
                let mut dataset = HashSet::new();

                dataset.insert(record.to_string());
                entry.insert(dataset);
            }
        }
    }

    fn pull_strict(&mut self, record: &String) {
        let (search_key, _) = self.search_keys(&self.reduce(record));

        match self.strict.entry(search_key) {
            Entry::Occupied(mut entry) => {
                entry.get_mut().remove(record);
            }
            Entry::Vacant(entry) => {
                let _ = entry;
            }
        }
    }

    fn push_present(&mut self, record: &String) {
        let (search_key, _) = self.search_keys(&self.reduce(record));

        match self.present.entry(search_key) {
            Entry::Occupied(mut entry) => {
                entry.get_mut().insert(record.to_string());
            }
            Entry::Vacant(entry) => {
                let mut dataset = HashSet::new();

                dataset.insert(record.to_string());
                entry.insert(dataset);
            }
        }
    }

    fn pull_present(&mut self, record: &String) {
        let (search_key, _) = self.search_keys(&self.reduce(record));

        match self.present.entry(search_key) {
            Entry::Occupied(mut entry) => {
                entry.get_mut().remove(record);
            }
            Entry::Vacant(entry) => {
                let _ = entry;
            }
        }
    }

    fn push_ends(&mut self, record: &String) {
        let (_, search_key) = self.search_keys(&self.reduce(record));

        match self.ends.entry(search_key) {
            Entry::Occupied(mut entry) => {
                entry.get_mut().insert(record.to_string());
            }
            Entry::Vacant(entry) => {
                let mut dataset = HashSet::new();

                dataset.insert(record.to_string());
                entry.insert(dataset);
            }
        }
    }

    fn pull_ends(&mut self, record: &String) {
        let (_, search_key) = self.search_keys(&self.reduce(record));

        match self.ends.entry(search_key) {
            Entry::Occupied(mut entry) => {
                entry.get_mut().remove(record);
            }
            Entry::Vacant(entry) => {
                let _ = entry;
            }
        }
    }

    fn push_regex(&mut self, record: &String) {
        if self.regex.is_empty() {
            self.regex.push_str(&record.to_string());
        } else {
            self.regex.push_str(&format!("|{}", record));
        }

        self.compiled_regex = Regex::new(&self.regex[..]).unwrap();
    }

    fn pull_regex(&mut self, record: &String) {
        if self.regex.starts_with(record) && self.regex.ends_with(record) {
            self.regex = String::from("");
        } else if self.regex.starts_with(record) {
            self.regex = self.regex.replace(&format!("{}|", record), "");
        } else {
            self.regex = self.regex.replace(&format!("|{}", record), "");
        }

        self.compiled_regex = Regex::new(&self.regex[..]).unwrap();
    }

    fn parse_all(&mut self, line: &String) -> bool {
        let record: String;

        if line.starts_with("ALL ") {
            record = line.replacen("ALL ", "", 1).trim().to_string()
        } else if line.starts_with("all ") {
            record = line.replacen("all ", "", 1).trim().to_string()
        } else {
            return false;
        }

        if record.starts_with('.') {
            if record.matches('.').count() > 1 {
                if self.settings.handle_complement {
                    self.push_strict(&format!("www.{}", record[1..].to_string()));
                }
                self.push_strict(&record[1..].to_string());
            }
            self.push_ends(&record);
        } else {
            self.parse(&format!("ALL .{}", record));
        }

        true
    }

    fn unparse_all(&mut self, line: &String) -> bool {
        let record: String;

        if line.starts_with("ALL ") {
            record = line.replacen("ALL ", "", 1).trim().to_string()
        } else if line.starts_with("all ") {
            record = line.replacen("all ", "", 1).trim().to_string()
        } else {
            return false;
        }

        if record.starts_with('.') {
            if record.matches('.').count() > 1 {
                if self.settings.handle_complement {
                    self.pull_strict(&format!("www.{}", record[1..].to_string()));
                }
                self.pull_strict(&record[1..].to_string());
            }
            self.pull_ends(&record);
        } else {
            self.unparse(&format!("ALL .{}", record));
        }

        true
    }

    fn parse_root_zone_db(&mut self, line: &String) -> bool {
        let mut record: String;

        if line.starts_with("RZD ") {
            record = line.replacen("RZD ", "", 1).trim().to_string()
        } else if line.starts_with("rzd ") {
            record = line.replacen("rzd ", "", 1).trim().to_string()
        } else {
            return false;
        }

        if self.settings.handle_complement && record.starts_with("www.") {
            record = record.replacen("www.", "", 1).trim().to_string();
        }

        if self.settings.extensions.is_empty() {
            self.settings.extensions = Ruler::extensions()
        }

        for extension in &self.settings.extensions.clone() {
            self.push_present(&format!("{}.{}", record, extension));

            if self.settings.handle_complement {
                self.push_present(&format!("www.{}.{}", record, extension));
            }
        }

        true
    }

    fn unparse_root_zone_db(&mut self, line: &String) -> bool {
        let mut record: String;

        if line.starts_with("RZD ") {
            record = line.replacen("RZD ", "", 1).trim().to_string()
        } else if line.starts_with("rzd ") {
            record = line.replacen("rzd ", "", 1).trim().to_string()
        } else {
            return false;
        }

        if self.settings.handle_complement && record.starts_with("www.") {
            record = record.replacen("www.", "", 1).trim().to_string();
        }

        if self.settings.extensions.is_empty() {
            self.settings.extensions = Ruler::extensions()
        }

        for extension in &self.settings.extensions.clone() {
            self.pull_present(&format!("{}.{}", record, extension));

            if self.settings.handle_complement {
                self.pull_present(&format!("www.{}.{}", record, extension));
            }
        }

        true
    }

    fn parse_regex(&mut self, line: &String) -> bool {
        let record: String;

        if line.starts_with("REG ") {
            record = line.replacen("REG ", "", 1).trim().to_string()
        } else if line.starts_with("reg ") {
            record = line.replacen("reg ", "", 1).trim().to_string()
        } else {
            return false;
        }

        self.push_regex(&record);

        true
    }

    fn unparse_regex(&mut self, line: &String) -> bool {
        let record: String;

        if line.starts_with("REG ") {
            record = line.replacen("REG ", "", 1).trim().to_string()
        } else if line.starts_with("reg ") {
            record = line.replacen("reg ", "", 1).trim().to_string()
        } else {
            return false;
        }

        self.pull_regex(&record);

        true
    }

    fn parse_plain(&mut self, line: &String) -> bool {
        let record: String;

        if self.settings.handle_complement && line.starts_with("www.") {
            record = line.replacen("www.", "", 1).trim().to_string();
        } else {
            record = line.to_string();
        }

        self.push_strict(&record);

        if self.settings.handle_complement {
            self.push_strict(&format!("www.{}", record));
        }

        true
    }

    fn unparse_plain(&mut self, line: &String) -> bool {
        let record: &String = &self.reduce(line);
        self.pull_strict(record);

        if self.settings.handle_complement {
            self.pull_strict(&format!("www.{}", record));
        }

        true
    }

    /// Parses the given String into the ruler.
    ///
    /// # Arguments
    ///
    /// * `line` - The line to parse.
    ///
    /// # Returns
    ///
    /// Nothing.
    pub fn parse(&mut self, line: &String) {
        if line.is_empty() || line.starts_with('#') {
            return;
        }

        let _ = self.parse_all(line)
            || self.parse_regex(line)
            || self.parse_root_zone_db(line)
            || self.parse_plain(line);
    }

    /// Parses the given Vector of Strings into the ruler.
    ///
    /// # Arguments
    ///
    /// * `lines` - The lines to parse.
    ///
    /// # Returns
    ///
    /// Nothing.
    pub fn parse_vec(&mut self, lines: &[String]) {
        for line in lines {
            self.parse(line);
        }
    }

    /// Parses the content of the given file into the ruler.
    ///
    /// # Arguments
    ///
    /// * `file` - The file to parse.
    ///
    /// # Returns
    ///
    /// Nothing.
    pub fn parse_file(&mut self, path: &str) {
        let file = File::open(path).unwrap();
        let reader = BufReader::new(file);

        for line in reader.lines() {
            self.parse(&line.unwrap());
        }
    }

    /// Parses the content of the given URL (after downloading it) into the ruler.
    ///
    /// # Arguments
    ///
    /// * `url` - The URL to download and parse.
    ///
    /// # Returns
    ///
    /// Nothing.
    pub fn parse_link(&mut self, url: &str) {
        let (real_path, downloaded) = utils::download_file(&url.to_string());

        if downloaded {
            self.tmps.downloaded_files.push(real_path.clone());
        }

        self.parse_file(real_path.as_str());
    }

    /// Unparses the given String into the ruler.
    ///
    /// # Arguments
    ///
    /// * `line` - The line to parse.
    ///
    /// # Returns
    ///
    /// Nothing.
    pub fn unparse(&mut self, line: &String) {
        if line.is_empty() || line.starts_with('#') {
            return;
        }

        let _ = self.unparse_all(line)
            || self.unparse_regex(line)
            || self.unparse_root_zone_db(line)
            || self.unparse_plain(line);
    }

    /// Unparses the given Vector of Strings into the ruler.
    ///
    /// # Arguments
    ///
    /// * `lines` - The lines to parse.
    ///
    /// # Returns
    ///
    /// Nothing.
    pub fn unparse_vec(&mut self, lines: &[String]) {
        for line in lines {
            self.unparse(line);
        }
    }

    /// Unparses the content of the given file into the ruler.
    ///
    /// # Arguments
    ///
    /// * `file` - The file to parse.
    ///
    /// # Returns
    ///
    /// Nothing.
    pub fn unparse_file(&mut self, path: &str) {
        let file = File::open(path).unwrap();
        let reader = BufReader::new(file);

        for line in reader.lines() {
            self.unparse(&line.unwrap());
        }
    }

    /// Unparses the content of the given URL (after downloading it) into the ruler.
    ///
    /// # Arguments
    ///
    /// * `url` - The URL to download and parse.
    ///
    /// # Returns
    ///
    /// Nothing.
    pub fn unparse_link(&mut self, url: &str) {
        let (real_path, downloaded) = utils::download_file(&url.to_string());

        if downloaded {
            self.tmps.downloaded_files.push(real_path.clone());
        }

        self.unparse_file(real_path.as_str());
    }

    /// Checks the given `line` against the rules.
    ///
    /// # Arguments
    ///
    /// * `line` - The line to check.
    ///
    ///   **Note:** If a URL (e.g `https://example.org/`) is given, the sub-domain
    ///   will be used to determine if the line has been whitelisted.
    ///
    /// # Returns
    ///
    /// A `bool` indicating whether the line matches the rules.
    /// Any `true` value should be considered positive.
    /// Meaning that the line matches one of the rule.
    pub fn is_whitelisted(&mut self, line: &String) -> bool {
        if line.is_empty() || line.starts_with('#') {
            return false;
        }

        let fline = utils::extract_netloc(&line);

        let (common_skey, ends_skey) = self.search_keys(&self.reduce(&fline));

        let mut matching_state;

        match self.strict.entry(common_skey.to_string()) {
            Entry::Occupied(entry) => matching_state = entry.get().contains(&fline),
            Entry::Vacant(_) => matching_state = false,
        }

        if matching_state {
            return true;
        }

        match self.present.entry(common_skey) {
            Entry::Occupied(entry) => matching_state = entry.get().contains(&fline),
            Entry::Vacant(_) => matching_state = false,
        }

        if matching_state {
            return true;
        }

        match self.ends.entry(ends_skey) {
            Entry::Occupied(entry) => {
                let mut matching = entry.get().iter().map(|x| fline.ends_with(x)).peekable();
                matching_state = *matching.peek().unwrap_or(&false);
            }
            Entry::Vacant(_) => matching_state = false,
        }

        if matching_state {
            return true;
        }

        !self.regex.is_empty() && self.compiled_regex.is_match(&fline[..]).unwrap()
    }
}

impl Drop for Ruler {
    fn drop(&mut self) {
        for file in &self.tmps.downloaded_files {
            let _ = fs::remove_file(file);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_ruler_gen_complement_true() {
        let ruler = Ruler::new(true);

        assert_eq!(ruler.settings.handle_complement, true)
    }

    #[test]
    fn test_new_ruler_gen_complement_false() {
        let ruler = Ruler::new(false);

        assert_eq!(ruler.settings.handle_complement, false)
    }

    #[test]
    fn test_reduce() {
        let ruler = Ruler::new(false);

        assert_eq!(
            ruler.reduce(&"www.example.org".to_string()),
            "example.org".to_string()
        )
    }

    #[test]
    fn test_reduce_no_www() {
        let ruler = Ruler::new(false);

        assert_eq!(
            ruler.reduce(&"example.org".to_string()),
            "example.org".to_string()
        )
    }

    #[test]
    fn test_reduce_multiple_www() {
        let ruler = Ruler::new(false);

        assert_eq!(
            ruler.reduce(&"www.www.example.org".to_string()),
            "www.example.org".to_string()
        )
    }

    #[test]
    fn test_search_keys() {
        let mut ruler = Ruler::new(false);

        assert_eq!(
            ruler.search_keys(&"example.org".to_string()),
            ("exam".to_string(), "org".to_string())
        )
    }

    #[test]
    fn test_search_keys_long_extension() {
        let mut ruler = Ruler::new(false);

        assert_eq!(
            ruler.search_keys(&"example.example".to_string()),
            ("exam".to_string(), "ple".to_string())
        )
    }

    #[test]
    fn test_push_strict() {
        let mut ruler = Ruler::new(false);

        // Ensure that it's really empty :)
        assert_eq!(ruler.strict.get_key_value("exam"), None);

        ruler.push_strict(&"www.example.org".to_string());

        let mut expected = HashSet::new();
        expected.insert("www.example.org".to_string());

        assert_eq!(
            ruler.strict.get_key_value("exam"),
            Some((&"exam".to_string(), &expected))
        );

        // Let's add another one.

        ruler.push_strict(&"example.net".to_string());
        expected.insert("example.net".to_string());

        assert_eq!(
            ruler.strict.get_key_value("exam"),
            Some((&"exam".to_string(), &expected))
        );
    }

    #[test]
    fn test_pull_strict() {
        let mut ruler = Ruler::new(false);

        // Ensure that it's really empty :)
        assert_eq!(ruler.strict.get_key_value("exam"), None);

        // Add some data into it :)
        ruler.push_strict(&"www.example.org".to_string());
        ruler.push_strict(&"example.net".to_string());

        ruler.pull_strict(&"www.example.org".to_string());

        let mut expected = HashSet::new();
        expected.insert("example.net".to_string());

        assert_eq!(
            ruler.strict.get_key_value("exam"),
            Some((&"exam".to_string(), &expected))
        );

        // Let's remove another one.
        ruler.pull_strict(&"example.net".to_string());
        expected.remove("example.net");

        assert_eq!(
            ruler.strict.get_key_value("exam"),
            Some((&"exam".to_string(), &expected))
        );
    }

    #[test]
    fn test_push_present() {
        let mut ruler = Ruler::new(false);

        // Ensure that it's really empty :)
        assert_eq!(ruler.present.get_key_value("exam"), None);

        ruler.push_present(&"www.example.net".to_string());

        let mut expected = HashSet::new();
        expected.insert("www.example.net".to_string());

        assert_eq!(
            ruler.present.get_key_value("exam"),
            Some((&"exam".to_string(), &expected))
        );

        // Let's add another one.

        ruler.push_present(&"example.com".to_string());
        expected.insert("example.com".to_string());

        assert_eq!(
            ruler.present.get_key_value("exam"),
            Some((&"exam".to_string(), &expected))
        );
    }

    #[test]
    fn test_pull_present() {
        let mut ruler = Ruler::new(false);

        // Ensure that it's really empty :)
        assert_eq!(ruler.present.get_key_value("exam"), None);

        // Add some data into it :)
        ruler.push_present(&"www.example.net".to_string());
        ruler.push_present(&"example.org".to_string());

        ruler.pull_present(&"www.example.net".to_string());

        let mut expected = HashSet::new();
        expected.insert("example.org".to_string());

        assert_eq!(
            ruler.present.get_key_value("exam"),
            Some((&"exam".to_string(), &expected))
        );

        // Let's remove another one.
        ruler.pull_present(&"example.org".to_string());
        expected.remove("example.org");

        assert_eq!(
            ruler.present.get_key_value("exam"),
            Some((&"exam".to_string(), &expected))
        );
    }

    #[test]
    fn test_push_ends() {
        let mut ruler = Ruler::new(false);

        // Ensure that it's really empty :)
        assert_eq!(ruler.ends.get_key_value("ple"), None);

        ruler.push_ends(&"www.example.example".to_string());

        let mut expected = HashSet::new();
        expected.insert("www.example.example".to_string());

        assert_eq!(
            ruler.ends.get_key_value("ple"),
            Some((&"ple".to_string(), &expected))
        );

        // Let's add another one.

        ruler.push_ends(&"example.com".to_string());

        let mut expected = HashSet::new();
        expected.insert("example.com".to_string());

        assert_eq!(
            ruler.ends.get_key_value("com"),
            Some((&"com".to_string(), &expected))
        );

        // Let's add another one.

        ruler.push_ends(&"example.co".to_string());

        let mut expected = HashSet::new();
        expected.insert("example.co".to_string());

        assert_eq!(
            ruler.ends.get_key_value(".co"),
            Some((&".co".to_string(), &expected))
        );

        assert_eq!(ruler.ends.contains_key("com"), true);
        assert_eq!(ruler.ends.contains_key("ple"), true);
        assert_eq!(ruler.ends.contains_key(".co"), true);
    }

    #[test]
    fn test_pull_ends() {
        let mut ruler = Ruler::new(false);

        // Ensure that it's really empty :)
        assert_eq!(ruler.ends.get_key_value("ple"), None);

        // Add some data into it :)
        ruler.push_ends(&"www.example.example".to_string());
        ruler.push_ends(&"example.com".to_string());
        ruler.push_ends(&"example.co".to_string());

        assert_eq!(ruler.ends.contains_key("com"), true);
        assert_eq!(ruler.ends.contains_key("ple"), true);
        assert_eq!(ruler.ends.contains_key(".co"), true);

        ruler.pull_ends(&"www.example.example".to_string());

        let expected = HashSet::new();

        assert_eq!(
            ruler.ends.get_key_value("ple"),
            Some((&"ple".to_string(), &expected))
        );

        let mut expected = HashSet::new();
        expected.insert("example.com".to_string());

        assert_eq!(
            ruler.ends.get_key_value("com"),
            Some((&"com".to_string(), &expected))
        );

        let mut expected = HashSet::new();
        expected.insert("example.co".to_string());

        assert_eq!(
            ruler.ends.get_key_value(".co"),
            Some((&".co".to_string(), &expected))
        );

        // Let's remove another one.
        ruler.pull_ends(&"example.com".to_string());

        let expected = HashSet::new();

        assert_eq!(
            ruler.ends.get_key_value("com"),
            Some((&"com".to_string(), &expected))
        );

        assert_eq!(ruler.ends.contains_key("com"), true);
        assert_eq!(ruler.ends.contains_key("ple"), true);
        assert_eq!(ruler.ends.contains_key(".co"), true);
    }

    #[test]
    fn test_push_regex() {
        let mut ruler = Ruler::new(false);

        // Ensure that it's really empty :)
        assert_eq!(ruler.regex, "");
        assert_eq!(ruler.compiled_regex.as_str(), "");

        ruler.push_regex(&"^(www.)?example.com$".to_string());

        let expected = "^(www.)?example.com$".to_string();

        assert_eq!(ruler.regex, expected);
        assert_eq!(ruler.compiled_regex.as_str(), &expected[..]);

        // Let's add another one.
        ruler.push_regex(&"^(api.)?example.org$".to_string());

        let expected = "^(www.)?example.com$|^(api.)?example.org$".to_string();

        assert_eq!(ruler.regex, expected);
        assert_eq!(ruler.compiled_regex.as_str(), &expected[..]);
    }

    #[test]
    fn test_pull_regex() {
        let mut ruler = Ruler::new(false);

        // Ensure that it's really empty :)
        assert_eq!(ruler.regex, "");
        assert_eq!(ruler.compiled_regex.as_str(), "");

        // Add some data into it :)
        ruler.push_regex(&"^(www.)?example.com$".to_string());
        ruler.push_regex(&"^(api.)?example.org$".to_string());

        ruler.pull_regex(&"^(www.)?example.com$".to_string());

        let expected = "^(api.)?example.org$".to_string();

        assert_eq!(ruler.regex, expected);
        assert_eq!(ruler.compiled_regex.as_str(), &expected[..]);

        // Let's remove another one.
        ruler.pull_regex(&"^(api.)?example.org$".to_string());

        let expected = "".to_string();

        assert_eq!(ruler.regex, expected);
        assert_eq!(ruler.compiled_regex.as_str(), &expected[..]);
    }

    #[test]
    fn test_parse_all() {
        let mut ruler = Ruler::new(false);

        let given = &"example.org".to_string();
        let mut expected_res = false;

        let mut expected_ends: HashMap<String, HashSet<String>> = HashMap::new();
        let mut expected_strict: HashMap<String, HashSet<String>> = HashMap::new();
        let expected_present: HashMap<String, HashSet<String>> = HashMap::new();
        let expected_regex = "".to_string();

        assert_eq!(ruler.parse_all(given), expected_res);
        assert_eq!(ruler.ends, expected_ends);
        assert_eq!(ruler.strict, expected_strict);
        assert_eq!(ruler.present, expected_present);
        assert_eq!(ruler.regex, expected_regex);

        // Let's add a new one.
        let given = &"ALL example.org".to_string();
        expected_res = true;

        let mut ends_set = HashSet::new();
        ends_set.insert(".example.org".to_string());
        expected_ends.insert("org".to_string(), ends_set);

        let mut strict_set = HashSet::new();
        strict_set.insert("example.org".to_string());
        expected_strict.insert("exam".to_string(), strict_set);

        assert_eq!(ruler.parse_all(given), expected_res);
        assert_eq!(ruler.ends, expected_ends);
        assert_eq!(ruler.strict, expected_strict);
        assert_eq!(ruler.present, expected_present);
        assert_eq!(ruler.regex, expected_regex);

        // Let's add another one but the marker is in lowercase.
        let given = &"all .example.net".to_string();
        expected_res = true;

        let mut new_set = HashSet::new();
        new_set.insert(".example.net".to_string());
        expected_ends.insert("net".to_string(), new_set);

        let mut new_set = HashSet::new();
        new_set.insert("example.org".to_string());
        new_set.insert("example.net".to_string());
        expected_strict.insert("exam".to_string(), new_set);

        assert_eq!(ruler.parse_all(given), expected_res);
        assert_eq!(ruler.ends, expected_ends);
        assert_eq!(ruler.strict, expected_strict);
        assert_eq!(ruler.present, expected_present);
        assert_eq!(ruler.regex, expected_regex);

        // Let's add another one but this time with the complement generation.
        ruler.settings.handle_complement = true;

        let given = &"ALL .example.de".to_string();
        expected_res = true;

        let mut new_set = HashSet::new();
        new_set.insert(".example.de".to_string());
        expected_ends.insert(".de".to_string(), new_set);

        let mut new_set = HashSet::new();
        new_set.insert("example.org".to_string());
        new_set.insert("example.net".to_string());
        new_set.insert("example.de".to_string());
        new_set.insert("www.example.de".to_string());

        expected_strict.insert("exam".to_string(), new_set);

        assert_eq!(ruler.parse_all(given), expected_res);
        assert_eq!(ruler.ends, expected_ends);
        assert_eq!(ruler.strict, expected_strict);
        assert_eq!(ruler.present, expected_present);
        assert_eq!(ruler.regex, expected_regex);
    }

    #[test]
    fn test_unparse_all() {
        let mut ruler = Ruler::new(false);

        let given = &"ALL example.com".to_string();
        let mut expected_ends: HashMap<String, HashSet<String>> = HashMap::new();
        let mut expected_strict: HashMap<String, HashSet<String>> = HashMap::new();
        let expected_present: HashMap<String, HashSet<String>> = HashMap::new();
        let expected_regex = "".to_string();

        // Fill ruler with some data
        ruler.parse_all(&"ALL .hello.com".to_string());
        ruler.parse_all(&"ALL .github.com".to_string());
        ruler.parse_all(&"ALL .example.com".to_string());

        let mut ends_set = HashSet::new();
        ends_set.insert(".github.com".to_string());
        ends_set.insert(".hello.com".to_string());
        expected_ends.insert("com".to_string(), ends_set);

        let mut strict_set1 = HashSet::new();
        strict_set1.insert("hello.com".to_string());
        expected_strict.insert("hell".to_string(), strict_set1);

        let mut strict_set2 = HashSet::new();
        strict_set2.insert("github.com".to_string());
        expected_strict.insert("gith".to_string(), strict_set2);
        expected_strict.insert("exam".to_string(), HashSet::new());

        assert_eq!(ruler.unparse_all(given), true);
        assert_eq!(ruler.ends, expected_ends);
        assert_eq!(ruler.strict, expected_strict);
        assert_eq!(ruler.present, expected_present);
        assert_eq!(ruler.regex, expected_regex);

        // Let's remove another one but this time with the complement generation.
        ruler.settings.handle_complement = true;

        ruler.parse_all(&"ALL .hello.com".to_string());

        let mut strict_set1 = HashSet::new();
        strict_set1.insert("hello.com".to_string());
        strict_set1.insert("www.hello.com".to_string());
        expected_strict.insert("hell".to_string(), strict_set1);

        let mut strict_set2 = HashSet::new();
        strict_set2.insert("github.com".to_string());
        expected_strict.insert("gith".to_string(), strict_set2);
        expected_strict.insert("exam".to_string(), HashSet::new());

        let given = &"ALL .hello.world".to_string();

        assert_eq!(ruler.strict, expected_strict);

        assert_eq!(ruler.unparse_all(given), true);
        assert_eq!(ruler.ends, expected_ends);
        assert_eq!(ruler.strict, expected_strict);
        assert_eq!(ruler.present, expected_present);
        assert_eq!(ruler.regex, expected_regex);
    }
}
