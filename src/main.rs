mod cli;
mod config;
mod error;
mod stackexchange;
mod term;
mod tui;
mod utils;

use tokio::runtime::Runtime;
use tokio::task;

use config::Config;
use error::{Error, Result};
use stackexchange::{LocalStorage, Question, Search};
use term::Term;
use tui::markdown::Markdown;

fn main() -> Result<()> {
    // Tokio runtime
    Runtime::new()?
        .block_on(run())
        .and_then(|qs| {
            // Run TUI
            qs.map(tui::run);
            Ok(())
        })
        .or_else(|e: Error| {
            // Handle errors
            term::print_error(&e.to_string())
        })
}

/// Runs the CLI and, if the user wishes to enter the TUI, returns
/// question/answer data
async fn run() -> Result<Option<Vec<Question<Markdown>>>> {
    // Get CLI opts
    let opts = cli::get_opts()?;
    let config = opts.config;
    let sites = &config.sites;
    let lucky = config.lucky;

    // Term tools and markdown styles (outside of TUI)
    let mut term = Term::new();

    let ls = LocalStorage::new(opts.update_sites).await?;

    if let Some(key) = opts.set_api_key {
        Config::set_api_key(key)?;
    }

    if opts.print_config_path {
        println!("{}", Config::config_file_path()?.display());
    }

    if opts.list_sites {
        let mut md = String::new();
        md.push_str("|:-:|:-:|\n");
        md.push_str("|Site Code|Site URL|\n");
        md.push_str("|-:|:-|\n");
        for s in ls.sites.iter() {
            md.push_str(&format!("|{}|{}\n", s.api_site_parameter, s.site_url));
        }
        md.push_str("|-\n");
        term.print(&md);
        return Ok(None);
    }

    if let Some(site) = ls.find_invalid_site(sites).await {
        term.print_error(&format!("{} is not a valid StackExchange site.\n\n", site))?;
        term.print_notice(
            "If you think this is incorrect, try running\n\
                ```\n\
                so --update-sites\n\
                ```\n\
                to update the cached site listing. \
                You can also run `so --list-sites` to list all available sites.",
        )?;
        return Ok(None);
    }

    if let Some(q) = opts.query {
        let mut search = Search::new(config, ls, q);
        if lucky {
            // Show top answer
            let md = Term::wrap_spinner(search.search_lucky()).await??;
            term.print(&md);
            term.print("\nPress **[SPACE]** to see more results, or any other key to exit");

            // Kick off the rest of the search in the background
            let qs = task::spawn(async move { search.search_md().await });
            if !Term::wait_for_char(' ')? {
                return Ok(None);
            }

            // Get the rest of the questions
            return Ok(Some(Term::wrap_spinner(qs).await?.unwrap()?));
        } else {
            return Ok(Some(Term::wrap_spinner(search.search_md()).await??));
        }
    }
    Ok(None)
}
