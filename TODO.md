# TODO

### v0.1.1
1. Touch up CLI and all that

### v0.2.0
3. Termimad interface for viewing questions and answers
1. Add `lucky: bool` to config, but
2. add --lucky and --no-lucky conflicting flags to cli
3. If --lucky, async get 1 result while getting limit results
4. Display with [space] to see more, any other key to exit.

### v0.2.1
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
