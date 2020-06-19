# TODO

[ ] Fix resizing. Something in `wait_for_char` screws up the future cursive app...

### v0.3.0
1. Duckduck go search ftw, e.g.
```
(site:stackoverflow.com OR site:unix.stackexchange.com) what is linux
```
etc.

#### Tech debt and low hanging fruit
1. Use [`par_iter`](https://github.com/rayon-rs/rayon) for text preprocess &
   parsing. In particular the `tui::markdown::preprocess` function should just
   get called on all markdown as soon as its received from stack exchange; this
   is prime for parallelization.
2. Also, we could `par_iter` the initial q&a data to SpannedStrings from the
   start, so that it's not done on the fly...
3. The rest of the questions should really start being fetched while waiting for
   the user to press [Enter]... maybe start with just simple threads?

### Endless future improvements for the TUI
1. Init with smaller layout depending on initial screen size.
2. Maybe cli `--auto-resize` option.
3. Small text at bottom with '?' to bring up key mapping dialog
4. Clean up! remove dupe between ListView and MdView by making a common trait
5. Maybe **[ESC]** cycles layout in the opposite direction? And stops at
   BothColumns?
6. Allow cycling through themes, either found in `~/.config/so/colors/*.toml`
    or just hardcoded ones.
7. Small tray at the bottom with "notifications", e.g. "GitHub Theme loaded!"

### resources for later

#### async
1. start with [this](http://patshaughnessy.net/2020/1/20/downloading-100000-files-using-async-rust) but also see the following gist and thread through the below links to make sure its actually async..
0. breakdown of futures+reqwest [here](https://stackoverflow.com/questions/51044467/how-can-i-perform-parallel-asynchronous-http-get-requests-with-reqwest)
0. general concurrency in rust [info](https://blog.yoshuawuyts.com/streams-concurrency/)
0. [Intro to async rust](http://jamesmcm.github.io/blog/2020/05/06/a-practical-introduction-to-async-programming-in-rust/)
1. Async API calls [tokio](https://stackoverflow.com/a/57770687)
2. Parallel calls against multiple sites [vid](https://www.youtube.com/watch?v=O-LagKc0MPA)
0. OR JUST THREADS [see here](https://rust-lang.github.io/async-book/01_getting_started/02_why_async.html)

#### scraping
6. Google stuff [scraping with reqwest](https://rust-lang-nursery.github.io/rust-cookbook/web/scraping.html))

#### distribution
7. App Distribution
   [cross-platform binaries via travis](https://github.com/rustwasm/wasm-pack/blob/51e6351c28fbd40745719e6d4a7bf26dadd30c85/.travis.yml#L74-L91)
   also see lobster script in this [repo](https://git.sr.ht/~wezm/lobsters).
9. Great tui-rs [example app](https://github.com/SoptikHa2/desed/blob/master/src/ui/tui.rs)
11. general CI & deploy [info](https://rust-cli.github.io/book/tutorial/packaging.html)
12. window binaries deployed via [github actions](https://github.com/rust-av/av-metrics)
13. oh game over [dawg](https://github.com/japaric/trust)

#### ideas
5. Add sort option, e.g. relevance|votes|date
8. Keep track of quota in a data file, inform user when getting close?
