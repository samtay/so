use clap::{
    builder::{styling::AnsiColor as Ansi, Styles},
    value_parser, Arg, ArgAction, ArgMatches, ColorChoice, Command,
};

use crate::config::Config;
use crate::error::Result;

// TODO --add-site (in addition to defaults)
// TODO set_api_key should probably just be a bool, since we have config
pub struct Opts {
    pub list_sites: bool,
    pub print_config_path: bool,
    pub update_sites: bool,
    pub set_api_key: Option<String>,
    pub query: Option<String>,
    pub config: Config,
}

/// Get CLI opts and args, with defaults pulled from user configuration
pub fn get_opts() -> Result<Opts> {
    get_opts_with(Config::new, |a| a.get_matches())
}

/// Get CLI opts, starting with defaults produced from `mk_config` and matching args with
/// `get_matches`.
fn get_opts_with<F, G>(mk_config: F, get_matches: G) -> Result<Opts>
where
    F: FnOnce() -> Result<Config>,
    G: for<'a> FnOnce(Command) -> ArgMatches,
{
    let config = mk_config()?;
    let limit = config.limit.to_string();
    let sites = config.sites.join(";");
    let engine = config.search_engine.to_string();
    let clapp = Command::new("so")
        .color(ColorChoice::Always)
        .styles(STYLES)
        .version(clap::crate_version!())
        .author(clap::crate_authors!())
        .about(clap::crate_description!())
        .arg(
            Arg::new("list-sites")
                .long("list-sites")
                .action(ArgAction::SetTrue)
                .help("Print available StackExchange sites"),
        )
        .arg(
            Arg::new("update-sites")
                .long("update-sites")
                .action(ArgAction::SetTrue)
                .help("Update cache of StackExchange sites"),
        )
        .arg(
            Arg::new("set-api-key")
                .long("set-api-key")
                .num_args(1)
                .value_name("key")
                .help("Set StackExchange API key"),
        )
        .arg(
            Arg::new("print-config-path")
                .long("print-config-path")
                .help("Print path to config file")
                .action(ArgAction::SetTrue)
                .hide(true),
        )
        .arg(
            Arg::new("site")
                .long("site")
                .short('s')
                .action(ArgAction::Append)
                .num_args(1)
                .default_value(&sites)
                .value_name("site-code")
                .help("StackExchange site to search"),
        )
        .arg(
            Arg::new("limit")
                .long("limit")
                .short('l')
                .num_args(1)
                .default_value(&limit)
                .value_name("int")
                .value_parser(value_parser!(u16))
                .help("Question limit"),
        )
        .arg(
            Arg::new("lucky")
                .long("lucky")
                .action(ArgAction::SetTrue)
                .help("Print the top-voted answer of the most relevant question"),
        )
        .arg(
            Arg::new("no-lucky")
                .long("no-lucky")
                .action(ArgAction::SetTrue)
                .help("Disable lucky")
                .conflicts_with("lucky")
                .hide(!config.lucky),
        )
        .arg(
            Arg::new("query")
                .num_args(1..)
                .index(1)
                .required_unless_present_any([
                    "list-sites",
                    "update-sites",
                    "set-api-key",
                    "print-config-path",
                ]),
        )
        .arg(
            Arg::new("search-engine")
                .long("search-engine")
                .short('e')
                .num_args(1)
                .default_value(&engine)
                .value_name("engine")
                .value_parser(["duckduckgo", "google", "stackexchange"])
                .help("Use specified search engine")
                .next_line_help(true),
        );
    let matches = get_matches(clapp);
    let lucky = match (matches.get_flag("lucky"), matches.get_flag("no-lucky")) {
        (true, _) => true,
        (_, true) => false,
        _ => config.lucky,
    };
    Ok(Opts {
        list_sites: matches.get_flag("list-sites"),
        print_config_path: matches.get_flag("print-config-path"),
        update_sites: matches.get_flag("update-sites"),
        set_api_key: matches.get_one("set-api-key").cloned(),
        query: matches
            .get_many::<String>("query")
            .map(|words| words.map(|s| s.as_str()).collect::<Vec<_>>().join(" ")),
        config: Config {
            // these unwraps are safe via clap default values & validators
            limit: *matches.get_one("limit").unwrap(),
            search_engine: serde_yaml::from_str(
                matches.get_one::<String>("search-engine").unwrap(),
            )?,
            sites: matches
                .get_many::<String>("site")
                .expect("at least one site is required!")
                .flat_map(|s| s.split(';'))
                .map(String::from)
                .collect(),
            api_key: matches.get_one("set-api-key").cloned().or(config.api_key),
            lucky,
            ..config
        },
    })
}

const STYLES: Styles = Styles::styled()
    .header(Ansi::Red.on_default().bold())
    .usage(Ansi::Red.on_default().bold())
    .literal(Ansi::Blue.on_default().bold())
    .placeholder(Ansi::Green.on_default());

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::SearchEngine;

    fn defaults() -> Config {
        Config {
            api_key: Some(String::from("my key")),
            limit: 64,
            lucky: false,
            sites: vec![
                String::from("some"),
                String::from("sites"),
                String::from("yeah"),
            ],
            search_engine: SearchEngine::DuckDuckGo,
            copy_cmd: Some(String::from("wl-copy")),
        }
    }

    fn mk_config() -> Result<Config> {
        Ok(defaults())
    }

    #[test]
    fn test_defaults() {
        let opts = get_opts_with(mk_config, |a| {
            a.get_matches_from(vec!["so", "how do I exit Vim"])
        });

        assert_eq!(opts.unwrap().config, defaults());
    }

    #[test]
    fn test_overrides() {
        let opts = get_opts_with(mk_config, |a| {
            a.get_matches_from(vec!["so", "-s", "english", "how do I exit Vim"])
        });

        assert_eq!(
            opts.unwrap().config,
            Config {
                sites: vec![String::from("english")],
                ..defaults()
            }
        );

        let opts = get_opts_with(mk_config, |a| {
            a.get_matches_from(vec!["so", "-l", "5", "--lucky", "how do I exit Vim"])
        });

        assert_eq!(
            opts.unwrap().config,
            Config {
                limit: 5,
                lucky: true,
                ..defaults()
            }
        );
    }

    #[test]
    fn test_set_api_key() {
        let opts = get_opts_with(mk_config, |a| {
            a.get_matches_from(vec!["so", "--set-api-key", "new key"])
        })
        .unwrap();

        // Uses key in new config
        assert_eq!(
            opts.config,
            Config {
                api_key: Some(String::from("new key")),
                ..defaults()
            }
        );

        // Flags it in opts
        assert_eq!(opts.set_api_key, Some(String::from("new key")));
    }

    #[test]
    #[should_panic]
    fn test_conflicts() {
        get_opts_with(mk_config, |a| {
            a.try_get_matches_from(vec!["so", "--lucky", "--no-lucky"])
                .unwrap()
        })
        .unwrap();
    }
}
