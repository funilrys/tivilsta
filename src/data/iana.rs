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

#![allow(dead_code)]

use serde_json::Value;
use std::collections::HashMap;

use crate::utils;

fn fetch_mapping() -> Result<reqwest::blocking::Response, Box<dyn std::error::Error>> {
    utils::fetch_json(
        String::from(
            "https://raw.githubusercontent.com/PyFunceble/iana/master/iana-domains-db.json",
        ),
        String::from("Failed to fetch IANA extensions. Is GitHub down?"),
    )
}

pub fn extensions() -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let response: Value = fetch_mapping()?.json()?;
    let mut result: Vec<String> = Vec::new();

    for (key, _) in response.as_object().unwrap() {
        result.push(key.to_string());
    }
    Ok(result)
}

pub fn extensions_regex_string() -> String {
    utils::to_regex_string(extensions())
}

pub fn extensions_and_whois() -> Result<HashMap<String, Option<String>>, Box<dyn std::error::Error>>
{
    let response: Value = fetch_mapping()?.json()?;
    let mut result: HashMap<String, Option<String>> = HashMap::new();

    for (key, value) in response.as_object().unwrap() {
        result.insert(
            key.to_string(),
            Some(value.as_str().unwrap_or("").to_string()),
        );
    }
    Ok(result)
}
