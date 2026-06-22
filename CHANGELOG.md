# Changelog

All notable changes to this project are documented here. The format is based on
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project
adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2026-06-22

### Added

- Initial release.
- `smartypants` — apply the default transformations (curly quotes, backticks, em
  dashes, ellipses), HTML-aware.
- `SmartyPants` builder — toggle `quotes`/`ellipses`, choose `dashes` (`EmDash`,
  `OldSchool`, `Inverted`, `Off`) and `backticks` (`Double`, `All`, `Off`), and
  switch between HTML-aware and plain-text modes via `html`.
- Context-aware quote education with apostrophe handling for contractions, decades
  (`'90s`), and common elisions (`'twas`, `'em`, `'n'`).
- Zero dependencies; `#![no_std]` (requires `alloc`).

[0.1.0]: https://github.com/trananhtung/smartypants/releases/tag/v0.1.0
