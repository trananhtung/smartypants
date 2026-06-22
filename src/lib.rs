//! # smartypants — smart typographic punctuation
//!
//! Translate plain ASCII punctuation into "smart" typographic punctuation:
//! straight quotes become curly quotes, `--`/`---` become en/em dashes, and `...`
//! becomes a horizontal ellipsis. A faithful, zero-dependency, `no_std` port of
//! John Gruber's [SmartyPants](https://daringfireball.net/projects/smartypants/).
//!
//! ```
//! use smartypants::smartypants;
//! assert_eq!(smartypants("\"It's\" -- a test..."), "“It’s” — a test…");
//! ```
//!
//! By default the input is treated as HTML: the contents of tags and of protected
//! elements (`pre`, `code`, `kbd`, `script`, `style`, `math`, `textarea`) are left
//! untouched. Use [`SmartyPants::html`]`(false)` for plain text.
//!
//! ```
//! use smartypants::{Backticks, Dashes, SmartyPants};
//! let sp = SmartyPants::new().dashes(Dashes::OldSchool).backticks(Backticks::All);
//! assert_eq!(sp.apply("a -- b"), "a – b");
//! ```
//!
//! ## Behavior
//!
//! - Quotes are educated by context: a quote after whitespace, an opening bracket,
//!   a dash, or another opening quote opens; otherwise it closes. Contractions
//!   (`don't`), decades (`'90s`), and common elisions (`'twas`, `'em`, `'n'`) become
//!   apostrophes.
//! - Dashes default to em (`--` and `---` → `—`); [`Dashes::OldSchool`] and
//!   [`Dashes::Inverted`] distinguish en and em dashes.
//! - The transformation is idempotent: already-curly text is left unchanged.

#![no_std]
#![doc(html_root_url = "https://docs.rs/smartypants/0.1.0")]

extern crate alloc;

use alloc::borrow::ToOwned;
use alloc::string::String;
use alloc::vec::Vec;

// Typographic characters produced by the educator.
const LDQUO: char = '\u{201c}'; // “ left double quote
const RDQUO: char = '\u{201d}'; // ” right double quote
const LSQUO: char = '\u{2018}'; // ‘ left single quote
const RSQUO: char = '\u{2019}'; // ’ right single quote / apostrophe
const ENDASH: char = '\u{2013}'; // – en dash
const EMDASH: char = '\u{2014}'; // — em dash

/// Common English elisions whose leading apostrophe should be a closing single
/// quote (the text *after* the apostrophe, lowercased).
const ELISIONS: &[&str] = &[
    "tis", "twas", "twere", "twould", "til", "bout", "cause", "round", "em", "n", "cept", "gainst",
    "neath", "nother", "fraid", "mongst", "tween",
];

/// HTML elements whose text content must not be educated.
const PROTECTED: &[&str] = &["pre", "code", "kbd", "script", "style", "math", "textarea"];

/// How `--` and `---` are converted.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Dashes {
    /// Leave dashes unchanged.
    Off,
    /// `--` and `---` both become an em dash (`—`). The default.
    #[default]
    EmDash,
    /// `--` → en dash (`–`), `---` → em dash (`—`).
    OldSchool,
    /// `--` → em dash (`—`), `---` → en dash (`–`).
    Inverted,
}

/// How backtick-style quotes are converted.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Backticks {
    /// Leave backticks unchanged.
    Off,
    /// Two backticks become `“` and `''` becomes `”`. The default.
    #[default]
    Double,
    /// Also convert a single `` ` `` → `‘` and `'` → `’`.
    All,
}

/// Builder configuring which transformations to apply.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SmartyPants {
    quotes: bool,
    backticks: Backticks,
    dashes: Dashes,
    ellipses: bool,
    html: bool,
}

impl Default for SmartyPants {
    fn default() -> Self {
        Self {
            quotes: true,
            backticks: Backticks::Double,
            dashes: Dashes::EmDash,
            ellipses: true,
            html: true,
        }
    }
}

impl SmartyPants {
    /// Default configuration: educate quotes, backtick-style quotes, em dashes,
    /// ellipses, treating the input as HTML.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Enable or disable curly-quote education (default `true`).
    #[must_use]
    pub fn quotes(mut self, on: bool) -> Self {
        self.quotes = on;
        self
    }

    /// Set how backtick-style quotes are converted (default [`Backticks::Double`]).
    #[must_use]
    pub fn backticks(mut self, mode: Backticks) -> Self {
        self.backticks = mode;
        self
    }

    /// Set how dashes are converted (default [`Dashes::EmDash`]).
    #[must_use]
    pub fn dashes(mut self, mode: Dashes) -> Self {
        self.dashes = mode;
        self
    }

    /// Enable or disable ellipsis education (default `true`).
    #[must_use]
    pub fn ellipses(mut self, on: bool) -> Self {
        self.ellipses = on;
        self
    }

    /// Treat the input as HTML, skipping tags and protected elements (default
    /// `true`). Set to `false` to educate the whole string as plain text.
    #[must_use]
    pub fn html(mut self, on: bool) -> Self {
        self.html = on;
        self
    }

    /// Apply the configured transformations to `text`.
    #[must_use]
    pub fn apply(&self, text: &str) -> String {
        if self.html {
            self.apply_html(text)
        } else {
            let mut prev = None;
            self.educate_run(text, &mut prev)
        }
    }

    /// HTML-aware pass: educate text, leave tags and protected-element content alone.
    fn apply_html(&self, text: &str) -> String {
        let chars: Vec<char> = text.chars().collect();
        let mut out = String::with_capacity(text.len());
        let mut prev: Option<char> = None;
        let mut protected = 0usize;
        let mut i = 0;
        while i < chars.len() {
            if chars[i] == '<' && is_tag_start(&chars, i) {
                let (tag, next) = read_tag(&chars, i);
                if let Some((name, closing)) = tag_name(&chars, i, next) {
                    if PROTECTED.contains(&name.as_str()) {
                        if closing {
                            protected = protected.saturating_sub(1);
                        } else if !is_self_closing(&chars, next) {
                            protected += 1;
                        }
                    }
                }
                out.push_str(&tag);
                i = next;
            } else {
                let start = i;
                loop {
                    while i < chars.len() && chars[i] != '<' {
                        i += 1;
                    }
                    if i >= chars.len() || is_tag_start(&chars, i) {
                        break; // real tag or end of input
                    }
                    i += 1; // a '<' that doesn't start a tag is literal text
                }
                let run: String = chars[start..i].iter().collect();
                if protected > 0 {
                    out.push_str(&run);
                    if let Some(c) = run.chars().last() {
                        prev = Some(c); // thread context through skipped content
                    }
                } else {
                    out.push_str(&self.educate_run(&run, &mut prev));
                }
            }
        }
        out
    }

    /// Educate a single run of text, threading quote context through `prev`.
    fn educate_run(&self, run: &str, prev: &mut Option<char>) -> String {
        let mut s = run.to_owned();
        if self.ellipses {
            s = educate_ellipses(&s);
        }
        s = educate_dashes(&s, self.dashes);
        s = educate_backticks(&s, self.backticks);
        if self.quotes {
            educate_quotes(&s, prev)
        } else {
            if let Some(c) = s.chars().last() {
                *prev = Some(c);
            }
            s
        }
    }
}

/// Apply the default SmartyPants transformations to `text`.
///
/// ```
/// assert_eq!(smartypants::smartypants("\"x\""), "“x”");
/// ```
#[must_use]
pub fn smartypants(text: &str) -> String {
    SmartyPants::new().apply(text)
}

// ---------------------------------------------------------------------------
// Individual transforms
// ---------------------------------------------------------------------------

/// Replace `...` and `. . .` with a horizontal ellipsis.
fn educate_ellipses(s: &str) -> String {
    s.replace("...", "\u{2026}").replace(". . .", "\u{2026}")
}

/// Replace dash runs according to `mode`. `---` is handled before `--`.
fn educate_dashes(s: &str, mode: Dashes) -> String {
    match mode {
        Dashes::Off => s.to_owned(),
        Dashes::EmDash => s.replace("---", "\u{2014}").replace("--", "\u{2014}"),
        Dashes::OldSchool => s.replace("---", "\u{2014}").replace("--", "\u{2013}"),
        Dashes::Inverted => s.replace("---", "\u{2013}").replace("--", "\u{2014}"),
    }
}

/// Replace backtick-style quotes according to `mode`.
fn educate_backticks(s: &str, mode: Backticks) -> String {
    match mode {
        Backticks::Off => s.to_owned(),
        Backticks::Double => s.replace("``", "\u{201c}").replace("''", "\u{201d}"),
        Backticks::All => s
            .replace("``", "\u{201c}")
            .replace("''", "\u{201d}")
            .replace('`', "\u{2018}")
            .replace('\'', "\u{2019}"),
    }
}

/// Educate straight quotes into curly quotes, deciding open vs. close from the
/// previously emitted character (threaded through `prev` across runs).
fn educate_quotes(s: &str, prev: &mut Option<char>) -> String {
    let chars: Vec<char> = s.chars().collect();
    let mut out = String::with_capacity(s.len());
    let mut prev_out = *prev;
    for (i, &c) in chars.iter().enumerate() {
        let educated = match c {
            // An immediately-repeated same-glyph quote closes (e.g. the empty pair "").
            '"' if prev_out == Some(LDQUO) => RDQUO,
            '"' if is_open_context(prev_out) => LDQUO,
            '"' => RDQUO,
            '\'' if prev_out == Some(LSQUO) => RSQUO,
            '\'' => decide_single(prev_out, &chars, i),
            other => other,
        };
        out.push(educated);
        prev_out = Some(educated);
    }
    *prev = prev_out;
    out
}

/// Whether a quote at this position should open (vs. close), based on the
/// preceding emitted character.
fn is_open_context(prev: Option<char>) -> bool {
    match prev {
        None => true,
        Some(c) => {
            c.is_whitespace()
                || c == '('
                || c == '['
                || c == '{'
                || c == EMDASH
                || c == ENDASH
                || c == LDQUO
                || c == LSQUO
        }
    }
}

/// Decide the curly form of a single straight quote at `chars[i]`.
fn decide_single(prev: Option<char>, chars: &[char], i: usize) -> char {
    if !is_open_context(prev) {
        return RSQUO; // apostrophe after a word: don't, dogs'
    }
    match chars.get(i + 1).copied() {
        Some(n) if n.is_ascii_digit() => RSQUO, // decade: '90s, '85
        Some(n) if n.is_alphabetic() && is_elision(chars, i + 1) => RSQUO, // 'twas, 'em
        Some(n) if n.is_alphabetic() => LSQUO,  // opening: 'quoted'
        _ => RSQUO,                             // standalone ' in open context
    }
}

/// Whether the maximal alphabetic run at `start` is a known elision.
fn is_elision(chars: &[char], start: usize) -> bool {
    let mut word = String::new();
    let mut j = start;
    while j < chars.len() && chars[j].is_alphabetic() {
        word.extend(chars[j].to_lowercase());
        j += 1;
    }
    ELISIONS.contains(&word.as_str())
}

// ---------------------------------------------------------------------------
// Minimal HTML tag scanning
// ---------------------------------------------------------------------------

/// Whether `chars[i] == '<'` plausibly begins a tag (followed by a letter, `/`,
/// `!`, or `?`). A `<` followed by anything else — `5 < 10`, `a < b` — is text.
fn is_tag_start(chars: &[char], i: usize) -> bool {
    match chars.get(i + 1) {
        Some(&c) => c.is_ascii_alphabetic() || c == '/' || c == '!' || c == '?',
        None => false,
    }
}

/// Read a tag (or comment) starting at `chars[i] == '<'`, returning its text and
/// the index just past it.
fn read_tag(chars: &[char], i: usize) -> (String, usize) {
    if chars[i..].starts_with(&['<', '!', '-', '-']) {
        // Search for the closing `-->` from just after `<!`, so abrupt-closing
        // forms `<!-->` and `<!--->` terminate instead of running to EOF.
        let mut j = i + 2;
        while j < chars.len() {
            if chars[j..].starts_with(&['-', '-', '>']) {
                j += 3;
                return (chars[i..j].iter().collect(), j);
            }
            j += 1;
        }
        return (chars[i..].iter().collect(), chars.len());
    }
    let mut j = i + 1;
    while j < chars.len() && chars[j] != '>' {
        let c = chars[j];
        if c == '"' || c == '\'' {
            // Skip a quoted attribute value so a '>' inside it doesn't end the tag.
            j += 1;
            while j < chars.len() && chars[j] != c {
                j += 1;
            }
        }
        if j < chars.len() {
            j += 1;
        }
    }
    if j < chars.len() {
        j += 1; // include the closing '>'
    }
    (chars[i..j].iter().collect(), j)
}

/// Parse the lowercased element name and whether it is a closing tag, for the
/// tag occupying `chars[start..end]`. Returns `None` for comments and doctypes.
fn tag_name(chars: &[char], start: usize, end: usize) -> Option<(String, bool)> {
    let mut k = start + 1;
    if k >= end || chars[k] == '!' {
        return None;
    }
    let closing = chars[k] == '/';
    if closing {
        k += 1;
    }
    let mut name = String::new();
    while k < end && chars[k].is_ascii_alphanumeric() {
        name.extend(chars[k].to_lowercase());
        k += 1;
    }
    if name.is_empty() {
        None
    } else {
        Some((name, closing))
    }
}

/// Whether the tag ending at `end` is self-closing (`.../>`).
fn is_self_closing(chars: &[char], end: usize) -> bool {
    end >= 2 && chars[end - 1] == '>' && chars[end - 2] == '/'
}
