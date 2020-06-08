mod cli;
mod config;
mod error;
mod macros;
mod stackexchange;

use crossterm::style::Color;
use error::{Error, ErrorKind};
use lazy_static::lazy_static;
use minimad::mad_inline;
use stackexchange::{LocalStorage, StackExchange};
use termimad::{CompoundStyle, MadSkin};

fn main() {
    (|| {
        let opts = cli::get_opts()?;
        let config = opts.config;
        let site = &config.site;
        let mut ls = LocalStorage::new()?;
        // TODO style configuration
        let mut skin = MadSkin::default();
        skin.inline_code = CompoundStyle::with_fg(Color::Cyan);
        skin.code_block.set_fgbg(Color::Cyan, termimad::gray(20));

        if opts.update_sites {
            ls.update_sites()?;
        }

        if opts.list_sites {
            let sites = ls.sites()?;
            let mut md = String::new();
            md.push_str("|:-:|:-:|\n");
            md.push_str("|Site Code|Site URL|\n");
            md.push_str("|-:|:-|\n");
            for s in sites.iter() {
                md.push_str(&format!("|{}|{}\n", s.api_site_parameter, s.site_url));
            }
            md.push_str("|-\n");
            termimad::print_text(&md);
            return Ok(());
        }

        match ls.validate_site(site) {
            Ok(true) => (),
            Ok(false) => {
                print_error!(skin, "$0 is not a valid StackExchange site.\n\n", site)?;
                // TODO what about using text wrapping feature?
                print_notice!(
                    skin,
                    "If you think this is incorrect, try running\n\
                    ```\n\
                    so --update-sites\n\
                    ```\n\
                    to update the cached site listing. You can also run `so --list-sites` \
                    to list all available sites.",
                )?;
                return Ok(());
            }
            Err(Error {
                kind: ErrorKind::EmptySites,
                ..
            }) => {
                // TODO use text wrapping feature
                print_error!(
                    skin,
                    "The cached site list is empty. This can likely be fixed by\n\n\
                    ```\n\
                    so --update-sites\n\
                    ```"
                )?;
                return Ok(());
            }
            Err(e) => return Err(e),
        }

        if let Some(q) = opts.query {
            let se = StackExchange::new(config);
            let que = se.search(&q)?;
            let ans = que
                .first()
                .ok_or_else(Error::no_results)?
                .answers
                .first()
                .ok_or_else(|| {
                    Error::from(
                        "StackExchange returned a question with no answers; \
                        this shouldn't be possible!",
                    )
                })?;
            // TODO eventually do this in the right place, e.g. abstract out md parser & do within threads
            let md = ans.body.replace("<kbd>", "**[").replace("</kbd>", "]**");
            skin.print_text(&md);
        }

        Ok(())
    })()
    .or_else(|e| print_error!(MadSkin::default(), "{}", &e.error))
    .unwrap_or_else(|e| {
        println!("{}", e.error);
    });
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_main() {
        //TODO
    }
}
