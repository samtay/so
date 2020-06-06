mod cli;
mod config;
mod stackexchange;

use config::Config;
use stackexchange::{LocalStorage, StackExchange};

fn main() {
    // TODO wrap inner function with Result<(), ErrorMessage>, propagate, print to stderr at the top level.
    let opts = cli::get_opts();
    let config = opts.config;
    let site = &config.site;
    let mut ls = LocalStorage::new();

    if opts.update_sites {
        ls.update_sites();
    }

    if opts.list_sites {
        for s in ls.sites() {
            println!("{}: {}", s.api_site_parameter, s.site_url);
        }
        return;
    }

    // TODO make this validation optional
    if !ls.validate_site(site) {
        // TODO tooling for printing to stderr with color, etc.
        println!(
            "{} is not a valid StackExchange site. If you think this
            is in error, try running `so --update-sites` to update
            the cached site listing.  Run `so --list-sites` for all
            available sites.",
            site
        );
        return;
    }

    let se = StackExchange::new(Config {
        api_key: Some(String::from("8o9g7WcfwnwbB*Qp4VsGsw((")), // TODO stash this
        ..config
    });

    let query = opts.query;
    (|| -> Option<_> {
        let q = query?;
        let que = se.search(&q).unwrap(); // TODO eventually be graceful
        let ans = que.first()?.answers.first()?;
        println!("{}", ans.body);
        Some(())
    })();
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_main() {
        //TODO
    }
}
