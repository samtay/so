use percent_encoding::percent_decode_str;
use reqwest::Url;
use scraper::html::Html;
use scraper::selector::Selector;
use std::collections::hash_map::Entry;
use std::collections::HashMap;

use crate::error::{Error, Result};

/// DuckDuckGo URL
const DUCKDUCKGO_URL: &str = "https://duckduckgo.com";
const GOOGLE_URL: &str = "https://google.com/search";

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

// TODO add this type system limitation to blog post
pub trait Scraper {
    /// Parse data from search results html
    fn parse(&self, html: &str, sites: &HashMap<String, String>, limit: u16)
        -> Result<ScrapedData>;

    /// Get the url to search query restricted to sites
    fn get_url<'a, I>(&self, query: &str, sites: I) -> Url
    where
        I: IntoIterator<Item = &'a String>;
}

pub struct DuckDuckGo;

impl Scraper for DuckDuckGo {
    /// Parse (site, question_id) pairs out of duckduckgo search results html
    fn parse(
        &self,
        html: &str,
        sites: &HashMap<String, String>,
        limit: u16,
    ) -> Result<ScrapedData> {
        let anchors = Selector::parse("a.result__a").unwrap();
        parse_with_selector(anchors, html, sites, limit).and_then(|sd| {
            // DDG seems to never have empty results, so assume this is blocked
            if sd.question_ids.is_empty() {
                Err(Error::ScrapingError(String::from(
                    "DuckDuckGo blocked this request",
                )))
            } else {
                Ok(sd)
            }
        })
    }

    /// Creates duckduckgo search url given sites and query
    /// See https://duckduckgo.com/params for more info
    fn get_url<'a, I>(&self, query: &str, sites: I) -> Url
    where
        I: IntoIterator<Item = &'a String>,
    {
        let q = make_query_arg(query, sites);
        Url::parse_with_params(
            DUCKDUCKGO_URL,
            &[("q", q.as_str()), ("kz", "-1"), ("kh", "-1")],
        )
        .unwrap()
    }
}

pub struct Google;

impl Scraper for Google {
    /// Parse SE data out of google search results html
    fn parse(
        &self,
        html: &str,
        sites: &HashMap<String, String>,
        limit: u16,
    ) -> Result<ScrapedData> {
        let anchors = Selector::parse("div.r > a").unwrap();
        // TODO detect no results
        // TODO detect blocked request
        parse_with_selector(anchors, html, sites, limit)
    }

    /// Creates duckduckgo search url given sites and query
    /// See https://duckduckgo.com/params for more info
    fn get_url<'a, I>(&self, query: &str, sites: I) -> Url
    where
        I: IntoIterator<Item = &'a String>,
    {
        let q = make_query_arg(query, sites);
        Url::parse_with_params(GOOGLE_URL, &[("q", q.as_str())]).unwrap()
    }
}

fn make_query_arg<'a, I>(query: &str, sites: I) -> String
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
    q
}

// TODO Benchmark this. It would likely be faster to use regex on the decoded url.
fn parse_with_selector(
    anchors: Selector,
    html: &str,
    sites: &HashMap<String, String>,
    limit: u16,
) -> Result<ScrapedData> {
    let fragment = Html::parse_document(html);
    let mut question_ids: HashMap<String, Vec<String>> = HashMap::new();
    let mut ordering: HashMap<String, usize> = HashMap::new();
    let mut count = 0;
    for anchor in fragment.select(&anchors) {
        let url = anchor
            .value()
            .attr("href")
            .ok_or_else(|| Error::ScrapingError("Anchor with no href".to_string()))
            .map(|href| percent_decode_str(href).decode_utf8_lossy().into_owned())?;
        sites.iter().find_map(|(site_code, site_url)| {
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
        });
        if count >= limit as usize {
            break;
        }
    }
    Ok(ScrapedData {
        question_ids,
        ordering,
    })
}

/// For example
/// ```
/// let id = "stackoverflow.com";
/// let input = "/l/?kh=-1&uddg=https://stackoverflow.com/questions/11828270/how-do-i-exit-the-vim-editor";
/// assert_eq!(question_url_to_id(site_url, input), Some(String::from("11828270")))
/// ```
// TODO use str_prefix once its stable
// TODO benchmark this. regex is almost undoubtably superior here
fn question_url_to_id(site_url: &str, input: &str) -> Option<String> {
    ["/questions/", "/q/"].iter().find_map(|segment| {
        let fragment = site_url.trim_end_matches('/').to_owned() + segment;
        let ix = input.find(&fragment)? + fragment.len();
        let input = &input[ix..];
        if let Some(end) = input.find('/') {
            Some(input[0..end].to_string())
        } else {
            Some(input[0..].to_string())
        }
    })
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
            DuckDuckGo.get_url(q, &sites).as_str(),
            String::from(
                "https://duckduckgo.com/\
                ?q=%28site%3Astackoverflow.com+OR+site%3Aunix.stackexchange.com%29\
                +how+do+I+exit+vim&kz=-1&kh=-1"
            )
        )
    }

    #[test]
    fn test_duckduckgo_parser() {
        let html = include_str!("../../test/duckduckgo/exit-vim.html");
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
            DuckDuckGo.parse(html, &sites, 3).unwrap(),
            expected_scraped_data
        );
    }

    #[test]
    fn test_google_parser() {
        let html = include_str!("../../test/google/exit-vim.html");
        let sites = vec![
            ("stackoverflow", "stackoverflow.com"),
            ("askubuntu", "askubuntu.com"),
        ]
        .into_iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect::<HashMap<String, String>>();
        let expected_scraped_data = ScrapedData {
            question_ids: vec![
                ("stackoverflow", vec!["11828270", "25919461"]),
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
            ordering: vec![("11828270", 0), ("25919461", 1), ("24406", 2)]
                .into_iter()
                .map(|(k, v)| (k.to_string(), v))
                .collect(),
        };
        assert_eq!(
            Google.parse(html, &sites, 3).unwrap(),
            expected_scraped_data
        );
    }

    #[test]
    fn test_google_q_parser() {
        let html = include_str!("../../test/google/parsing-q.html");
        let mut sites = HashMap::new();
        sites.insert(
            String::from("stackoverflow"),
            String::from("stackoverflow.com"),
        );
        let expected_scraped_data = ScrapedData {
            question_ids: vec![(
                String::from("stackoverflow"),
                vec![
                    String::from("3940128"),
                    String::from("4647368"),
                    String::from("12336105"),
                ],
            )]
            .into_iter()
            .collect(),
            ordering: vec![
                (String::from("3940128"), 0),
                (String::from("4647368"), 1),
                (String::from("12336105"), 2),
            ]
            .into_iter()
            .collect(),
        };
        assert_eq!(
            Google.parse(html, &sites, 3).unwrap(),
            expected_scraped_data
        );
    }

    #[test]
    fn test_duckduckgo_blocker() -> Result<(), String> {
        let html = include_str!("../../test/duckduckgo/bad-user-agent.html");
        let mut sites = HashMap::new();
        sites.insert(
            String::from("stackoverflow"),
            String::from("stackoverflow.com"),
        );

        match DuckDuckGo.parse(html, &sites, 2) {
            Err(Error::ScrapingError(s)) if s == "DuckDuckGo blocked this request".to_string() => {
                Ok(())
            }
            _ => Err(String::from("Failed to detect DuckDuckGo blocker")),
        }
    }

    // TODO  Get blocked google request html
    // note: this may only be possible at search.rs level (with non-200 code)

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
