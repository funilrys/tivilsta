// Tivilsta - A different whitelisting mechanism
//
// Author:
//      Nissar Chababy, @funilrys, contactTATAfunilrysTODTODcom
//
// License:
//      Copyright (c) 2022, 2023, 2024 Nissar Chababy
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

#![allow(dead_code)]

use serde_json::Value;
use std::collections::HashMap;

use crate::utils;

/// Fetches the PSL registry of the PyFunceble project and provide the `reqwest` response
/// for other to use.
fn fetch_mapping() -> Result<reqwest::blocking::Response, Box<dyn std::error::Error>> {
    utils::fetch_url(
        &String::from(
            "https://raw.githubusercontent.com/PyFunceble/public-suffix/master/public-suffix.json",
        ),
        "Failed to fetch PSL. Is GitHub down?".to_string(),
    )
}

/// Fetches the PSL registry of the PyFunceble project, parse it and return
/// all known TLDs.
pub fn extensions() -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let response: Value = fetch_mapping()?.json()?;
    let mut result: Vec<String> = Vec::new();

    for (extension, _) in response.as_object().unwrap() {
        result.push(extension.to_string());
    }
    Ok(result)
}

/// Fetches the PSL registry of the PyFunceble project, parse it and return
/// all known public suffixes.
pub fn suffixes() -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let response: Value = fetch_mapping()?.json()?;
    let mut result: Vec<String> = Vec::new();

    for (_, suffixes) in response.as_object().unwrap() {
        for suffix in suffixes.as_array().unwrap() {
            result.push(suffix.as_str().unwrap().to_string());
        }
    }
    Ok(result)
}

/// Get all known suffixes and parse them as a huge Regex String with the following
/// format:
///
/// ```txt
/// ((?:\.(?:\.co\.uk)))|((?:\.(?:de.example)))
/// ```
///
/// Where `co.uk` and `de.example` are suffixes.
pub fn suffixes_regex_string() -> String {
    utils::to_regex_string(suffixes())
}

/// Get all known suffixes and parse them as a huge Regex String with the following
/// format:
///
/// ```txt
/// ((?:\.(?:com)))|((?:\.(?:de)))
/// ```
///
/// Where `com` and `de` are TLDs.
pub fn extensions_regex_string() -> String {
    utils::to_regex_string(extensions())
}

/// Read the IANA registry of the PyFunceble project and return all known
/// TLDs and their whois server.
///
/// The registry is a JSON file that has the following format:
///
/// ```json
/// {
///     "com": {
///         "xx.com",
///         "xy.com"
///     }
/// }
/// ```
///
/// Where `com` is the Top Level Domain (TlD) and `xx.com`+`xy.com` public suffixes.

pub fn extensions_and_suffixes() -> Result<HashMap<String, Vec<String>>, Box<dyn std::error::Error>>
{
    let response: Value = fetch_mapping()?.json()?;
    let mut result: HashMap<String, Vec<String>> = HashMap::new();

    for (extension, suffixes) in response.as_object().unwrap() {
        result.insert(
            extension.to_string(),
            suffixes
                .as_array()
                .unwrap()
                .iter()
                .map(|suffix| suffix.as_str().unwrap().to_string())
                .collect(),
        );
    }
    Ok(result)
}
