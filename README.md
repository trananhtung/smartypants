# smartypants

[![All Contributors](https://img.shields.io/badge/all_contributors-1-orange.svg?style=flat-square)](#contributors-)

[![Crates.io](https://img.shields.io/crates/v/smartypants.svg)](https://crates.io/crates/smartypants)
[![Documentation](https://docs.rs/smartypants/badge.svg)](https://docs.rs/smartypants)
[![CI](https://github.com/trananhtung/smartypants/actions/workflows/ci.yml/badge.svg)](https://github.com/trananhtung/smartypants/actions/workflows/ci.yml)
[![License](https://img.shields.io/crates/l/smartypants.svg)](#license)

**Translate plain ASCII punctuation into smart typographic punctuation** — straight
quotes become curly quotes, `--`/`---` become en/em dashes, and `...` becomes a
horizontal ellipsis. A faithful, zero-dependency, `no_std` Rust port of John Gruber's
[SmartyPants](https://daringfireball.net/projects/smartypants/).

```rust
use smartypants::smartypants;

assert_eq!(smartypants("\"It's\" -- a test..."), "“It’s” — a test…");
assert_eq!(smartypants("He said 'hi'"), "He said ‘hi’");
assert_eq!(smartypants("the '90s"), "the ’90s");
```

## Why smartypants?

`pulldown-cmark` can produce smart punctuation while parsing Markdown, and
[`smart_quotes`](https://docs.rs/smart_quotes) offers an open/close *heuristic* — but
nothing converts the glyphs for arbitrary text the way the original SmartyPants does.
This crate is that missing piece: a standalone educator for prose, templates, and
post-processing pipelines, with no parser to drag along.

```toml
[dependencies]
smartypants = "0.1"
```

## Configuration

```rust
use smartypants::{Backticks, Dashes, SmartyPants};

let sp = SmartyPants::new()
    .dashes(Dashes::OldSchool)     // -- → –, --- → —
    .backticks(Backticks::All)     // also single `…' → ‘…’
    .ellipses(true)
    .html(false);                  // treat input as plain text, not HTML

assert_eq!(sp.apply("a -- b `c'"), "a – b ‘c’");
```

| Item | Purpose |
| --- | --- |
| `smartypants(text)` | Default education (quotes, backticks, em dashes, ellipses), HTML-aware |
| `SmartyPants::new()…apply(text)` | Configurable education |
| `Dashes` | `EmDash` (default), `OldSchool`, `Inverted`, `Off` |
| `Backticks` | `Double` (default), `All`, `Off` |

## Behavior

- **Quotes** are educated by context: a quote after whitespace, an opening bracket,
  a dash, or another opening quote opens; otherwise it closes. Contractions (`don't`),
  decades (`'90s`), and common elisions (`'twas`, `'em`, `'n'`) become apostrophes —
  an improvement over vanilla SmartyPants, which mangles those into opening quotes.
- **Dashes** default to em (`--` and `---` → `—`); `Dashes::OldSchool` and
  `Dashes::Inverted` distinguish en (`–`) and em (`—`) dashes.
- **HTML-aware** by default: the contents of tags and of protected elements (`pre`,
  `code`, `kbd`, `script`, `style`, `math`, `textarea`) are left untouched. Use
  `.html(false)` to educate the whole string as plain text.
- **Idempotent**: already-curly text is left unchanged, so it's safe to run twice.
- Operates on character boundaries, so it's safe on any UTF-8 input.

## `no_std`

The crate is `#![no_std]` (it only needs `alloc` for `String`) and has zero
dependencies.

## Limitations

Like the original, this is a heuristic, not a parser:

- A single quote can't always tell an elision (`'cause` = because) from a quoted
  word (`'cause'`); ambiguous cases resolve to an apostrophe.
- `Backticks::Double` maps `''` to `”` (the TeX convention), so literal double
  apostrophes and inch marks are affected — use `Backticks::Off` to opt out.
- Mismatched or malformed protected tags (e.g. `<code>…</b>`) may leave the
  remaining text uneducated, matching SmartyPants' behavior.

## Contributors ✨

This project follows the [all-contributors](https://github.com/all-contributors/all-contributors) specification. Contributions of any kind are welcome — code, docs, bug reports, ideas, reviews! See the [emoji key](https://allcontributors.org/docs/en/emoji-key) for how each contribution is recognized, and open a PR or issue to get involved.

Thanks goes to these wonderful people:

<!-- ALL-CONTRIBUTORS-LIST:START - Do not remove or modify this section -->
<!-- prettier-ignore-start -->
<!-- markdownlint-disable -->
<table>
  <tbody>
    <tr>
      <td align="center" valign="top" width="14.28%"><a href="https://github.com/trananhtung"><img src="https://avatars.githubusercontent.com/u/30992229?v=4?s=100" width="100px;" alt="Tung Tran"/><br /><sub><b>Tung Tran</b></sub></a><br /><a href="https://github.com/trananhtung/smartypants/commits?author=trananhtung" title="Code">💻</a> <a href="#maintenance-trananhtung" title="Maintenance">🚧</a></td>
    </tr>
  </tbody>
</table>

<!-- markdownlint-restore -->
<!-- prettier-ignore-end -->

<!-- ALL-CONTRIBUTORS-LIST:END -->

## License

Licensed under either of [Apache-2.0](LICENSE-APACHE) or [MIT](LICENSE-MIT) at
your option.
