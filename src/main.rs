mod cli;
mod config;
mod error;
mod macros;
mod stackexchange;
mod term;

use config::Config;
use error::{Error, ErrorKind};
use stackexchange::{LocalStorage, StackExchange};
use std::io::stderr;
use term::ColoredOutput;

fn main() {
    (|| {
        let opts = cli::get_opts()?;
        let config = opts.config;
        let site = &config.site;
        let mut ls = LocalStorage::new()?;

        if opts.update_sites {
            ls.update_sites()?;
        }

        if opts.list_sites {
            let sites = ls.sites()?;
            match sites.into_iter().map(|s| s.api_site_parameter.len()).max() {
                Some(max_w) => {
                    for s in sites {
                        println!("{:>w$}: {}", s.api_site_parameter, s.site_url, w = max_w);
                    }
                }
                None => {
                    stderr()
                        .queue_error("The site list is empty. Try running ")
                        .queue_code_inline("so --update-sites")
                        .unsafe_flush();
                }
            }
            return Ok(());
        }

        match ls.validate_site(site) {
            Ok(true) => (),
            Ok(false) => {
                stderr()
                    .queue_error(&format!("{} is not a valid StackExchange site.\n\n", site)[..])
                    .queue_notice("If you think this is in error, try running\n\n")
                    .queue_code("so --update-sites\n\n")
                    .queue_notice_inline("to update the cached site listing. You can also run ")
                    .queue_code_inline("so --list-sites")
                    .queue_notice_inline(" to list all available sites.")
                    .unsafe_flush();
                return Ok(());
            }
            Err(Error {
                kind: ErrorKind::EmptySites,
                ..
            }) => {
                stderr()
                    .queue_error("The cached site list is empty. This can likely be fixed by\n\n")
                    .queue_code("so --update-sites\n\n")
                    .unsafe_flush();
                return Ok(());
            }
            Err(e) => return Err(e),
        }

        let se = StackExchange::new(Config {
            api_key: Some(String::from("8o9g7WcfwnwbB*Qp4VsGsw((")), // TODO remove when releasing
            ..config
        });

        if let Some(q) = opts.query {
            let que = se.search(&q)?;
            let ans = que
                .first()
                .ok_or(Error::no_results())?
                .answers
                .first()
                .ok_or(Error::from(
                "StackExchange returned a question with no answers; this shouldn't be possible!",
            ))?;
            println!("{}", ans.body);
        }

        Ok(())
    })()
    .unwrap_or_else(|e| match e {
        Error { error, .. } => printerr!(error),
    })
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_main() {
        //TODO
    }
}
