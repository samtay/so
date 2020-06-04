use clap::{App, AppSettings, Arg};

mod config;
mod stackexchange;

// TODO maybe consts for these keywords?

// TODO pull defaults from config file
// TODO --set-api-key KEY
// TODO --update-cache
// TODO --install-filter-key --force
//?TODO --set-default-opt opt val # e.g. --set-default-opt sites site1;site2;site3
// may require dropping the macros
fn mk_app<'a, 'b>() -> App<'a, 'b> {
    App::new("so")
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
            Arg::with_name("site")
                .long("site")
                .short("s")
                .multiple(true)
                .number_of_values(1)
                .takes_value(true)
                .default_value("stackoverflow")
                .help("StackExchange site codes to search"),
        )
        .arg(
            Arg::with_name("limit")
                .long("limit")
                .short("l")
                .number_of_values(1)
                .takes_value(true)
                .default_value("50")
                .validator(|s| s.parse::<u32>().map_err(|e| e.to_string()).map(|_| ()))
                .help("Question limit per site query"),
        )
        .arg(
            Arg::with_name("lucky")
                .long("lucky")
                .help("Print the top-voted answer of the most relevant question"),
        )
        .arg(
            Arg::with_name("query")
                .multiple(true)
                .index(1)
                .required(true)
                .required_unless("list-sites"),
        )
}

fn main() {
    let matches = mk_app().get_matches();
    println!("{:?}", matches);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli() {
        let m = mk_app().get_matches_from(vec![
            "so", "--site", "meta", "how", "do", "I", "exit", "Vim",
        ]);
        println!("{:?}", m);
        assert_eq!(m.value_of("site"), Some("meta"));
        assert_eq!(
            m.values_of("query").unwrap().collect::<Vec<_>>().join(" "),
            "how do I exit Vim"
        );
    }
}
