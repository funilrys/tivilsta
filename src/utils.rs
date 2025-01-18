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

use fancy_regex::escape as regex_escape;
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use std::env;
use std::fs::File;
use std::io;
use std::path::Path;
use urlparse::urlparse;

/// A helper function that fetches a remote URL.
///
/// # Arguments
///
/// * `url` - The URL to fetch.
///
/// * `error_message` - A Message to return if the fetch fails.
///
/// # Returns
///
/// A `reqwest::blocking::Response` object to work with.
pub fn fetch_url(
    url: &String,
    error_message: String,
) -> Result<reqwest::blocking::Response, Box<dyn std::error::Error>> {
    let response = reqwest::blocking::get(url)?;

    if response.status().is_success() {
        Ok(response)
    } else {
        Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            error_message,
        )))
    }
}

/// A function that will fetch the content of the given `url` into the given `destination`.
///
/// # Arguments
///
/// * `url` - The URL to fetch.
///
/// * `destination` - The path to the destination file.
///
/// # Returns
///
/// The path of the file where the content was copied into.
pub fn fetch_file(
    url: &String,
    destination: &String,
) -> Result<String, Box<dyn std::error::Error>> {
    let response = fetch_url(url, String::from("Couldn't reach the remote server."))?;

    let body = response.text().expect("Invalid body.");

    let mut output_file = File::create(destination).expect("Couldn't create file.");
    io::copy(&mut body.as_bytes(), &mut output_file).expect("Couldn't write content.");
    Ok(destination.to_string())
}

/// A function that download a presumed `user_input`.
///
/// # Arguments
///
/// * `user_input` - The presumed user input.
/// If it contains `://`, it will be treated as a URL, and downloaded.
/// Otherwise, the given `user_input` will be the direct return value of this function.
///
/// # Returns
///
/// A tuple containing the downloaded file and a boolean informing the end-user
/// whether the the `user_input` was a URL that has been downloaded by this function.
/// In the later case, a path to a file with a random name will be provided as the
/// first part or the tuple.
pub fn download_file(user_input: &String) -> (String, bool) {
    if !user_input.contains("://") {
        return (user_input.clone(), false);
    }

    let filename: String = thread_rng()
        .sample_iter(&Alphanumeric)
        .take(30)
        .map(char::from)
        .collect();

    let temp_file = Path::new(&env::temp_dir().as_os_str()).join(filename);

    let tmp_path = temp_file.to_str().unwrap().to_string();

    return (fetch_file(user_input, &tmp_path).unwrap_or(tmp_path), true);
}

/// A function that will escape a given `extensions` before joining them into
/// a regex in the following format:
///
/// ```txt
/// ((?:\.(?:xx)))|((?:\.(?:yy)))
///
/// Where `xx` and `yy` are extensions.
/// ```
pub fn to_regex_string(extensions: Result<Vec<String>, Box<dyn std::error::Error>>) -> String {
    let result = extensions
        .unwrap()
        .iter()
        .map(|ext| format!(r"((?:\.(?:{})))", regex_escape(ext)))
        .collect::<Vec<String>>()
        .join("|");

    result
}

/// A function that tries to extract the network location of a given URL.
/// This function may be used when you don't really know what kind of dataset
/// you injest. This function will check if the given `data` is a URL by parsing
/// it. If it is not the case, it will just return the given input.
///
/// # Arguments
///
/// * `data` - The presumed data to extract the netloc from.
///
/// # Returns
///
/// A string with the extracted network location.
///
pub fn extract_netloc(data: &String) -> String {
    let parsed_url = urlparse(data);
    let mut result;

    if parsed_url.netloc.is_empty() && !parsed_url.path.is_empty() {
        result = parsed_url.path.as_str()
    } else if !parsed_url.netloc.is_empty() {
        result = parsed_url.netloc.as_str()
    } else {
        result = data.as_str()
    }

    if result.contains("//") {
        result = result.split("//").next().unwrap()
    }

    if result.contains('/') {
        result = result.split('/').next().unwrap()
    }

    result.to_string()
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

    #[test]
    fn test_extract_netloc_empty_str() {
        let given = "".to_string();
        let expected = "".to_string();

        assert_eq!(extract_netloc(&given), expected)
    }

    #[test]
    fn test_extract_netloc_nothing_to_decode() {
        let given = "example.org".to_string();
        let expected = "example.org".to_string();

        assert_eq!(extract_netloc(&given), expected)
    }

    #[test]
    fn test_extract_netloc_full_url() {
        let given = "https://example.org/hello/world/this/is/a/test".to_string();
        let expected = "example.org".to_string();

        assert_eq!(extract_netloc(&given), expected)
    }

    #[test]
    fn test_extract_netloc_full_url_with_port() {
        let given = "https://example.org:8080/hello/world/this/is/a/test".to_string();
        let expected = "example.org:8080".to_string();

        assert_eq!(extract_netloc(&given), expected)
    }

    #[test]
    fn test_extract_netloc_full_url_with_params() {
        let given = "https://example.org/?is_admin=true".to_string();
        let expected = "example.org".to_string();

        assert_eq!(extract_netloc(&given), expected)
    }

    #[test]
    fn test_extract_netloc_url_without_scheme() {
        let given = "example.org/hello/world/this/is/a/test".to_string();
        let expected = "example.org".to_string();

        assert_eq!(extract_netloc(&given), expected)
    }

    #[test]
    fn test_extract_netloc_url_without_scheme_and_with_params() {
        let given = "example.org/?is_admin=true".to_string();
        let expected = "example.org".to_string();

        assert_eq!(extract_netloc(&given), expected)
    }

    #[test]
    fn test_extract_netloc_url_without_protocol() {
        let given = "://example.org/hello/world/this/is/a/test".to_string();
        let expected = "example.org".to_string();

        assert_eq!(extract_netloc(&given), expected)
    }

    #[test]
    fn test_extract_netloc_url_without_protocol_and_with_params() {
        let given = "://example.org/?is_admin=true".to_string();
        let expected = "example.org".to_string();

        assert_eq!(extract_netloc(&given), expected)
    }

    #[test]
    fn test_extract_netloc_url_without_protocol_and_path() {
        let given = "://example.org/".to_string();
        let expected = "example.org".to_string();

        assert_eq!(extract_netloc(&given), expected)
    }

    #[test]
    fn test_extract_netloc_url_startswith_2_slashes() {
        let given = "//example.org/hello/world/this/is/a/test".to_string();
        let expected = "example.org".to_string();

        assert_eq!(extract_netloc(&given), expected)
    }

    #[test]
    fn test_extract_netloc_url_startswith_1_slash() {
        let given = "/example.org/hello/world/this/is/a/test".to_string();
        let expected = "".to_string();

        assert_eq!(extract_netloc(&given), expected)
    }
}
