# roadmap

### v0.1.0
 - [x] Set up CLI with options (assume --lucky)
 - [x] Hit SO API
 - [x] Markdown parser
 - [x] End-to-end synchronous --lucky output

### v0.1.1
 - [x] Add xdga config
 - [x] Finishing touches on cli opts like --set-api-key, etc.

### v0.2.0
 - [x] Termimad interface for viewing multiple results

### v0.2.1
 - [x] Add --no-lucky option
 - [x] For --lucky, async parsing first q/a, then the rest

### v0.2.2
 - [x] Support multiple --site args & searches

### v0.3.0
 - [x] Add duckduckgo scraper

### v0.3.1
 - [x] Add google scraper

### at some point
 - [x] use trust to distrubute app binaries
 - [ ] look up how to add logging `debug!` macros; will help troubleshooting blocked requests
 - [ ] handle backoff responses from SE
 - [ ] allow new queries from TUI, e.g. hit `/` for a prompt. This could
 also bring up an advanced search form that allows mutliselect of sites, select
 search engine, etc.
 - [ ] or `/` searches current q/a
 - [ ] clean up dingleberries in error.rs and term.rs ; only keep what's actually ergonomic
 - [ ] per platform package mgmt
 - [ ] more testing
