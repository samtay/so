mod api;
mod local_storage;
mod search;
// Exposed for benchmarking
pub mod scraper;

pub use api::{Answer, Id, Question};
pub use local_storage::{LocalStorage, SiteMap};
pub use search::Search;

use reqwest::header::{self, HeaderMap, HeaderName, HeaderValue};
use reqwest::Client;

/// Mock user agent (kept for API client which doesn't need browser spoofing)
const USER_AGENT: &str =
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10.7; rv:11.0) Gecko/20100101 Firefox/11.0";

/// Pool of modern browser User-Agent strings for scraper requests.
/// Covers Chrome, Firefox, Safari, and Edge across Windows, macOS, and Linux.
const SCRAPER_USER_AGENTS: &[&str] = &[
    // Chrome 131 - Windows
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36",
    // Chrome 131 - macOS
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36",
    // Firefox 133 - Windows
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:133.0) Gecko/20100101 Firefox/133.0",
    // Firefox 133 - Linux
    "Mozilla/5.0 (X11; Linux x86_64; rv:133.0) Gecko/20100101 Firefox/133.0",
    // Safari 17.5 - macOS
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.5 Safari/605.1.15",
    // Chrome 131 - Linux
    "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36",
    // Edge 131 - Windows
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36 Edg/131.0.0.0",
    // Firefox 133 - macOS
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10.15; rv:133.0) Gecko/20100101 Firefox/133.0",
];

/// Select a User-Agent from the pool using the process ID for variation
/// across invocations without requiring the `rand` crate.
fn select_scraper_user_agent() -> &'static str {
    let index = (std::process::id() as usize) % SCRAPER_USER_AGENTS.len();
    SCRAPER_USER_AGENTS[index]
}

/// Build browser-like default headers matched to the selected User-Agent.
fn scraper_headers() -> HeaderMap {
    let ua = select_scraper_user_agent();
    let mut headers = HeaderMap::new();

    headers.insert(header::USER_AGENT, HeaderValue::from_static(ua));
    headers.insert(
        header::ACCEPT_LANGUAGE,
        HeaderValue::from_static("en-US,en;q=0.9"),
    );
    headers.insert(
        HeaderName::from_static("upgrade-insecure-requests"),
        HeaderValue::from_static("1"),
    );

    // Match Accept header to browser family
    if ua.contains("Firefox") {
        headers.insert(
            header::ACCEPT,
            HeaderValue::from_static(
                "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8",
            ),
        );
    } else {
        headers.insert(
            header::ACCEPT,
            HeaderValue::from_static(
                "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8",
            ),
        );
    }

    // Chrome and Edge send Sec-Fetch-* headers
    if ua.contains("Chrome") || ua.contains("Edg/") {
        headers.insert(
            HeaderName::from_static("sec-fetch-dest"),
            HeaderValue::from_static("document"),
        );
        headers.insert(
            HeaderName::from_static("sec-fetch-mode"),
            HeaderValue::from_static("navigate"),
        );
        headers.insert(
            HeaderName::from_static("sec-fetch-site"),
            HeaderValue::from_static("none"),
        );
        headers.insert(
            HeaderName::from_static("sec-fetch-user"),
            HeaderValue::from_static("?1"),
        );
    }

    headers
}

/// Build an HTTP client with browser-like headers for scraping search engines.
pub(crate) fn scraper_client() -> Client {
    Client::builder()
        .default_headers(scraper_headers())
        .build()
        .expect("Failed to build scraper HTTP client")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_agent_pool_contains_only_modern_agents() {
        for ua in SCRAPER_USER_AGENTS {
            assert!(!ua.contains("Firefox/11.0"), "{}", ua);
            assert!(!ua.contains("Mac OS X 10.7"), "{}", ua);
        }
    }

    #[test]
    fn test_select_scraper_user_agent_returns_from_pool() {
        let ua = select_scraper_user_agent();
        assert!(SCRAPER_USER_AGENTS.contains(&ua));
    }

    #[test]
    fn test_select_scraper_user_agent_is_deterministic() {
        let ua1 = select_scraper_user_agent();
        let ua2 = select_scraper_user_agent();
        assert_eq!(ua1, ua2, "Same process should always select the same UA");
    }

    #[test]
    fn test_scraper_headers_include_required_fields() {
        let headers = scraper_headers();
        assert!(headers.contains_key(header::USER_AGENT));
        assert!(headers.contains_key(header::ACCEPT));
        assert!(headers.contains_key(header::ACCEPT_LANGUAGE));
        assert!(headers.contains_key("upgrade-insecure-requests"));
    }

    #[test]
    fn test_scraper_headers_match_browser_family() {
        let headers = scraper_headers();
        let ua = headers.get(header::USER_AGENT).unwrap().to_str().unwrap();
        let accept = headers.get(header::ACCEPT).unwrap().to_str().unwrap();

        if ua.contains("Firefox") {
            // Firefox uses a shorter Accept without image types
            assert!(
                !accept.contains("image/avif"),
                "Firefox UA should not have Chrome-style Accept"
            );
        }

        if ua.contains("Chrome") || ua.contains("Edg/") {
            // Chrome/Edge should have Sec-Fetch headers
            assert!(
                headers.contains_key("sec-fetch-dest"),
                "Chrome/Edge UA should have sec-fetch-dest"
            );
        }
    }

    #[test]
    fn test_scraper_client_builds_successfully() {
        let _client = scraper_client();
    }
}
