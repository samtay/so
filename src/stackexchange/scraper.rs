use percent_encoding::percent_decode_str;
use reqwest::Url;
use scraper::html::Html;
use scraper::selector::Selector;
use std::collections::hash_map::Entry;
use std::collections::HashMap;

use crate::error::{Error, Result};

/// DuckDuckGo URL
const DUCKDUCKGO_URL: &str = "https://duckduckgo.com";

// TODO Should there be separate Unit-type structs for each one? With separate implementations?
pub enum SearchEngine {
    DuckDuckGo,
}

// Is question_id unique across all sites? If not, then this edge case is
// unaccounted for when sorting.
//
// If this is ever an issue, it wouldn't be too hard to account for this; just
// keep track of site in the `ordering` field and also return site from the
// spawned per-site tasks.
#[derive(Debug, PartialEq)]
pub struct ScrapedData {
    /// Mapping of site code to question ids
    pub question_ids: HashMap<String, Vec<String>>,
    /// Mapping of question_id to its ordinal place in search results
    pub ordering: HashMap<String, usize>,
}

pub trait Scraper {
    /// Parse data from search results html
    fn parse(&self, html: &str, sites: &HashMap<String, String>, limit: u16)
        -> Result<ScrapedData>;

    /// Get the url to search query restricted to sites
    fn get_url<'a, I>(&self, query: &str, sites: I) -> Url
    where
        I: IntoIterator<Item = &'a String>;
}

impl Scraper for SearchEngine {
    fn parse(
        &self,
        html: &str,
        sites: &HashMap<String, String>,
        limit: u16,
    ) -> Result<ScrapedData> {
        match &self {
            SearchEngine::DuckDuckGo => parse_duckduckgo(html, sites, limit),
        }
    }
    fn get_url<'a, I>(&self, query: &str, sites: I) -> Url
    where
        I: IntoIterator<Item = &'a String>,
    {
        match &self {
            SearchEngine::DuckDuckGo => duckduckgo_url(query, sites),
        }
    }
}

/// Parse (site, question_id) pairs out of duckduckgo search results html
// TODO Benchmark this. It would likely be faster to use regex on the decoded url.
// TODO pull out parts that are composable across different engines
fn parse_duckduckgo<'a>(
    html: &'a str,
    sites: &'a HashMap<String, String>,
    limit: u16,
) -> Result<ScrapedData> {
    let fragment = Html::parse_document(html);
    let anchors = Selector::parse("a.result__a").unwrap();
    let mut question_ids: HashMap<String, Vec<String>> = HashMap::new();
    let mut ordering: HashMap<String, usize> = HashMap::new();
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
                ordering.insert(id.to_owned(), count);
                match question_ids.entry(site_code.to_owned()) {
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
        Ok(ScrapedData {
            question_ids,
            ordering,
        })
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

#[cfg(test)]
mod tests {
    use super::*;

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
    fn test_duckduckgo_parser() {
        let html = include_str!("../../test/exit-vim.html");
        let sites = vec![
            ("stackoverflow", "stackoverflow.com"),
            ("askubuntu", "askubuntu.com"),
        ]
        .into_iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect::<HashMap<String, String>>();
        let expected_scraped_data = ScrapedData {
            question_ids: vec![
                ("stackoverflow", vec!["11828270", "9171356"]),
                ("askubuntu", vec!["24406"]),
            ]
            .into_iter()
            .map(|(k, v)| {
                (
                    k.to_string(),
                    v.into_iter().map(|s| s.to_string()).collect(),
                )
            })
            .collect(),
            ordering: vec![("11828270", 0), ("9171356", 2), ("24406", 1)]
                .into_iter()
                .map(|(k, v)| (k.to_string(), v))
                .collect(),
        };
        assert_eq!(
            SearchEngine::DuckDuckGo.parse(html, &sites, 3).unwrap(),
            expected_scraped_data
        );
    }

    #[test]
    fn test_duckduckgo_blocker() -> Result<(), String> {
        let html = include_str!("../../test/bad-user-agent.html");
        let mut sites = HashMap::new();
        sites.insert(
            String::from("stackoverflow"),
            String::from("stackoverflow.com"),
        );

        match SearchEngine::DuckDuckGo.parse(html, &sites, 2) {
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
