#![allow(dead_code, unused_imports, unused_mut, unused_variables)]
use flate2::read::GzDecoder;
use reqwest::blocking::Client;
use reqwest::Url;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use crate::config::{project_dir, Config};
use crate::error::{Error, Result};
use crate::utils;

/// StackExchange API v2.2 URL
const SE_API_URL: &str = "http://api.stackexchange.com";
const SE_API_VERSION: &str = "2.2";

/// Filter generated to include only the fields needed to populate
/// the structs below. Go here to make new filters:
/// [create filter](https://api.stackexchange.com/docs/create-filter).
const SE_FILTER: &str = ".DND5X2VHHUH8HyJzpjo)5NvdHI3w6auG";

/// Pagesize when fetching all SE sites. Should be good for many years...
const SE_SITES_PAGESIZE: u16 = 10000;

/// This structure allows interacting with parts of the StackExchange
/// API, using the `Config` struct to determine certain API settings and options.
pub struct StackExchange {
    client: Client,
    config: Config,
}

/// This structure allows interacting with locally cached StackExchange metadata.
pub struct LocalStorage {
    sites: Option<Vec<Site>>,
    filename: PathBuf,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Site {
    pub api_site_parameter: String,
    pub site_url: String,
}

/// Represents a StackExchange answer with a custom selection of fields from
/// the [StackExchange docs](https://api.stackexchange.com/docs/types/answer)
#[derive(Deserialize, Debug)]
pub struct Answer {
    #[serde(rename = "answer_id")]
    pub id: u32,
    pub score: i32,
    #[serde(rename = "body_markdown")]
    pub body: String,
    pub is_accepted: bool,
}

/// Represents a StackExchange question with a custom selection of fields from
/// the [StackExchange docs](https://api.stackexchange.com/docs/types/question)
// TODO let body be a generic that implements Display!
#[derive(Deserialize, Debug)]
pub struct Question {
    #[serde(rename = "question_id")]
    pub id: u32,
    pub score: i32,
    pub answers: Vec<Answer>,
    pub title: String,
    #[serde(rename = "body_markdown")]
    pub body: String,
}

/// Internal struct that represents the boilerplate response wrapper from SE API.
#[derive(Deserialize, Debug)]
struct ResponseWrapper<T> {
    items: Vec<T>,
}

impl StackExchange {
    pub fn new(config: Config) -> Self {
        let client = Client::new();
        StackExchange { client, config }
    }

    /// Search against the search/advanced endpoint with a given query.
    /// Only fetches questions that have at least one answer.
    /// TODO async
    /// TODO parallel requests over multiple sites
    pub fn search(&self, q: &str) -> Result<Vec<Question>> {
        let resp_body = self
            .client
            .get(stackexchange_url("search/advanced"))
            .header("Accepts", "application/json")
            .query(&self.get_default_opts())
            .query(&[
                ("q", q),
                ("pagesize", &self.config.limit.to_string()),
                ("page", "1"),
                ("answers", "1"),
                ("order", "desc"),
                ("sort", "relevance"),
            ])
            .send()?;

        let gz = GzDecoder::new(resp_body);
        let wrapper: ResponseWrapper<Question> = serde_json::from_reader(gz).map_err(|e| {
            Error::StackExchange(format!(
                "Error decoding questions from the StackExchange API: {}",
                e
            ))
        })?;
        let qs = wrapper
            .items
            .into_iter()
            .map(|mut q| {
                q.answers.sort_unstable_by_key(|a| -a.score);
                q
            })
            .collect();
        Ok(qs)
    }

    fn get_default_opts(&self) -> HashMap<&str, &str> {
        let mut params = HashMap::new();
        params.insert("site", self.config.site.as_str());
        params.insert("filter", &SE_FILTER);
        if let Some(key) = &self.config.api_key {
            params.insert("key", key.as_str());
        }
        params
    }
}

impl LocalStorage {
    pub fn new() -> Result<Self> {
        let project = project_dir()?;
        let dir = project.cache_dir();
        fs::create_dir_all(&dir)?;
        Ok(LocalStorage {
            sites: None,
            filename: dir.join("sites.json"),
        })
    }

    // TODO make this async, inform user if we are downloading
    pub fn sites(&mut self) -> Result<&Vec<Site>> {
        // Stop once Option ~ Some or Result ~ Err
        if self.sites.is_none() && !self.fetch_local_sites()? {
            self.fetch_remote_sites()?;
        }
        match &self.sites {
            Some(sites) if sites.is_empty() => Err(Error::EmptySites),
            Some(sites) => Ok(sites),
            None => panic!("Code failure in site listing retrieval"),
        }
    }

    pub fn update_sites(&mut self) -> Result<()> {
        self.fetch_remote_sites()
    }

    pub fn validate_site(&mut self, site_code: &str) -> Result<bool> {
        Ok(self
            .sites()?
            .iter()
            .any(|site| site.api_site_parameter == *site_code))
    }

    fn fetch_local_sites(&mut self) -> Result<bool> {
        match utils::open_file(&self.filename)? {
            Some(file) => {
                self.sites = serde_json::from_reader(file)
                    .map_err(|_| Error::MalformedFile(self.filename.clone()))?;
                Ok(true)
            }
            None => Ok(false),
        }
    }

    // TODO decide whether or not I should give LocalStorage an api key..
    // TODO cool loading animation?
    fn fetch_remote_sites(&mut self) -> Result<()> {
        let resp_body = Client::new()
            .get(stackexchange_url("sites"))
            .header("Accepts", "application/json")
            .query(&[
                ("pagesize", SE_SITES_PAGESIZE.to_string()),
                ("page", "1".to_string()),
            ])
            .send()?;
        let gz = GzDecoder::new(resp_body);
        let wrapper: ResponseWrapper<Site> = serde_json::from_reader(gz).map_err(|e| {
            Error::StackExchange(format!(
                "Error decoding sites from the StackExchange API: {}",
                e
            ))
        })?;
        self.sites = Some(wrapper.items);
        self.store_local_sites()
    }

    fn store_local_sites(&self) -> Result<()> {
        let file = utils::create_file(&self.filename)?;
        Ok(serde_json::to_writer(file, &self.sites)?)
    }
}

/// Creates stackexchange API url given endpoint; can technically panic
fn stackexchange_url(path: &str) -> Url {
    let mut url = Url::parse(SE_API_URL).unwrap();
    url.path_segments_mut()
        .unwrap()
        .push(SE_API_VERSION)
        .extend(path.split('/'));
    url
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_stackexchange_url() {
        assert_eq!(
            stackexchange_url("some/endpoint").as_str(),
            "http://api.stackexchange.com/2.2/some/endpoint"
        )
    }
}
