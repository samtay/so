use futures::stream::StreamExt;
use rayon::prelude::*;
use reqwest::header;
use reqwest::Client;
use std::collections::HashMap;

use crate::config::{Config, SearchEngine};
use crate::error::{Error, Result};
use crate::tui::markdown;
use crate::tui::markdown::Markdown;

use super::api::{Answer, Api, Question};
use super::local_storage::LocalStorage;
use super::scraper::{DuckDuckGo, Google, ScrapedData, Scraper};

/// Limit on concurrent requests (gets passed to `buffer_unordered`)
const CONCURRENT_REQUESTS_LIMIT: usize = 8;

/// Mock user agent to get real DuckDuckGo results
// TODO copy other user agents and use random one each time
const USER_AGENT: &str =
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10.7; rv:11.0) Gecko/20100101 Firefox/11.0";

/// This structure provides methods to search queries and get StackExchange
/// questions/answers in return.
// TODO this really needs a better name...
#[derive(Clone)]
pub struct Search {
    api: Api,
    config: Config,
    query: String,
    sites: HashMap<String, String>,
}

impl Search {
    pub fn new(config: Config, local_storage: LocalStorage, query: String) -> Self {
        let api = Api::new(config.api_key.clone());
        let sites = local_storage.get_urls(&config.sites);
        Search {
            api,
            config,
            query,
            sites,
        }
    }

    /// Search query and get the top answer body
    ///
    /// For StackExchange engine, use only the first configured site,
    /// since, parodoxically, sites with the worst results will finish
    /// executing first, because there's less data to retrieve.
    ///
    /// Needs mut because it temporarily changes self.config
    pub async fn search_lucky(&mut self) -> Result<String> {
        let original_config = self.config.clone();
        // Temp set lucky config
        self.config.limit = 1;
        if let SearchEngine::StackExchange = self.config.search_engine {
            self.config.sites.truncate(1);
        }
        // Run search with temp config
        let result = self.search().await;
        // Reset config
        self.config = original_config;

        Ok(result?
            .into_iter()
            .next()
            .ok_or(Error::NoResults)?
            .answers
            .into_iter()
            .next()
            .ok_or_else(|| Error::StackExchange(String::from("Received question with no answers")))?
            .body)
    }

    /// Search and parse to Markdown for TUI
    pub async fn search_md(&self) -> Result<Vec<Question<Markdown>>> {
        Ok(parse_markdown(self.search().await?))
    }

    /// Search using the configured search engine
    pub async fn search(&self) -> Result<Vec<Question<String>>> {
        match self.config.search_engine {
            SearchEngine::DuckDuckGo => self.search_by_scraper(DuckDuckGo).await,
            SearchEngine::Google => self.search_by_scraper(Google).await,
            SearchEngine::StackExchange => self.parallel_search_advanced().await,
        }
        .and_then(|qs| {
            if qs.is_empty() {
                Err(Error::NoResults)
            } else {
                Ok(qs)
            }
        })
    }

    /// Search query at duckduckgo and then fetch the resulting questions from SE.
    async fn search_by_scraper(&self, scraper: impl Scraper) -> Result<Vec<Question<String>>> {
        let url = scraper.get_url(&self.query, self.sites.values());
        let html = Client::new()
            .get(url)
            .header(header::USER_AGENT, USER_AGENT)
            .send()
            .await?
            .text()
            .await?;
        let data = scraper.parse(&html, &self.sites, self.config.limit)?;
        self.parallel_questions(data).await
    }

    /// Parallel requests against the SE question endpoint across all sites in data.
    // TODO I'm sure there is a way to DRY the following two functions
    async fn parallel_questions(&self, data: ScrapedData) -> Result<Vec<Question<String>>> {
        let ScrapedData {
            question_ids,
            ordering,
        } = data;
        futures::stream::iter(question_ids)
            .map(|(site, ids)| {
                let api = self.api.clone();
                tokio::spawn(async move {
                    let api = &api;
                    api.questions(&site, ids).await
                })
            })
            .buffer_unordered(CONCURRENT_REQUESTS_LIMIT)
            .collect::<Vec<_>>()
            .await
            .into_iter()
            .map(|r| r.map_err(Error::from).and_then(|x| x))
            .collect::<Result<Vec<Vec<_>>>>()
            .map(|v| {
                let mut qs: Vec<Question<String>> = v.into_iter().flatten().collect();
                qs.sort_unstable_by_key(|q| ordering.get(&q.id.to_string()).unwrap());
                qs
            })
    }

    /// Parallel requests against the SE search/advanced endpoint across all configured sites
    async fn parallel_search_advanced(&self) -> Result<Vec<Question<String>>> {
        futures::stream::iter(self.config.sites.clone())
            .map(|site| {
                let api = self.api.clone();
                let limit = self.config.limit;
                let query = self.query.clone();
                tokio::spawn(async move {
                    let api = &api;
                    api.search_advanced(&query, &site, limit).await
                })
            })
            .buffer_unordered(CONCURRENT_REQUESTS_LIMIT)
            .collect::<Vec<_>>()
            .await
            .into_iter()
            .map(|r| r.map_err(Error::from).and_then(|x| x))
            .collect::<Result<Vec<Vec<_>>>>()
            .map(|v| {
                let mut qs: Vec<Question<String>> = v.into_iter().flatten().collect();
                if self.config.sites.len() > 1 {
                    qs.sort_unstable_by_key(|q| -q.score);
                }
                qs
            })
    }
}

/// Parse all markdown fields
/// This only happens for content going into the cursive TUI (not lucky prompt)
fn parse_markdown(qs: Vec<Question<String>>) -> Vec<Question<Markdown>> {
    qs.into_par_iter()
        .map(|q| {
            let body = markdown::parse(q.body);
            let answers = q
                .answers
                .into_par_iter()
                .map(|a| {
                    let body = markdown::parse(a.body);
                    Answer {
                        body,
                        id: a.id,
                        score: a.score,
                        is_accepted: a.is_accepted,
                    }
                })
                .collect::<Vec<_>>();
            Question {
                body,
                answers,
                id: q.id,
                score: q.score,
                title: q.title,
            }
        })
        .collect::<Vec<_>>()
}

// TODO find a query that returns no results so that I can test it and
// differentiate it from a blocked request
#[cfg(test)]
mod tests {

    #[test]
    fn test_duckduckgo_response() {
        // TODO make sure results are either 1) answers 2) failed connection 3) blocked
    }
}
