use clap::{App, AppSettings, Arg};

use crate::config;
use crate::config::Config;
use crate::error::Result;

// TODO maybe consts for these keywords?

// TODO --set-api-key KEY
// TODO --update-sites
// TODO --install-filter-key --force
// TODO --sites plural
// TODO --add-site (in addition to defaults)
pub struct Opts {
    pub list_sites: bool,
    pub update_sites: bool,
    pub query: Option<String>,
    pub config: Config,
}

pub fn get_opts() -> Result<Opts> {
    let config = config::user_config()?;
    let limit = &config.limit.to_string();
    let matches = App::new("so")
        .setting(AppSettings::ColoredHelp)
        .version(clap::crate_version!())
        .author(clap::crate_authors!())
        .about(clap::crate_description!())
        .arg(
            Arg::with_name("list-sites")
                .long("list-sites")
                .help("Print available StackExchange sites"),
        )
        .arg(
            Arg::with_name("update-sites")
                .long("update-sites")
                .help("Update cache of StackExchange sites"),
        )
        .arg(
            Arg::with_name("site")
                .long("site")
                .short("s")
                .multiple(true)
                .number_of_values(1)
                .takes_value(true)
                .default_value(&config.site)
                .help("StackExchange site code to search"), // TODO sites plural
        )
        .arg(
            Arg::with_name("limit")
                .long("limit")
                .short("l")
                .number_of_values(1)
                .takes_value(true)
                .default_value(limit)
                .validator(|s| s.parse::<u32>().map_err(|e| e.to_string()).map(|_| ()))
                .help("Question limit per site query")
                .hidden(true), // TODO unhide once more than just --lucky
        )
        .arg(
            Arg::with_name("lucky")
                .long("lucky")
                .help("Print the top-voted answer of the most relevant question")
                .hidden(true), // TODO unhide
        )
        .arg(
            Arg::with_name("query")
                .multiple(true)
                .index(1)
                .required_unless_one(&["list-sites", "update-sites"]),
        )
        .get_matches();

    Ok(Opts {
        list_sites: matches.is_present("list-sites"),
        update_sites: matches.is_present("update-sites"),
        query: matches
            .values_of("query")
            .map(|q| q.into_iter().collect::<Vec<_>>().join(" ")),
        config: Config {
            // these unwraps are safe via clap default values & validators
            limit: matches.value_of("limit").unwrap().parse::<u16>().unwrap(),
            site: matches.value_of("site").unwrap().to_string(),
            // TODO if set_api_key passed, pass it here too
            ..config
        },
    })
}

#[cfg(test)]
mod tests {
    // TODO how can I test this now that it depends on user dir?
}
