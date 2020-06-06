# TODO

### initial release
0. Install sites when file not found
0. Implement --update-sites command
3. Parse markdown (`pulldown_cmark`)
4. Maybe default --validate-sites off (parsing 30k file a big hit)
5. Print to stderr in [style](https://github.com/BurntSushi/termcolor)

### resources for later
0. [Intro to async rust](http://jamesmcm.github.io/blog/2020/05/06/a-practical-introduction-to-async-programming-in-rust/)
1. Async API calls [tokio](https://stackoverflow.com/a/57770687)
2. Parallel calls against multiple sites [vid](https://www.youtube.com/watch?v=O-LagKc0MPA)
0. OR JUST THREADS [see here](https://rust-lang.github.io/async-book/01_getting_started/02_why_async.html)
3. [config mgmt](https://github.com/rust-cli/confy) or just use directories
4. Test if pre-made filter can be used for various api keys
5. Add sort option, e.g. relevance|votes|date
6. Google stuff [scraping with reqwest](https://rust-lang-nursery.github.io/rust-cookbook/web/scraping.html))
8. Keep track of quota in a data file, inform user when getting close?
7. App Distribution
   [cross-platform binaries via travis](https://github.com/rustwasm/wasm-pack/blob/51e6351c28fbd40745719e6d4a7bf26dadd30c85/.travis.yml#L74-L91)
   also see lobster script in this [repo](https://git.sr.ht/~wezm/lobsters).
