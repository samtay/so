use anyhow;
use flate2::read::GzDecoder;
use reqwest::blocking::Client;
use reqwest::Url;
use serde::Deserialize;
use std::collections::HashMap;

use crate::config::Config;

const SE_URL: &str = "http://api.stackexchange.com/2.2/";

/// This structure allows intercting with parts of the StackExchange
/// API, using the `Config` struct to determine certain API settings and options.
pub struct StackExchange {
    client: Client,
    config: Config,
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
struct ResponseWrapper {
    items: Vec<Question>,
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
    pub fn search(&self, q: &str) -> Result<Vec<Question>, anyhow::Error> {
        let resp_body = self
            .client
            .get(stackechange_url("search/advanced"))
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
        let wrapper: ResponseWrapper = serde_json::from_reader(gz)?;
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

    fn get_default_opts(&self) -> HashMap<&str, &String> {
        let mut params = HashMap::new();
        params.insert("filter", &self.config.filter);
        params.insert("site", &self.config.site);
        params.insert("key", &self.config.api_key);
        params
    }
}

/// Creates url from const string; can technically panic
fn stackechange_url(path: &str) -> Url {
    let mut url = Url::parse(SE_URL).unwrap();
    url.set_path(path);
    url
}

#[cfg(test)]
mod tests {
    // TODO for both, detect situation and print helpful error message
    #[test]
    fn test_invalid_api_key() {
        assert!(true)
    }
    #[test]
    fn test_invalid_filter() {
        assert!(true)
    }
}
