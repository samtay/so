# so

**Note:** under development, not ready for prime time.

# documentation

### api keys
According to the [StackExchange
docs](https://api.stackexchange.com/docs/throttle), most users should be fine
without generating a personal API key (10k requests per IP per day). If you do
run into throttling issues, get a key
[here](https://stackapps.com/apps/oauth/register) and tell `so` to use it:
```
so --set-api-key <KEY>
```

### multi-site searching
As stated in the docs linked above,

> If a single IP is making more than 30 requests a second, new requests will be dropped.

So, don't go crazy with the multi-site search, since it is all done in parallel.
In particular, if you specify more than 30 sites, SE will likely ban you for a short time.

### selecting a backend
Crossterm's level of support is awesome, but it comes at a price. On my machine,
the app kind of flashes between draws quite a bit. If you are on Mac, Linux, or
Redox, your best bet is to compile with default features which uses the
termion backend. If you are on windows, use crossterm, but know it will be
slightly jumpy.

If the crossterm folks figure out a fix for [allowing ncurses to receive resize events](),
and you have ncurses installed on your system, then you should use the
ncurses backend, or the pancurses backend if you are on Windows. Just know that
currently if you choose this option, you won't be able to resize the terminal
window while the TUI is open.

default = ["cursive/termion-backend"]
ncurses-backend = ["cursive/ncurses-backend"]
pancurses-backend = ["cursive/pancurses-backend"]
crossterm-backend = ["cursive/crossterm-backend"]

# notes to self

### async considerations
Implemented async with tokio in ec92f93, but unclear if this is necessary. For
< 10 simultaneous network requests, it might be better and simpler to just use
rayon (i.e. OS threads).

### TUI considerations
Going with cursive because it is way more flexible than tui-rs.
**Important note** Tables are not currently allowed in stackexchange... so the
benefit of incorporating termimad features will not be felt. But, this is
changing [soon](https://meta.stackexchange.com/q/348746).

### to stress test
Produces a long answer with noticeable pause on markdown view:
```
cargo run -- --site stackoverflow --site serverfault how do I exit Vim
```

### to troubleshoot
```
export RUST_BACKTRACE=full
cargo run -- how do I exit Vim > test.txt 2>&1
```




**Remove this** Recall my api key is: `8o9g7WcfwnwbB*Qp4VsGsw((`
