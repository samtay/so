use rayon::prelude::*;
use reqwest::blocking::Client;
use reqwest::Url;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use crate::config::{project_dir, Config};
use crate::error::{Error, Result};
use crate::tui::markdown;
use crate::tui::markdown::Markdown;
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
// TODO should my se structs have &str instead of String?
#[derive(Clone)]
pub struct StackExchange {
    client: Client,
    config: Config,
    query: String,
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
#[derive(Clone, Deserialize, Debug)]
pub struct Answer<S> {
    #[serde(rename = "answer_id")]
    pub id: u32,
    pub score: i32,
    #[serde(rename = "body_markdown")]
    pub body: S,
    pub is_accepted: bool,
}

/// Represents a StackExchange question with a custom selection of fields from
/// the [StackExchange docs](https://api.stackexchange.com/docs/types/question)
// TODO container over answers should be generic iterator
// TODO let body be a generic that implements Display!
#[derive(Clone, Deserialize, Debug)]
pub struct Question<S> {
    #[serde(rename = "question_id")]
    pub id: u32,
    pub score: i32,
    pub answers: Vec<Answer<S>>,
    pub title: String,
    #[serde(rename = "body_markdown")]
    pub body: S,
}

/// Internal struct that represents the boilerplate response wrapper from SE API.
#[derive(Deserialize, Debug)]
struct ResponseWrapper<T> {
    items: Vec<T>,
}

impl StackExchange {
    pub fn new(config: Config, query: String) -> Self {
        let client = Client::new();
        StackExchange {
            client,
            config,
            query,
        }
    }

    /// Search query at stack exchange and get the top answer body
    ///
    /// For now, use only the first configured site, since, parodoxically, sites
    /// with the worst results will finish executing first, since there's less
    /// data to retrieve.
    pub fn search_lucky(&self) -> Result<String> {
        Ok(self
            .search_advanced_site(self.config.sites.iter().next().unwrap(), 1)?
            .into_iter()
            .next()
            .ok_or(Error::NoResults)?
            .answers
            .into_iter()
            .next()
            .ok_or_else(|| Error::StackExchange(String::from("Received question with no answers")))?
            .body)
    }

    /// Search query at stack exchange and get a list of relevant questions
    pub fn search(&self) -> Result<Vec<Question<Markdown>>> {
        self.search_advanced(self.config.limit)
    }

    /// Parallel searches against the search/advanced endpoint across all configured sites
    fn search_advanced(&self, limit: u16) -> Result<Vec<Question<Markdown>>> {
        self.config
            .sites
            .iter()
            .map(|site| self.search_advanced_site(&site, limit))
            .collect::<Result<Vec<_>>>()
            .map(|v| {
                let mut qs: Vec<Question<String>> = v.into_iter().flatten().collect();
                if self.config.sites.len() > 1 {
                    qs.sort_unstable_by_key(|q| -q.score);
                }
                Self::parse_markdown(qs)
            })
    }

    /// Search against the site's search/advanced endpoint with a given query.
    /// Only fetches questions that have at least one answer.
    fn search_advanced_site(&self, site: &str, limit: u16) -> Result<Vec<Question<String>>> {
        let qs = self
            .client
            .get(stackexchange_url("search/advanced"))
            .header("Accepts", "application/json")
            .query(&self.get_default_opts())
            .query(&[
                ("q", self.query.as_str()),
                ("pagesize", &limit.to_string()),
                ("site", site),
                ("page", "1"),
                ("answers", "1"),
                ("order", "desc"),
                ("sort", "relevance"),
            ])
            .send()?
            .json::<ResponseWrapper<Question<String>>>()?
            .items;
        Ok(Self::preprocess(qs))
    }

    fn get_default_opts(&self) -> HashMap<&str, &str> {
        let mut params = HashMap::new();
        params.insert("filter", SE_FILTER);
        if let Some(key) = &self.config.api_key {
            params.insert("key", &key);
        }
        params
    }

    /// Sorts answers by score
    /// Preprocess SE markdown to "cmark" markdown (or something closer to it)
    fn preprocess(qs: Vec<Question<String>>) -> Vec<Question<String>> {
        qs.par_iter()
            .map(|q| {
                let Question {
                    id,
                    score,
                    title,
                    answers,
                    body,
                } = q;
                answers.to_vec().par_sort_unstable_by_key(|a| -a.score);
                let answers = answers
                    .par_iter()
                    .map(|a| Answer {
                        body: markdown::preprocess(a.body.clone()),
                        ..*a
                    })
                    .collect();
                Question {
                    answers,
                    body: markdown::preprocess(body.to_string()),
                    id: *id,
                    score: *score,
                    title: title.to_string(),
                }
            })
            .collect::<Vec<_>>()
    }

    /// Parse all markdown fields
    fn parse_markdown(qs: Vec<Question<String>>) -> Vec<Question<Markdown>> {
        qs.par_iter()
            .map(|q| {
                let Question {
                    id,
                    score,
                    title,
                    answers,
                    body,
                } = q;
                let body = markdown::parse(body);
                let answers = answers
                    .par_iter()
                    .map(|a| {
                        let Answer {
                            id,
                            score,
                            is_accepted,
                            body,
                        } = a;
                        let body = markdown::parse(body);
                        Answer {
                            body,
                            id: *id,
                            score: *score,
                            is_accepted: *is_accepted,
                        }
                    })
                    .collect::<Vec<_>>();
                Question {
                    body,
                    answers,
                    id: *id,
                    score: *score,
                    title: title.to_string(),
                }
            })
            .collect::<Vec<_>>()
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

    // TODO inform user if we are downloading
    pub fn sites(&mut self) -> Result<&Vec<Site>> {
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

    // TODO is this HM worth it? Probably only will ever have < 10 site codes to search...
    pub fn find_invalid_site<'a, 'b>(
        &'b mut self,
        site_codes: &'a [String],
    ) -> Result<Option<&'a String>> {
        let hm: HashMap<&str, ()> = self
            .sites()?
            .iter()
            .map(|site| (site.api_site_parameter.as_str(), ()))
            .collect();
        Ok(site_codes.iter().find(|s| !hm.contains_key(&s.as_str())))
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
    fn fetch_remote_sites(&mut self) -> Result<()> {
        self.sites = Some(
            Client::new()
                .get(stackexchange_url("sites"))
                .header("Accepts", "application/json")
                .query(&[
                    ("pagesize", SE_SITES_PAGESIZE.to_string()),
                    ("page", "1".to_string()),
                ])
                .send()?
                .json::<ResponseWrapper<Site>>()?
                .items,
        );
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
