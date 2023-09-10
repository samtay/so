<h1 align="center">
<img width="300px" src="assets/logo.png" />

[![ci][s0]][l0] [![appveyor][s1]][l1] [![crates][s2]][l2] [![MIT][s3]][l3] [![Packaging status](https://repology.org/badge/tiny-repos/so.svg)](https://repology.org/project/so/versions)
</h1>

[s0]: https://github.com/samtay/so/actions/workflows/ci.yml/badge.svg
[l0]: https://github.com/samtay/so/actions/workflows/ci.yml
[s1]: https://ci.appveyor.com/api/projects/status/pu7e63f2sqq6x1iq/branch/master?svg=true
[l1]: https://ci.appveyor.com/project/samtay/so/branch/master
[s2]: https://img.shields.io/crates/v/so.svg
[l2]: https://crates.io/crates/so
[s3]: https://img.shields.io/badge/license-MIT-blue.svg
[l3]: ./LICENSE

<h5 align="center"> A blazing fast terminal interface for StackOverflow written in Rust </h5>

<h1 align="center">

![](assets/demo.gif)

</h1>

## example usage
While I like the acronym *so*, this tool would actually be better described as
*se*: an interface to the StackExchange network. In particular one thing that
differentiates it from [similar](https://github.com/santinic/how2)
[tools](https://github.com/gleitz/howdoi) is that you can simultaneously search
any number of sites in the StackExchange network:
```shell
# search using your default configuration
$ so how do i reverse a list in python

# search for a latex solution
$ so --site tex how to put tilde over character

# use google to search stackoverflow.com, askubuntu.com, and unix.stackexchange.com
$ so -e google -s askubuntu -s stackoverflow -s unix how do i install linux
```

## installation

#### Arch Linux
You can install the AUR package
[so](https://aur.archlinux.org/packages/so/) (tracks latest release)
or
[so-git](https://aur.archlinux.org/packages/so-git/) (tracks master), e.g.
```
yay -S so-git
```

#### FreeBSD
You can install the package [so](https://cgit.freebsd.org/ports/tree/www/so) via
```
pkg install so
```

#### NetBSD
You can install the package [so](https://pkgsrc.se/www/so) via
```
pkgin install so
```

#### MacOS
You can install the homebrew [formula](https://formulae.brew.sh/formula/so)
```
brew install so
```

Alternatively, you can use [MacPorts](https://www.macports.org) to install [so](https://ports.macports.org/port/so/):
```
sudo port install so
```

#### Windows
If you have [scoop](https://scoop.sh/) you can install via the extras bucket:
```shell
# add extras bucket
scoop bucket add extras

# install so
scoop install so
```

#### from source
For any OS you can install the crate [so](https://crates.io/crates/so) directly:
```
# everything but windows
cargo install so

# windows
cargo install so --no-default-features --features windows
```
For more information on the feature flags, see [selecting a
backend](#selecting-a-backend).

#### release binaries
Static binaries are available on the [releases
page](https://github.com/samtay/so/releases) for common Linux, MacOS, and
Windows targets. You can quickly install the one you need to directory `DEST`
with:
```bash
curl --proto '=https' --tlsv1.2 -sSf https://samtay.github.io/so/install.sh \
  | bash -s -- --to DEST
```
Right now I'm only building the most common targets, but in theory it should be
easy to add more, so if you don't see what you are looking for just open an
issue and I can add it. Here's a
list of the [supported
targets](https://github.com/japaric/trust#supported-targets). If you don't know
what you need, you can install [rustc](https://www.rust-lang.org/tools/install)
and open an issue with the output of `rustc -Vv | grep host | cut -d' ' -f2`.

## documentation

### configuration
The configuration files for e.g. a user `Alice` can be found in the following
directories:

- Linux: `/home/alice/.config/so`
- Windows: `C:\Users\Alice\AppData\Roaming\Sam Tay\so`
- MacOS: `/Users/Alice/Library/Preferences/io.Sam-Tay.so`

#### defaults
The `config.yml` file lets you specify your CLI defaults. So if you dislike the
lucky prompt, always search serverfault.com and unix.stackexchange.com, and
want the [fastest search engine](#search-engines), you can set your config file like this:
```yaml
# config.yml
---
api_key: ~
limit: 10
lucky: false
sites:
  - serverfault
  - unix
search_engine: stackexchange
```
Run `so --help` to see your current defaults.

#### themes
In the same directory you'll find `colors.toml` which is self-documented. The
default theme attempts to blend in with your default terminal theme, but you can
change it as necessary. In particular, you may want to change the `highlight_text` if the current selection is difficult to read. There are some themes in the [themes](./themes) directory as well.

#### system clipboard integration
There's a very primitive integration in place to copy the contents of
the currently focused question or answer to the system clipboard. This requires
some command in your PATH that can accept stdin and pipe to the clipboard.
On mac & windows, this will work out of the box with the default set to `pbcopy`
& `clip` respectively. On Linux, I've made the assumption that `xclip` is likely
the most popular, but if you use something else (e.g. `wl-copy` on wayland),
you'll need to set the command directly:
```yaml
# config.yml
---
copy_cmd: copy --option-to-take-stdin
```

#### api keys
If you want to use your own [StackExchange API
Key](https://api.stackexchange.com/docs) you can set it via
```
so --set-api-key <KEY>
```
You can also choose to use no key by editing your configuration to `api_key: ~`.
If for some reason my API key is globally throttled, you can hit the
StackExchange API with no key up to 300 times per day per IP, which I imagine is
fine for most users.

### search engines
The available search engines are StackExchange, DuckDuckGo, and Google.
StackExchange will always be the fastest to search because it doesn't require an
additional request or any HTML parsing; however, it is also very primitive.
~~DuckDuckGo is in second place for speed, as its response HTML is much smaller
than Google's. I've found that it performs well for my queries, so it is the
default search engine.~~

DuckDuckGo [sometimes blocks requests](https://github.com/samtay/so/issues/16), so
it is no longer the default.

### multi-site searching
As stated in the [docs](https://api.stackexchange.com/docs/throttle),

> If a single IP is making more than 30 requests a second, new requests will be dropped.

So, don't go crazy with the multi-site search, since it is all done in parallel.
In particular, if you specify more than 30 sites, SE will likely ban you for a short time.

### selecting a backend
If you're installing from source, you can choose from a number of available
backend rendering engines. Note that the package `default` and `windows` feature
flags do not have an ncurses dependency, for the sake of portability.  The
default backend is [termion](https://github.com/redox-os/termion), a bindless
library in pure Rust which seems to work quite well on Linux, MacOS, BSD, and
Redox.  The windows backend is by default
[crossterm](https://github.com/crossterm-rs/crossterm), and while its level of
support is awesome, it does comes at a price in performance. On my machine, the
app kind of flashes between draws. So if you are on Mac, Linux, or Redox, your
best bet is to compile with default features which uses the termion backend. If
you are on windows, use crossterm, but know it will be slightly jumpy.

If the crossterm folks figure out a fix for allowing ncurses to receive [resize
events](https://github.com/crossterm-rs/crossterm/issues/447), and you have
[ncurses installed](https://github.com/gyscos/cursive/wiki/Install-ncurses) on
your system, then the ncurses and pancurses backends are likely the most
performant.  Just know that *currently* if you choose this option, and you run
the `--lucky` prompt, you won't be able to resize the terminal window while the
TUI is open.

Available backends:

- `termion-backend`
- `ncurses-backend`
- `pancurses-backend`
- `crossterm-backend`

E.g. to use `ncurses-backend`:
```
cargo install so --no-default-features --features ncurses-backend
```

See more information about this choice
[here](https://github.com/gyscos/cursive/wiki/Backends).

## contributing
**Warning**: this was my first time writing Rust and there is very likely some
non-idiomatic and straight up ugly code throughout this project, so don't come
looking here for a good Rust example! That being said, I would love to improve
the codebase. Feel free to check out the [contributing
guidelines](.github/CONTRIBUTING.md) and submit any refactoring issues or pull
requests.

## credits
Credit to my good friend [Charles](http://heyitscharles.com) for logo design.
