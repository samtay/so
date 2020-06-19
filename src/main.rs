mod cli;
mod config;
mod error;
mod stackexchange;
mod term;
mod tui;
mod utils;

use crate::stackexchange::Question;
use crate::tui::markdown::Markdown;
use crossterm::style::Color;
use error::{Error, Result};
use lazy_static::lazy_static;
use minimad::mad_inline;
use stackexchange::{LocalStorage, StackExchange};
use term::mk_print_error;
use termimad::{CompoundStyle, MadSkin};
use tokio::runtime::Runtime;
use tokio::task;

fn main() -> Result<()> {
    // Markdown styles (outside of TUI)
    let mut skin = MadSkin::default();
    skin.inline_code = CompoundStyle::with_fg(Color::Cyan);
    skin.code_block.set_fgbg(Color::Cyan, termimad::gray(20));
    let mut print_error = mk_print_error(&skin);

    // Tokio runtime
    let mut rt = Runtime::new()?;
    rt.block_on(run(&mut skin))
        .and_then(|qs| {
            // Run TUI
            qs.map(tui::run);
            Ok(())
        })
        .or_else(|e: Error| {
            // Handle errors
            print_error(&e.to_string())?;
            match e {
                Error::EmptySites => {
                    print_notice!(skin, "This can likely be fixed by `so --update-sites`.")
                }
                _ => Ok(()),
            }
        })
}

/// Runs the CLI and, if the user wishes to enter the TUI, returns
/// question/answer data
async fn run(skin: &mut MadSkin) -> Result<Option<Vec<Question<Markdown>>>> {
    let opts = cli::get_opts()?;
    let config = opts.config;
    let sites = &config.sites;
    let lucky = config.lucky;
    let mut ls = LocalStorage::new()?;

    if let Some(key) = opts.set_api_key {
        config::set_api_key(key)?;
    }

    if opts.update_sites {
        ls.update_sites().await?;
    }

    if opts.list_sites {
        let sites = ls.sites().await?;
        let mut md = String::new();
        md.push_str("|:-:|:-:|\n");
        md.push_str("|Site Code|Site URL|\n");
        md.push_str("|-:|:-|\n");
        for s in sites.iter() {
            md.push_str(&format!("|{}|{}\n", s.api_site_parameter, s.site_url));
        }
        md.push_str("|-\n");
        termimad::print_text(&md);
        return Ok(None);
    }

    if let Some(site) = ls.find_invalid_site(sites).await? {
        print_error!(skin, "$0 is not a valid StackExchange site.\n\n", site)?;
        // TODO should only use inline for single lines; use termimad::text stuff
        print_notice!(
            skin,
            "If you think this is incorrect, try running\n\
                ```\n\
                so --update-sites\n\
                ```\n\
                to update the cached site listing. You can also run `so --list-sites` \
                to list all available sites.",
        )?;
        return Ok(None);
    }

    if let Some(q) = opts.query {
        let se = StackExchange::new(config, q);
        if lucky {
            // TODO this needs preprocessing; all the more reason to do it at SE level
            let md = se.search_lucky().await?;
            skin.print_text(&md);
            skin.print_text("\nPress **[SPACE]** to see more results, or any other key to exit");
            // Kick off the rest of the search in the background
            let qs = task::spawn(async move { se.search().await });
            if !utils::wait_for_char(' ')? {
                return Ok(None);
            }
            return Ok(Some(qs.await.unwrap()?));
        } else {
            return Ok(Some(se.search().await?));
        }
    }
    Ok(None)
}
