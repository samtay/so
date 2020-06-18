# so

**Note:** under development, not ready for prime time.

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



**Remove this** Recall my api key is: `8o9g7WcfwnwbB*Qp4VsGsw((`
