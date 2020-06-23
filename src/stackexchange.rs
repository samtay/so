use futures::stream::StreamExt;
use percent_encoding::percent_decode_str;
use rayon::prelude::*;
use reqwest::header;
use reqwest::Client;
use reqwest::Url;
use scraper::html::Html;
use scraper::selector::Selector;
use serde::{Deserialize, Serialize};
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use crate::config::{project_dir, Config};
use crate::error::{Error, Result};
use crate::tui::markdown;
use crate::tui::markdown::Markdown;
use crate::utils;

/// DuckDuckGo URL
const DUCKDUCKGO_URL: &str = "https://duckduckgo.com";

/// StackExchange API v2.2 URL
// TODO why not https?
const SE_API_URL: &str = "http://api.stackexchange.com";
const SE_API_VERSION: &str = "2.2";

/// Filter generated to include only the fields needed to populate
/// the structs below. Go here to make new filters:
/// [create filter](https://api.stackexchange.com/docs/create-filter).
const SE_FILTER: &str = ".DND5X2VHHUH8HyJzpjo)5NvdHI3w6auG";

/// Pagesize when fetching all SE sites. Should be good for many years...
const SE_SITES_PAGESIZE: u16 = 10000;

/// Limit on concurrent requests (gets passed to `buffer_unordered`)
const CONCURRENT_REQUESTS_LIMIT: usize = 8;

/// Mock user agent to get real DuckDuckGo results
// TODO copy other user agents and use random one each time
const USER_AGENT: &str =
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10.7; rv:11.0) Gecko/20100101 Firefox/11.0";

/// This structure allows interacting with parts of the StackExchange
/// API, using the `Config` struct to determine certain API settings and options.
// TODO should my se structs have &str instead of String?
#[derive(Clone)]
pub struct StackExchange {
    client: Client,
    config: Config,
    sites: HashMap<String, String>,
    query: String,
}

/// This structure allows interacting with locally cached StackExchange metadata.
pub struct LocalStorage {
    pub sites: Vec<Site>,
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
    pub fn new(config: Config, local_storage: LocalStorage, query: String) -> Self {
        let client = Client::new();
        StackExchange {
            client,
            sites: local_storage.get_urls(&config.sites),
            config,
            query,
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
        if !self.config.duckduckgo {
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

    /// Search query and get a list of relevant questions
    pub async fn search(&self) -> Result<Vec<Question<String>>> {
        if self.config.duckduckgo {
            self.search_duckduck_go().await
        } else {
            // TODO after duckduck go finished, refactor to _not_ thread this limit, its unnecessary
            self.se_search_advanced(self.config.limit).await
        }
    }

    /// Search query at duckduckgo and then fetch the resulting questions from SE.
    async fn search_duckduck_go(&self) -> Result<Vec<Question<String>>> {
        let url = duckduckgo_url(&self.query, self.sites.values());
        let html = self
            .client
            .get(url)
            .header(header::USER_AGENT, USER_AGENT)
            .send()
            .await?
            .text()
            .await?;
        let ids = parse_questions_from_ddg_html(&html, &self.sites, self.config.limit)?;
        self.se_questions(ids).await
    }

    /// Parallel searches against the SE question endpoint across the sites in `ids`.
    // TODO I'm sure there is a way to DRY the se_question & se_search_advanced functions
    async fn se_questions(
        &self,
        ids: HashMap<String, Vec<String>>,
    ) -> Result<Vec<Question<String>>> {
        futures::stream::iter(ids)
            .map(|(site, ids)| {
                let clone = self.clone();
                tokio::spawn(async move {
                    let clone = &clone;
                    clone.se_questions_site(&site, ids).await
                })
            })
            .buffer_unordered(CONCURRENT_REQUESTS_LIMIT)
            .collect::<Vec<_>>()
            .await
            .into_iter()
            .map(|r| r.map_err(Error::from).and_then(|x| x))
            .collect::<Result<Vec<Vec<_>>>>()
            .map(|v| {
                let qs: Vec<Question<String>> = v.into_iter().flatten().collect();
                // TODO sort by original ordering !
                qs
            })
    }

    /// Parallel searches against the SE search/advanced endpoint across all configured sites
    async fn se_search_advanced(&self, limit: u16) -> Result<Vec<Question<String>>> {
        futures::stream::iter(self.config.sites.clone())
            .map(|site| {
                let clone = self.clone();
                tokio::spawn(async move {
                    let clone = &clone;
                    clone.se_search_advanced_site(&site, limit).await
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

    /// Search against the SE site's /questions/{ids} endpoint.
    /// Filters out questions with no answers.
    async fn se_questions_site(
        &self,
        site: &str,
        ids: Vec<String>,
    ) -> Result<Vec<Question<String>>> {
        let total = ids.len().to_string();
        let endpoint = format!("questions/{ids}", ids = ids.join(";"));
        let qs = self
            .client
            .get(stackexchange_url(&endpoint))
            .header("Accepts", "application/json")
            .query(&self.get_default_se_opts())
            .query(&[("site", site), ("pagesize", &total), ("page", "1")])
            .send()
            .await?
            .json::<ResponseWrapper<Question<String>>>()
            .await?
            .items;
        Ok(Self::preprocess(qs))
    }

    /// Search against the SE site's /search/advanced endpoint with a given query.
    /// Only fetches questions that have at least one answer.
    async fn se_search_advanced_site(
        &self,
        site: &str,
        limit: u16,
    ) -> Result<Vec<Question<String>>> {
        let qs = self
            .client
            .get(stackexchange_url("search/advanced"))
            .header("Accepts", "application/json")
            .query(&self.get_default_se_opts())
            .query(&[
                ("q", self.query.as_str()),
                ("pagesize", &limit.to_string()),
                ("site", site),
                ("page", "1"),
                ("answers", "1"),
                ("order", "desc"),
                ("sort", "relevance"),
            ])
            .send()
            .await?
            .json::<ResponseWrapper<Question<String>>>()
            .await?
            .items;
        Ok(Self::preprocess(qs))
    }

    fn get_default_se_opts(&self) -> HashMap<&str, &str> {
        let mut params = HashMap::new();
        params.insert("filter", SE_FILTER);
        if let Some(key) = &self.config.api_key {
            params.insert("key", &key);
        }
        params
    }

    /// Sorts answers by score
    /// Preprocess SE markdown to "cmark" markdown (or something closer to it)
    /// This markdown preprocess _always_ happens.
    fn preprocess(qs: Vec<Question<String>>) -> Vec<Question<String>> {
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
                    body: markdown::preprocess(q.body),
                    ..q
                }
            })
            .collect::<Vec<_>>()
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

impl LocalStorage {
    fn fetch_local_sites(filename: &PathBuf) -> Result<Option<Vec<Site>>> {
        if let Some(file) = utils::open_file(filename)? {
            return serde_json::from_reader(file)
                .map_err(|_| Error::MalformedFile(filename.clone()));
        }
        Ok(None)
    }

    // TODO decide whether or not I should give LocalStorage an api key..
    async fn fetch_remote_sites() -> Result<Vec<Site>> {
        let se_sites = Client::new()
            .get(stackexchange_url("sites"))
            .header("Accepts", "application/json")
            .query(&[
                ("pagesize", SE_SITES_PAGESIZE.to_string()),
                ("page", "1".to_string()),
            ])
            .send()
            .await?
            .json::<ResponseWrapper<Site>>()
            .await?
            .items;
        Ok(se_sites
            .into_par_iter()
            .map(|site| {
                let site_url = site.site_url.trim_start_matches("https://").to_string();
                Site { site_url, ..site }
            })
            .collect())
    }

    fn store_local_sites(filename: &PathBuf, sites: &[Site]) -> Result<()> {
        let file = utils::create_file(filename)?;
        serde_json::to_writer(file, sites)?;
        Ok(())
    }

    async fn init_sites(filename: &PathBuf, update: bool) -> Result<Vec<Site>> {
        if !update {
            if let Some(sites) = Self::fetch_local_sites(filename)? {
                return Ok(sites);
            }
        }
        let sites = Self::fetch_remote_sites().await?;
        Self::store_local_sites(filename, &sites)?;
        Ok(sites)
    }

    pub async fn new(update: bool) -> Result<Self> {
        let project = project_dir()?;
        let dir = project.cache_dir();
        fs::create_dir_all(&dir)?;
        let sites_filename = dir.join("sites.json");
        let sites = Self::init_sites(&sites_filename, update).await?;
        Ok(LocalStorage { sites })
    }

    // TODO is this HM worth it? Probably only will ever have < 10 site codes to search...
    // TODO store this as Option<HM> on self if other methods use it...
    pub async fn find_invalid_site<'a, 'b>(
        &'b self,
        site_codes: &'a [String],
    ) -> Option<&'a String> {
        let hm: HashMap<&str, ()> = self
            .sites
            .iter()
            .map(|site| (site.api_site_parameter.as_str(), ()))
            .collect();
        site_codes.iter().find(|s| !hm.contains_key(&s.as_str()))
    }

    pub fn get_urls(&self, site_codes: &[String]) -> HashMap<String, String> {
        self.sites
            .iter()
            .filter_map(move |site| {
                let _ = site_codes
                    .iter()
                    .find(|&sc| *sc == site.api_site_parameter)?;
                Some((site.api_site_parameter.to_owned(), site.site_url.to_owned()))
            })
            .collect()
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

/// Creates duckduckgo search url given sites and query
/// See https://duckduckgo.com/params for more info
fn duckduckgo_url<'a, I>(query: &str, sites: I) -> Url
where
    I: IntoIterator<Item = &'a String>,
{
    let mut q = String::new();
    //  Restrict to sites
    q.push('(');
    q.push_str(
        sites
            .into_iter()
            .map(|site| String::from("site:") + site)
            .collect::<Vec<_>>()
            .join(" OR ")
            .as_str(),
    );
    q.push_str(") ");
    //  Search terms
    q.push_str(
        query
            .trim_end_matches('?')
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
            .as_str(),
    );
    Url::parse_with_params(
        DUCKDUCKGO_URL,
        &[("q", q.as_str()), ("kz", "-1"), ("kh", "-1")],
    )
    .unwrap()
}

/// Parse (site, question_id) pairs out of duckduckgo search results html
/// TODO currently hashmap {site: [qids]} BUT we should maintain relevance order !
///      maybe this is as simple as a HashMap {qid: ordinal}
fn parse_questions_from_ddg_html<'a>(
    html: &'a str,
    sites: &'a HashMap<String, String>,
    limit: u16,
) -> Result<HashMap<String, Vec<String>>> {
    let fragment = Html::parse_document(html);
    let anchors = Selector::parse("a.result__a").unwrap();
    let mut qids: HashMap<String, Vec<String>> = HashMap::new();
    let mut count = 0;
    for anchor in fragment.select(&anchors) {
        let url = anchor
            .value()
            .attr("href")
            .ok_or_else(|| Error::ScrapingError("Anchor with no href".to_string()))
            .map(|href| percent_decode_str(href).decode_utf8_lossy().into_owned())?;
        sites
            .iter()
            .find_map(|(site_code, site_url)| {
                let id = question_url_to_id(site_url, &url)?;
                match qids.entry(site_code.to_owned()) {
                    Entry::Occupied(mut o) => o.get_mut().push(id),
                    Entry::Vacant(o) => {
                        o.insert(vec![id]);
                    }
                }
                count += 1;
                Some(())
            })
            .ok_or_else(|| {
                Error::ScrapingError(
                    "Duckduckgo returned results outside of SE network".to_string(),
                )
            })?;
        if count >= limit as usize {
            break;
        }
    }
    // It doesn't seem possible for DDG to return no results, so assume this is
    // a bad user agent
    if count == 0 {
        Err(Error::ScrapingError(String::from(
            "DuckDuckGo blocked this request",
        )))
    } else {
        Ok(qids)
    }
}

/// For example
/// ```
/// let id = "stackoverflow.com";
/// let input = "/l/?kh=-1&uddg=https://stackoverflow.com/questions/11828270/how-do-i-exit-the-vim-editor";
/// assert_eq!(question_url_to_id(site_url, input), "11828270")
/// ```
fn question_url_to_id(site_url: &str, input: &str) -> Option<String> {
    // TODO use str_prefix once its stable
    let fragment = site_url.trim_end_matches('/').to_owned() + "/questions/";
    let ix = input.find(&fragment)? + fragment.len();
    let input = &input[ix..];
    let end = input.find('/')?;
    Some(input[0..end].to_string())
}

// TODO figure out a query that returns no results so that I can test it and differentiate it from
// a blocked request
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

    #[test]
    fn test_duckduckgo_url() {
        let q = "how do I exit vim?";
        let sites = vec![
            String::from("stackoverflow.com"),
            String::from("unix.stackexchange.com"),
        ];
        assert_eq!(
            duckduckgo_url(q, &sites).as_str(),
            String::from(
                "https://duckduckgo.com/\
                ?q=%28site%3Astackoverflow.com+OR+site%3Aunix.stackexchange.com%29\
                +how+do+I+exit+vim&kz=-1&kh=-1"
            )
        )
    }

    #[test]
    fn test_duckduckgo_response() {
        // TODO make sure results are either 1) answers 2) failed connection 3) blocked
    }

    #[test]
    fn test_duckduckgo_parser() {
        let html = include_str!("../test/exit-vim.html");
        let sites = vec![
            ("stackoverflow", "stackoverflow.com"),
            ("askubuntu", "askubuntu.com"),
        ]
        .into_iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect::<HashMap<String, String>>();
        let mut expected_question_ids = HashMap::new();
        expected_question_ids.insert(
            "stackoverflow".to_string(),
            vec!["11828270".to_string(), "9171356".to_string()],
        );
        expected_question_ids.insert("askubuntu".to_string(), vec!["24406".to_string()]);
        assert_eq!(
            parse_questions_from_ddg_html(html, &sites, 3).unwrap(),
            expected_question_ids
        );
    }

    #[test]
    fn test_duckduckgo_blocker() -> Result<(), String> {
        let html = include_str!("../test/bad-user-agent.html");
        let mut sites = HashMap::new();
        sites.insert(
            String::from("stackoverflow"),
            String::from("stackoverflow.com"),
        );

        match parse_questions_from_ddg_html(html, &sites, 2) {
            Err(Error::ScrapingError(s)) if s == "DuckDuckGo blocked this request".to_string() => {
                Ok(())
            }
            _ => Err(String::from("Failed to detect DuckDuckGo blocker")),
        }
    }

    #[test]
    fn test_question_url_to_id() {
        let site_url = "stackoverflow.com";
        let input = "/l/?kh=-1&uddg=https://stackoverflow.com/questions/11828270/how-do-i-exit-the-vim-editor";
        assert_eq!(question_url_to_id(site_url, input).unwrap(), "11828270");

        let site_url = "stackoverflow.com";
        let input = "/l/?kh=-1&uddg=https://askubuntu.com/questions/24406/how-to-close-vim-from-the-command-line";
        assert_eq!(question_url_to_id(site_url, input), None);
    }
}
