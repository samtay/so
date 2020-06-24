[![Build Status (travis)](https://travis-ci.org/samtay/so.svg?branch=master)](https://travis-ci.org/samtay/so)
[![Build status](https://ci.appveyor.com/api/projects/status/pu7e63f2sqq6x1iq/branch/master?svg=true)](https://ci.appveyor.com/project/samtay/so/branch/master)

# so
A terminal interface for StackOverflow.

**Note**: still working out some kinks. Initial release not published just yet.
**<Insert GIF here>**

## example usage
While I like the acronym *so*, this tool would actually be better described as
*se*: an interface to the StackExchange network. In particular one thing that
differentiates it from [similar](https://github.com/santinic/how2)
[tools](https://github.com/gleitz/howdoi) is that you can simultaneously search
any number of sites in the StackExchange network:
```shell
# use default configured sites, e.g. [stackoverflow.com]
$ so how do i reverse a list in python

# search for a latex solution
$ so --site tex how to put tilde over character

# use google to search stackoverflow.com, askubuntu.com, and unix.stackexchange.com
$ so -e google -s askubuntu -s stackoverflow -s unix how do i install linux
```

## installation
The quickest installation method is to download the appropriate
binary from the [release artifacts](TODO link). Right now I'm only building the
most common targets, but in theory it should be easy to add more, so if you
don't see what you are looking for just open an issue and I can add it. Here's a
list of the [supported
targets](https://github.com/japaric/trust#supported-targets).  You can quickly
install the binary for your OS with:
```shell
$ curl -LSfs https://samtay.github.io/so/install.sh | \
    sh -s -- --git samtay/so
```

#### cargo
```
cargo install so
```

## documentation

### api keys
If you want to use your own [StackExchange API
Key](https://api.stackexchange.com/docs) you can set it via
```
so --set-api-key <KEY>
```
You can also choose to use no key by editing your configuration to `api_key: ~`.
If for some reason my API key is globally throttled, you can hit the
StackExchange API with no key up to 300 times per day per IP, which I imagine is
fine for most users.

### multi-site searching
As stated in the [docs](https://api.stackexchange.com/docs/throttle),

> If a single IP is making more than 30 requests a second, new requests will be dropped.

So, don't go crazy with the multi-site search, since it is all done in parallel.
In particular, if you specify more than 30 sites, SE will likely ban you for a short time.

### selecting a backend
If you're installing from source, you can choose from a number of available
backend rendering engines. Note that the package `default` and `windows` feature
flags do not have an ncurses dependency, for the sake of simplicity.  The
default backend is [termion](https://github.com/redox-os/termion), a bindless
library in pure Rust which seems to work quite well on Linux, MacOS, BSD, and
Redox.  The windows backend is by default
[crossterm](https://github.com/crossterm-rs/crossterm), and while its level of
support is awesome, it does comes at a price in performance. On my machine, the
app kind of flashes between draws quite a bit. So if you are on Mac, Linux, or
Redox, your best bet is to compile with default features which uses the termion
backend. If you are on windows, use crossterm, but know it will be slightly
jumpy.

If the crossterm folks figure out a fix for allowing ncurses to receive [resize
events](https://github.com/crossterm-rs/crossterm/issues/447), and you have
[ncurses installed](https://github.com/gyscos/cursive/wiki/Install-ncurses) on
your system, then the ncurses and pancurses backends will also work well.
Just know that *currently* if you choose this option, and you run the `--lucky`
prompt, you won't be able to resize the terminal window while the TUI is open.

Available backends:
- `termion-backend`
- `ncurses-backend`
- `pancurses-backend`
- `crossterm-backend`

E.g. to use `ncurses-backend`:
```
cargo install so --no-default-features --features ncurses-backend
```

See more information on this choice
[here](https://github.com/gyscos/cursive/wiki/Backends).

## contributions
This was my first time writing Rust, and I wrote about it
[here](https://samtay.github.io/TODO) if you're into that sort of thing.
I just want to put out a **warning** that there is very likely some non-idiomatic
and straight up ugly code throughout this project, so don't come looking here
for a good Rust example! That being said, I would love to improve the codebase
so if you have any refactoring contributions feel free to send me a PR, but
please also accompany them with a short explanation.
