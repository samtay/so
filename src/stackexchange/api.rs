use rayon::prelude::*;
use reqwest::header;
use reqwest::Client;
use reqwest::Url;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::error::Result;
use crate::tui::markdown;

/// StackExchange API v2.2 URL
const SE_API_URL: &str = "https://api.stackexchange.com";
const SE_API_VERSION: &str = "2.2";

/// Filter generated to include only the fields needed to populate
/// the structs below. Go here to make new filters:
/// [create filter](https://api.stackexchange.com/docs/create-filter).
const SE_FILTER: &str = ".DND5X2VHHUH8HyJzpjo)5NvdHI3w6auG";

/// Pagesize when fetching all SE sites. Should be good for many years...
const SE_SITES_PAGESIZE: u16 = 10000;

pub type Id = u32;

/// Represents a StackExchange answer with a custom selection of fields from
/// the [StackExchange docs](https://api.stackexchange.com/docs/types/answer)
#[derive(Clone, Deserialize, Debug)]
pub struct Answer<S> {
    #[serde(rename = "answer_id")]
    pub id: Id,
    pub score: i32,
    #[serde(rename = "body_markdown")]
    pub body: S,
    pub is_accepted: bool,
}

/// Represents a StackExchange question with a custom selection of fields from
/// the [StackExchange docs](https://api.stackexchange.com/docs/types/question)
#[derive(Clone, Deserialize, Debug)]
pub struct Question<S> {
    #[serde(rename = "question_id")]
    pub id: Id,
    pub score: i32,
    // N.B. empty vector default needed because questions endpoint cannot filter
    // answers >= 1
    #[serde(default = "Vec::new")]
    pub answers: Vec<Answer<S>>,
    pub title: String,
    #[serde(rename = "body_markdown")]
    pub body: S,
    // This is the only field that doesn't actually come back from SE; we add
    // this site code to which the question belongs
    pub site: Option<String>,
}

/// Internal struct that represents the boilerplate response wrapper from SE API.
#[derive(Deserialize, Debug)]
struct ResponseWrapper<T> {
    items: Vec<T>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Site {
    pub api_site_parameter: String,
    pub site_url: String,
}

#[derive(Debug, Clone)]
pub struct Api {
    client: Client,
    api_key: Option<String>,
}

impl Api {
    pub fn new(api_key: Option<String>) -> Self {
        // TODO can lazy_static this above
        let mut headers = header::HeaderMap::new();
        headers.insert(
            header::ACCEPT,
            header::HeaderValue::from_static("application/json"),
        );
        headers.insert(
            header::USER_AGENT,
            header::HeaderValue::from_static(super::USER_AGENT),
        );
        let client = Client::builder().default_headers(headers).build().unwrap();
        Api { client, api_key }
    }

    /// Search against the SE site's /questions/{ids} endpoint.
    /// Filters out questions with no answers.
    pub async fn questions(&self, site: &str, ids: Vec<String>) -> Result<Vec<Question<String>>> {
        let total = ids.len().to_string();
        let endpoint = format!("questions/{ids}", ids = ids.join(";"));
        let url = stackexchange_url(&endpoint);
        log::debug!("Fetching questions from: {url}");
        let qs_rsp = self
            .client
            .get(url)
            .query(&self.get_default_se_opts())
            .query(&[("site", site), ("pagesize", &total)])
            .send()
            .await?;
        let status_code = qs_rsp.status();
        let body = qs_rsp.text().await?;
        log::debug!("Stack exchange returned status {status_code} and body {body}");
        let qs = serde_json::from_str::<ResponseWrapper<Question<String>>>(&body)?
            .items
            .into_iter()
            .filter(|q| !q.answers.is_empty())
            .collect();
        Ok(Self::preprocess(site, qs))
    }

    /// Search against the SE site's /search/advanced endpoint with a given query.
    /// Only fetches questions that have at least one answer.
    pub async fn search_advanced(
        &self,
        query: &str,
        site: &str,
        limit: u16,
    ) -> Result<Vec<Question<String>>> {
        let qs = self
            .client
            .get(stackexchange_url("search/advanced"))
            .query(&self.get_default_se_opts())
            .query(&[
                ("q", query),
                ("pagesize", &limit.to_string()),
                ("site", site),
                ("answers", "1"),
                ("order", "desc"),
                ("sort", "relevance"),
            ])
            .send()
            .await?
            .json::<ResponseWrapper<Question<String>>>()
            .await?
            .items;
        Ok(Self::preprocess(site, qs))
    }

    pub async fn sites(&self) -> Result<Vec<Site>> {
        let sites = self
            .client
            .get(stackexchange_url("sites"))
            .query(&[("pagesize", SE_SITES_PAGESIZE.to_string())])
            .send()
            .await?
            .json::<ResponseWrapper<Site>>()
            .await?
            .items;
        Ok(sites
            .into_par_iter()
            .map(|site| {
                let site_url = site.site_url.trim_start_matches("https://").to_string();
                Site { site_url, ..site }
            })
            .collect())
    }

    fn get_default_se_opts(&self) -> HashMap<&str, &str> {
        let mut params = HashMap::new();
        params.insert("filter", SE_FILTER);
        params.insert("page", "1");
        if let Some(key) = &self.api_key {
            params.insert("key", key);
        }
        params
    }

    /// Sorts answers by score
    /// Add the site code to which the question belongs
    /// Preprocess SE markdown to "cmark" markdown (or something closer to it)
    /// This markdown preprocess _always_ happens.
    fn preprocess(site: &str, qs: Vec<Question<String>>) -> Vec<Question<String>> {
        qs.into_par_iter()
            .map(|q| {
                let mut answers = q.answers;
                answers.par_sort_unstable_by_key(|a| -a.score);
                let answers = answers
                    .into_par_iter()
                    .map(|a| Answer {
                        body: markdown::preprocess(a.body.clone()),
                        ..a
                    })
                    .collect();
                Question {
                    answers,
                    site: Some(site.to_string()),
                    body: markdown::preprocess(q.body),
                    ..q
                }
            })
            .collect::<Vec<_>>()
    }
}

/// Creates stackexchange API url given endpoint
// TODO lazy static this url parse
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
