# TODO

### chores
1. Add significant cursive TUI test
2. Move to `directories 3.0`; optionally migrate existing macos configs? Not
   many people using this anyway...
3. Move to github actions ASAP, travis & appveyor are a PITA. See resources below.

### bugs
[ ] Support lack of persistent configuration:
   1) If we can't write it, still continue with application defaults, and
   2) allow passing --config-path=PATH;
[ ] Shift+TAB should move focus backwards

### feature ideas
- Add sort option, e.g. relevance|votes|date
- Keep track of quota in a data file, inform user when getting close?
- Maybe allow slimmer builds without TUI that only offer --lucky or a more
  primitive prompt interface as implemented in `so-hs`.

#### Endless improvements for the TUI
1. Init with smaller layout depending on initial screen size.
2. Maybe cli `--auto-resize` option that changes layouts at breakpoints.
3. Maybe **[ESC]** cycles layout in the opposite direction? And stops at
   BothColumns?
4. Allow cycling through themes, either found in `~/.config/so/colors/*.toml`
    or just hardcoded ones.
5. Small tray at the bottom with "notifications", e.g. "GitHub Theme loaded!"

**Important note:** Tables are not currently allowed in stackexchange... so the
benefit of incorporating termimad features into a cursive view will not be felt.
But, this is changing [soon](https://meta.stackexchange.com/q/348746).

### resources for later
- [Github Actions example](https://github.com/extrawurst/gitui)
- [another GA example](https://github.com/casey/just)
- [logging example](https://deterministic.space/rust-cli-tips.html)
- More mock user agents
  - `Mozilla/5.0 (Macintosh; Intel Mac OS X 10.7; rv:11.0) Gecko/20100101 Firefox/11.0`
  - `Mozilla/5.0 (X11; Ubuntu; Linux x86_64; rv:22.0) Gecko/20100 101 Firefox/22.0`
  - `Mozilla/5.0 (Windows NT 6.1; rv:11.0) Gecko/20100101 Firefox/11.0`
  - `Mozilla/5.0 (Macintosh; Intel Mac OS X 10_7_4) AppleWebKit/536.5 (KHTML, like Gecko) Chrome/19.0.1084.46 Safari/536.5`
  - `Mozilla/5.0 (Windows; Windows NT 6.1) AppleWebKit/536.5 (KHTML, like Gecko) Chrome/19.0.1084.46 Safari/536.5`

