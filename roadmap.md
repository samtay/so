# roadmap

### v0.1.0
1. Set up CLI with options (assume --lucky)
2. Hit SO API
3. Markdown parser
4. End-to-end synchronous --lucky output

### v0.1.1
1. Add xdga config

### v0.2.0
1. Add --no-lucky option
2. For --lucky, async parsing first q/a, then the rest
3. Tui-rs interface for viewing multiple results

### v0.2.1
1. Support multiple --site args & searches

### v0.3.0
1. Add google scraper + helpful error messages

### at some point
1. cross-platform release binaries
2. per platform package mgmt

## deps
1. clap
2. serde-json
2. pulldown-cmark (?)
3. crossterm
4. tui-rs
5. reqwest + scraper for google
