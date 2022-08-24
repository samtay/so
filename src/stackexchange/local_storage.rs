use std::collections::HashMap;
use std::fs;
use std::ops::Deref;
use std::path::Path;

use crate::config::Config;
use crate::error::{Error, Result};
use crate::utils;

use super::api::{Api, Site};
use super::Question;

/// This structure allows interacting with locally cached StackExchange metadata.
pub struct LocalStorage {
    pub sites: Vec<Site>,
}

impl LocalStorage {
    fn fetch_local_sites(filename: &Path) -> Result<Option<Vec<Site>>> {
        if let Some(file) = utils::open_file(filename)? {
            return serde_json::from_reader(file)
                .map_err(|_| Error::MalformedFile(filename.to_path_buf()));
        }
        Ok(None)
    }

    fn store_local_sites(filename: &Path, sites: &[Site]) -> Result<()> {
        let file = utils::create_file(filename)?;
        serde_json::to_writer(file, sites)?;
        Ok(())
    }

    async fn init_sites(filename: &Path, update: bool) -> Result<Vec<Site>> {
        if !update {
            if let Some(sites) = Self::fetch_local_sites(filename)? {
                return Ok(sites);
            }
        }
        let sites = Api::new(None).sites().await?;
        Self::store_local_sites(filename, &sites)?;
        Ok(sites)
    }

    pub async fn new(update: bool) -> Result<Self> {
        let project = Config::project_dir()?;
        let dir = project.cache_dir();
        fs::create_dir_all(&dir)?;
        let sites_filename = dir.join("sites.json");
        let sites = Self::init_sites(&sites_filename, update).await?;
        Ok(LocalStorage { sites })
    }

    // TODO is this HM worth it? Probably only will ever have < 10 site codes to search...
    // maybe store this as Option<HM> on self if other methods use it...
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

    pub fn get_site_map(&self, site_codes: &[String]) -> SiteMap {
        let inner = self
            .sites
            .iter()
            .filter_map(move |site| {
                let _ = site_codes
                    .iter()
                    .find(|&sc| *sc == site.api_site_parameter)?;
                Some((site.api_site_parameter.to_owned(), site.site_url.to_owned()))
            })
            .collect();
        SiteMap { inner }
    }
}

/// Just a map of site codes to site URLs, shareable across the app. These are
/// only the sites relevant to the configuration / query, not all cached SE
/// sites.
#[derive(Debug)]
pub struct SiteMap {
    inner: HashMap<String, String>,
}

impl Deref for SiteMap {
    type Target = HashMap<String, String>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl SiteMap {
    /// Get SE answer url. Panics if site was not set on the question, or
    /// site code not found in the map.
    pub fn answer_url<S>(&self, question: &Question<S>, answer_id: u32) -> String {
        // answer link actually doesn't need question id
        let site_url = self.site_url(question);
        format!("https://{site_url}/a/{answer_id}")
    }

    /// Get SE question url. Panics if site was not set on the question, or
    /// site code not found in the map.
    pub fn question_url<S>(&self, question: &Question<S>) -> String {
        // answer link actually doesn't need question id
        let question_id = question.id;
        let site_url = self.site_url(question);
        format!("https://{site_url}/q/{question_id}")
    }

    fn site_url<S>(&self, question: &Question<S>) -> String {
        self.inner
            .get(
                question
                    .site
                    .as_ref()
                    .expect("bug: site not attached to question"),
            )
            .cloned()
            .expect("bug: lost a site")
    }
}
