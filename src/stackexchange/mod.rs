mod api;
mod local_storage;
mod search;
// Exposed for benchmarking
pub mod scraper;

pub use api::{Answer, Id, Question};
pub use local_storage::{LocalStorage, SiteMap};
pub use search::Search;

/// Mock user agent
const USER_AGENT: &str =
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10.7; rv:11.0) Gecko/20100101 Firefox/11.0";
