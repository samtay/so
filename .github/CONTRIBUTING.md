## About the Project
This is designed to be a straightforward, simple tool. At the moment, I'm not
very interested in adding the ability to interact more heavily with
stackexchange (such as answering or voting), but adding user authentication is
not off the table.  The main priorities are:

- **Portability**: namely, maintain the same level of support as
[crossterm](https://github.com/crossterm-rs/crossterm)
- **Speed**: the HTTP requests should be the only limiting factor
- **Intuition**: any keybindings should be intuitive enough to guess (for VIM
users anyway)

## Getting Started
Familiarize yourself with both
[crossterm](https://github.com/crossterm-rs/crossterm)
and
[cursive](https://github.com/gyscos/Cursive)
since much of the application code is written against those libraries. In
particular it would be helpful to be able to recognize when a bug is coming from
this application or one of those underlying libraries.

## Bugs

When filing an issue, please mention the OS, the terminal, the offending CLI
arguments, and add accompanying screenshots if applicable.

## Features

As long as your feature request fits with the priorities above, feel free to add
the suggestion.

## Pull Requests

I'm still new to Rust, so I definitely welcome any refactoring contributions! I
just ask that you also include an explanation for such changes. Of course, help
with bugs and approved features is also much appreciated. Just make sure you've
formatted code with [rustfmt](https://github.com/rust-lang/rustfmt). Sooner or
later this will be added to the CI testing.
