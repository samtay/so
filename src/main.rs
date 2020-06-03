use clap::clap_app;
use clap::App;

// TODO pull defaults from config file
// may require dropping the macros
fn mk_app<'a, 'b>() -> App<'a, 'b> {
    clap_app!(so =>
        (version: clap::crate_version!())
        (author: clap::crate_authors!())
        (about: clap::crate_description!())
        (@arg site: -s --site +takes_value default_value("stackoverflow") "StackExchange site")
        (@arg query: ... +required "Query to search")
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
