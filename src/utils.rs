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

use fancy_regex;

pub fn fetch_json(
    url: String,
    error_message: String,
) -> Result<reqwest::blocking::Response, Box<dyn std::error::Error>> {
    let response = reqwest::blocking::get(url)?;

    if response.status().is_success() {
        Ok(response)
    } else {
        return Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            error_message,
        )));
    }
}

pub fn to_regex_string(extensions: Result<Vec<String>, Box<dyn std::error::Error>>) -> String {
    let result = extensions
        .unwrap()
        .iter()
        .map(|ext| format!(r"((?:\.(?:{})))", fancy_regex::escape(ext)))
        .collect::<Vec<String>>()
        .join("|");

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_regex_string() {
        let given = Ok(vec!["com".to_string(), "google".to_string()]);
        let expected = "((?:\\.(?:com)))|((?:\\.(?:google)))".to_string();

        assert_eq!(to_regex_string(given), expected)
    }

    #[test]
    fn test_to_regex_string_emtpy_vec() {
        let given = Ok(vec![]);
        let expected = "".to_string();

        assert_eq!(to_regex_string(given), expected)
    }

    #[test]
    fn test_to_regex_string_single_ext_vec() {
        let given = Ok(vec!["com".to_string()]);
        let expected = "((?:\\.(?:com)))".to_string();

        assert_eq!(to_regex_string(given), expected)
    }
}
