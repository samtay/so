# TODO

## TUI considerations
Going with cursive because it is way more flexible than tui-rs.
**Important note** Tables are not currently allowed in stackexchange... so the
benefit of incorporating termimad features will not be felt. But, this is
changing [soon](https://meta.stackexchange.com/q/348746).

### v0.2.0
0. Cursive interface for viewing questions and answers
1. Focus in one of four areas
3. arrows and vim bindings to manipulate lists and scroll, depending on focus
2. `/` to bring up new search prompt
1. For sure: Use custom code palette nodes in theme.toml to style markdown
1. Layout cycling?
0. use [par_iter](https://github.com/rayon-rs/rayon) for parsing markdown and manual <kbd> removers, etc.
1. Add `lucky: bool` to config, but
2. add --lucky and --no-lucky conflicting flags to cli
3. If --lucky, async get 1 result while getting limit results
4. Display with [space] to see more, any other key to exit.
1. maybe <query> is optional, and leaving blank starts up TUI?

### v0.2.1
1. Add colors.yml configuration

### v0.2.2
1. Site can be multiple
2. do tokio async on SE api
3. add warning to README about throttling on excessive requests

### v0.3.0
1. Duckduck go search ftw, e.g.
```
(site:stackoverflow.com OR site:unix.stackexchange.com) what is linux
```
etc.

### resources for later
0. [Intro to async rust](http://jamesmcm.github.io/blog/2020/05/06/a-practical-introduction-to-async-programming-in-rust/)
1. Async API calls [tokio](https://stackoverflow.com/a/57770687)
2. Parallel calls against multiple sites [vid](https://www.youtube.com/watch?v=O-LagKc0MPA)
0. OR JUST THREADS [see here](https://rust-lang.github.io/async-book/01_getting_started/02_why_async.html)
3. [config mgmt](https://github.com/rust-cli/confy) or just use directories
5. Add sort option, e.g. relevance|votes|date
6. Google stuff [scraping with reqwest](https://rust-lang-nursery.github.io/rust-cookbook/web/scraping.html))
8. Keep track of quota in a data file, inform user when getting close?
7. App Distribution
   [cross-platform binaries via travis](https://github.com/rustwasm/wasm-pack/blob/51e6351c28fbd40745719e6d4a7bf26dadd30c85/.travis.yml#L74-L91)
   also see lobster script in this [repo](https://git.sr.ht/~wezm/lobsters).
9. Great tui-rs [example app](https://github.com/SoptikHa2/desed/blob/master/src/ui/tui.rs)
10 nah look at [termimad example](https://github.com/Canop/whalespotter)
