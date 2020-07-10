use clap::{App, AppSettings, Arg};

use crate::config::Config;
use crate::error::Result;

// TODO --add-site (in addition to defaults)
pub struct Opts {
    pub list_sites: bool,
    pub print_config_path: bool,
    pub update_sites: bool,
    pub set_api_key: Option<String>,
    pub query: Option<String>,
    pub config: Config,
}

// TODO accept FnOnce() -> Result<Config> so I can test this
pub fn get_opts() -> Result<Opts> {
    let config = Config::new()?;
    let limit = &config.limit.to_string();
    let sites = &config.sites.join(";");
    let engine = &config.search_engine.to_string();
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
            Arg::with_name("set-api-key")
                .long("set-api-key")
                .number_of_values(1)
                .takes_value(true)
                .value_name("key")
                .help("Set StackExchange API key"),
        )
        .arg(
            Arg::with_name("print-config-path")
                .long("print-config-path")
                .help("Print path to config file")
                .hidden(true),
        )
        .arg(
            Arg::with_name("site")
                .long("site")
                .short("s")
                .multiple(true)
                .number_of_values(1)
                .takes_value(true)
                .default_value(sites)
                .value_name("site-code")
                .help("StackExchange site to search"),
        )
        .arg(
            Arg::with_name("limit")
                .long("limit")
                .short("l")
                .number_of_values(1)
                .takes_value(true)
                .default_value(limit)
                .value_name("int")
                .validator(|s| s.parse::<u32>().map(|_| ()).map_err(|e| e.to_string()))
                .help("Question limit"),
        )
        .arg(
            Arg::with_name("lucky")
                .long("lucky")
                .help("Print the top-voted answer of the most relevant question"),
        )
        .arg(
            Arg::with_name("no-lucky")
                .long("no-lucky")
                .help("Disable lucky")
                .conflicts_with("lucky")
                .hidden(!config.lucky),
        )
        .arg(
            Arg::with_name("query")
                .multiple(true)
                .index(1)
                .required_unless_one(&[
                    "list-sites",
                    "update-sites",
                    "set-api-key",
                    "print-config-path",
                ]),
        )
        .arg(
            Arg::with_name("search-engine")
                .long("search-engine")
                .short("e")
                .number_of_values(1)
                .takes_value(true)
                .default_value(engine)
                .value_name("engine")
                .possible_values(&["duckduckgo", "google", "stackexchange"])
                .help("Use specified search engine")
                .next_line_help(true),
        )
        .get_matches();
    let lucky = match (matches.is_present("lucky"), matches.is_present("no-lucky")) {
        (true, _) => true,
        (_, true) => false,
        _ => config.lucky,
    };
    Ok(Opts {
        list_sites: matches.is_present("list-sites"),
        print_config_path: matches.is_present("print-config-path"),
        update_sites: matches.is_present("update-sites"),
        set_api_key: matches.value_of("set-api-key").map(String::from),
        query: matches
            .values_of("query")
            .map(|q| q.collect::<Vec<_>>().join(" ")),
        config: Config {
            // these unwraps are safe via clap default values & validators
            limit: matches.value_of("limit").unwrap().parse::<u16>().unwrap(),
            search_engine: serde_yaml::from_str(matches.value_of("search-engine").unwrap())?,
            sites: matches
                .values_of("site")
                .unwrap()
                .map(|s| s.split(';'))
                .flatten()
                .map(String::from)
                .collect(),
            api_key: matches
                .value_of("set-api-key")
                .map(String::from)
                .or(config.api_key),
            lucky,
        },
    })
}

// TODO how can I test this App given
// https://users.rust-lang.org/t/help-with-idiomatic-rust-and-ownership-semantics/43880
// Maybe pass get_opts a closure that produces the Config...?
