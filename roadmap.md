# roadmap

### v0.1.0
[x] Set up CLI with options (assume --lucky)
[x] Hit SO API
[x] Markdown parser
[x] End-to-end synchronous --lucky output

### v0.1.1
[x] Add xdga config
[x] Finishing touches on cli opts like --set-api-key, etc.

### v0.2.0
[x] Termimad interface for viewing multiple results

### v0.2.1
[x] Add --no-lucky option
[x] For --lucky, async parsing first q/a, then the rest

### v0.2.2
[x] Support multiple --site args & searches

### v0.3.0
[ ] Add duckduckgo scraper

### at some point
[ ] use trust to distrubute app binaries
[ ] ask SE forums if I should bundle my api-key? (if so use an env var macro)
[ ] allow new queries from TUI, e.g. hit `/` for a prompt
[ ] or `/` searches current q/a
[ ] clean up error.rs and term.rs ; only keep whats actually ergonomic
[ ] ask legal@stackoverflow.com for permission to logo stackoverflow/stackexchange in readme
[ ] add duckduckgo logo to readme
[ ] per platform package mgmt
[ ] more testing
[ ] maybe add google engine too. but fuck google.
