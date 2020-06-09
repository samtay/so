mod cli;
mod config;
mod error;
mod stackexchange;
mod term;
mod utils;

use crossterm::style::Color;
use error::Error;
use lazy_static::lazy_static;
use minimad::mad_inline;
use stackexchange::{LocalStorage, StackExchange};
use term::with_error_style;
use termimad::{CompoundStyle, MadSkin};

fn main() {
    let mut skin = MadSkin::default();
    // TODO style configuration
    skin.inline_code = CompoundStyle::with_fg(Color::Cyan);
    skin.code_block.set_fgbg(Color::Cyan, termimad::gray(20));
    (|| {
        let opts = cli::get_opts()?;
        let config = opts.config;
        let site = &config.site;
        let mut ls = LocalStorage::new()?;

        if let Some(key) = opts.set_api_key {
            config::set_api_key(key)?;
        }

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

        if !ls.validate_site(site)? {
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

        if let Some(q) = opts.query {
            let se = StackExchange::new(config);
            let que = se.search(&q)?;
            let ans = que.first().ok_or(Error::NoResults)?.answers.first().expect(
                "StackExchange returned a question with no answers; \
                    this shouldn't be possible!",
            );
            // TODO eventually do this in the right place, e.g. abstract out md parser, write benches, & do within threads
            let md = ans.body.replace("<kbd>", "**[").replace("</kbd>", "]**");
            skin.print_text(&md);
        }

        Ok(())
    })()
    .or_else(|e: Error| {
        with_error_style(&mut skin, |err_skin, stderr| {
            err_skin.write_text_on(stderr, &e.to_string())
        })?;
        match e {
            Error::EmptySites => {
                print_notice!(skin, "This can likely be fixed by `so --update-sites`.")
            }
            _ => Ok(()),
        }
    })
    .unwrap();
}
