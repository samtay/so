# TODO

### chores
1. Make an example GIF
2. Move to github actions ASAP, travis & appveyor are a PITA. See resources below.
3. Benchmark parsing. Probaly way faster to use regex to find question IDs
   within URLs (or the entire doc).

### bugs
1. Need to also allow /q/ in search results in addition to /questions/
2.
```
so --search-engine google --site stackoverflow --site askubuntu how to stop typing sudo
```
results in
```
thread '<unnamed>' panicked at 'assertion failed: value_pos >= source_pos', /home/sam/.cargo/registry/src/github.com-1ecc6299db9ec823/cursive_core-0.1.0/src/utils/span.rs:372:17
```
So maybe the md parser should just build its own source for
SpannedString, and own everything...

### feature ideas
- Add sort option, e.g. relevance|votes|date
- Keep track of quota in a data file, inform user when getting close?

#### Endless improvements for the TUI
3. Small text at bottom with '?' to bring up key mapping dialog
1. Init with smaller layout depending on initial screen size.
2. Maybe cli `--auto-resize` option that changes layouts at breakpoints.
5. Maybe **[ESC]** cycles layout in the opposite direction? And stops at
   BothColumns?
6. Allow cycling through themes, either found in `~/.config/so/colors/*.toml`
    or just hardcoded ones.
7. Small tray at the bottom with "notifications", e.g. "GitHub Theme loaded!"

**Important note:** Tables are not currently allowed in stackexchange... so the
benefit of incorporating termimad features into a cursive view will not be felt.
But, this is changing [soon](https://meta.stackexchange.com/q/348746).


### resources for later
- [Trust example](https://github.com/badboy/signify-rs)
- [Github Actions example](https://github.com/extrawurst/gitui)
- [logging example](https://deterministic.space/rust-cli-tips.html)
- [PKGBUILD example](https://aur.archlinux.org/cgit/aur.git/tree/PKGBUILD?h=gitui) + openssl dep
- More mock user agents
  - Mozilla/5.0 (Macintosh; Intel Mac OS X 10.7; rv:11.0) Gecko/20100101 Firefox/11.0
  - Mozilla/5.0 (X11; Ubuntu; Linux x86_64; rv:22.0) Gecko/20100 101 Firefox/22.0
  - Mozilla/5.0 (Windows NT 6.1; rv:11.0) Gecko/20100101 Firefox/11.0
  - Mozilla/5.0 (Macintosh; Intel Mac OS X 10_7_4) AppleWebKit/536.5 (KHTML, like Gecko) Chrome/19.0.1084.46 Safari/536.5
  - Mozilla/5.0 (Windows; Windows NT 6.1) AppleWebKit/536.5 (KHTML, like Gecko) Chrome/19.0.1084.46 Safari/536.5

