# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.4.10]

#### Fixed

- Cloudflare blocking `reqwest`s ([#46](https://github.com/samtay/so/issues/46))

#### Added

- Some limited logging
- Mac ports install option ([#31](https://github.com/samtay/so/pulls/31))

## [0.4.9]

#### Added

- Allow `o` keybinding from the lucky prompt.
- Permanent help text ([#24](https://github.com/samtay/so/issues/24)).

#### Fixed

- Hardcoded stackoverflow link

## [0.4.8]

#### Added

- Keybinding: Press `o` to open the current answer in the default browser

## [0.4.7]

#### Added

- Keybinding: Press `y` to yank to system clipboard
- New config field for specifying command to copy to system clipboard

#### Changed

- Switch linux/mac builds from Travis to GitHub Actions.

## [0.4.6]

#### Changed

- Use Google as the default search engine, due to issues with DuckDuckGo.

## [0.4.5]

#### Added

- NetBSD installation option. Thanks **voidpin**.

#### Fixed

- Google parser went out of date
- Panic from termimad ([#5](https://github.com/samtay/so/issues/5))

## [0.4.3]

#### Fixed

- Build issue caused by syn dependency ([#13](https://github.com/samtay/so/issues/13))
- Panic from termimad ([#8](https://github.com/samtay/so/issues/8))

#### Added

- Windows installation option
  [lukesampson/scoop-extras#4376](https://github.com/lukesampson/scoop-extras/pull/4376).
  Thanks [@laralex](https://github.com/laralex)!

## [0.4.1]

#### Added

- GitHub Action to bump homebrew-core formula

#### Changed

- Homebrew installation method: core
  [formula](https://formulae.brew.sh/formula/so) now available

## [0.4.0]

#### Added

- _Keybinding_: Press `q` to quit ([#1](https://github.com/samtay/so/pull/1)).
  Thanks [@zynaxsoft](https://github.com/zynaxsoft)!
- Arch Linux installation options: [so](https://aur.archlinux.org/packages/so/)
  and [so-git](https://aur.archlinux.org/packages/so-git/)
- Homebrew installation option: [samtay/tui/so](https://github.com/samtay/homebrew-tui)
- This changelog

#### Changed

- Default `highlight_text` is now `black`

## [0.3.6]

#### Added

- CLI spinner

#### Fixed

- Fragmented highlighting styles

## [0.3.5]

- (Unofficial) initial passable release

[Unreleased]: (https://github.com/samtay/so/compare/v0.4.9...HEAD)
[0.4.9]: (https://github.com/samtay/so/compare/v0.4.8...v0.4.9)
[0.4.8]: (https://github.com/samtay/so/compare/v0.4.7...v0.4.8)
[0.4.7]: (https://github.com/samtay/so/compare/v0.4.6...v0.4.7)
[0.4.6]: (https://github.com/samtay/so/compare/v0.4.5...v0.4.6)
[0.4.5]: (https://github.com/samtay/so/compare/v0.4.3...v0.4.5)
[0.4.3]: (https://github.com/samtay/so/compare/v0.4.1...v0.4.3)
[0.4.1]: (https://github.com/samtay/so/compare/v0.4.0...v0.4.1)
[0.4.0]: (https://github.com/samtay/so/compare/v0.3.6...v0.4.0)
[0.3.6]: (https://github.com/samtay/so/compare/v0.3.5...v0.3.6)
[0.3.5]: (https://github.com/samtay/so/compare/030cd70...v0.3.5)
