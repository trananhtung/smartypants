//! End-to-end behavioral spec for the public `smartypants` API.

use smartypants::{smartypants, Backticks, Dashes, SmartyPants};

// ---------------------------------------------------------------------------
// Double quotes
// ---------------------------------------------------------------------------

#[test]
fn double_quotes_open_and_close() {
    assert_eq!(smartypants("\"Hello\""), "\u{201c}Hello\u{201d}");
    assert_eq!(smartypants("say \"hi\"."), "say \u{201c}hi\u{201d}.");
    assert_eq!(smartypants("(\"x\")"), "(\u{201c}x\u{201d})");
    assert_eq!(
        smartypants("\"one\" \"two\""),
        "\u{201c}one\u{201d} \u{201c}two\u{201d}"
    );
}

// ---------------------------------------------------------------------------
// Single quotes / apostrophes
// ---------------------------------------------------------------------------

#[test]
fn apostrophes_in_contractions() {
    assert_eq!(smartypants("don't"), "don\u{2019}t");
    assert_eq!(
        smartypants("it's a dog's life"),
        "it\u{2019}s a dog\u{2019}s life"
    );
}

#[test]
fn single_quotes_open_and_close() {
    assert_eq!(smartypants("He said 'hi'"), "He said \u{2018}hi\u{2019}");
    assert_eq!(smartypants("'quoted'"), "\u{2018}quoted\u{2019}");
}

#[test]
fn nested_double_then_single_quotes() {
    // "'Quoted'" → “‘Quoted’”
    assert_eq!(
        smartypants("\"'Quoted'\""),
        "\u{201c}\u{2018}Quoted\u{2019}\u{201d}"
    );
}

#[test]
fn decade_abbreviation_uses_apostrophe() {
    assert_eq!(smartypants("the '90s"), "the \u{2019}90s");
}

#[test]
fn leading_elisions_use_apostrophe() {
    // SmartyPants vanilla mangles these to opening quotes; we treat common
    // elisions as apostrophes (closing single quote).
    assert_eq!(smartypants("'Twas the night"), "\u{2019}Twas the night");
    assert_eq!(smartypants("get 'em"), "get \u{2019}em");
    assert_eq!(smartypants("rock 'n' roll"), "rock \u{2019}n\u{2019} roll");
}

// ---------------------------------------------------------------------------
// Backticks
// ---------------------------------------------------------------------------

#[test]
fn backtick_double_quotes() {
    assert_eq!(smartypants("``Hello''"), "\u{201c}Hello\u{201d}");
}

#[test]
fn backtick_all_mode_handles_singles() {
    assert_eq!(
        SmartyPants::new().backticks(Backticks::All).apply("`x'"),
        "\u{2018}x\u{2019}"
    );
}

#[test]
fn backticks_off_leaves_them() {
    assert_eq!(
        SmartyPants::new().backticks(Backticks::Off).apply("``x''"),
        "``x\u{2019}\u{2019}" // '' still educated by the quotes pass as closing
    );
}

// ---------------------------------------------------------------------------
// Dashes
// ---------------------------------------------------------------------------

#[test]
fn dashes_default_em() {
    assert_eq!(smartypants("a -- b"), "a \u{2014} b");
    assert_eq!(smartypants("a --- b"), "a \u{2014} b");
}

#[test]
fn dashes_old_school() {
    let sp = SmartyPants::new().dashes(Dashes::OldSchool);
    assert_eq!(sp.apply("a -- b"), "a \u{2013} b"); // en
    assert_eq!(sp.apply("a --- b"), "a \u{2014} b"); // em
}

#[test]
fn dashes_inverted() {
    let sp = SmartyPants::new().dashes(Dashes::Inverted);
    assert_eq!(sp.apply("a -- b"), "a \u{2014} b"); // em
    assert_eq!(sp.apply("a --- b"), "a \u{2013} b"); // en
}

#[test]
fn dashes_off() {
    assert_eq!(
        SmartyPants::new().dashes(Dashes::Off).apply("a -- b"),
        "a -- b"
    );
}

// ---------------------------------------------------------------------------
// Ellipses
// ---------------------------------------------------------------------------

#[test]
fn ellipses() {
    assert_eq!(smartypants("Wait..."), "Wait\u{2026}");
    assert_eq!(smartypants("Wait. . ."), "Wait\u{2026}");
}

// ---------------------------------------------------------------------------
// Combined
// ---------------------------------------------------------------------------

#[test]
fn combined_transformations() {
    assert_eq!(
        smartypants("\"It's\" -- a test..."),
        "\u{201c}It\u{2019}s\u{201d} \u{2014} a test\u{2026}"
    );
}

// ---------------------------------------------------------------------------
// HTML awareness
// ---------------------------------------------------------------------------

#[test]
fn html_skips_tags_but_educates_text() {
    assert_eq!(smartypants("<p>\"Hi\"</p>"), "<p>\u{201c}Hi\u{201d}</p>");
    // attribute values inside the tag must not be touched
    assert_eq!(
        smartypants("<a href=\"x\">\"y\"</a>"),
        "<a href=\"x\">\u{201c}y\u{201d}</a>"
    );
}

#[test]
fn html_skips_protected_elements() {
    assert_eq!(
        smartypants("<code>don't \"x\"</code>"),
        "<code>don't \"x\"</code>"
    );
    assert_eq!(smartypants("<pre>a -- b</pre>"), "<pre>a -- b</pre>");
    // text outside the protected element is still educated
    assert_eq!(
        smartypants("\"a\" <code>\"b\"</code> \"c\""),
        "\u{201c}a\u{201d} <code>\"b\"</code> \u{201c}c\u{201d}"
    );
}

#[test]
fn html_comments_are_left_alone() {
    assert_eq!(
        smartypants("<!-- don't \"touch\" -->ok \"x\""),
        "<!-- don't \"touch\" -->ok \u{201c}x\u{201d}"
    );
}

// ---------------------------------------------------------------------------
// Plain-text mode
// ---------------------------------------------------------------------------

#[test]
fn plain_text_mode_treats_angle_brackets_as_text() {
    assert_eq!(
        SmartyPants::new().html(false).apply("a < b -- c"),
        "a < b \u{2014} c"
    );
}

// ---------------------------------------------------------------------------
// Toggles & idempotence
// ---------------------------------------------------------------------------

#[test]
fn quotes_can_be_disabled() {
    assert_eq!(
        SmartyPants::new().quotes(false).apply("\"x\" -- y"),
        "\"x\" \u{2014} y"
    );
}

#[test]
fn already_curly_text_is_unchanged() {
    let once = smartypants("\"It's\" -- a test...");
    assert_eq!(smartypants(&once), once);
}

#[test]
fn empty_input() {
    assert_eq!(smartypants(""), "");
}

// ---------------------------------------------------------------------------
// Regression: adversarial-review findings
// ---------------------------------------------------------------------------

#[test]
fn empty_quote_pair_closes() {
    // SP-Q1: "" must become “” (open + close), not ““.
    assert_eq!(smartypants("\"\""), "\u{201c}\u{201d}");
    assert_eq!(
        smartypants("an empty \"\" here"),
        "an empty \u{201c}\u{201d} here"
    );
}

#[test]
fn quote_context_threads_through_protected_elements() {
    // SP-Q2: the apostrophe after </code> sees 'x', so it is a closing quote.
    assert_eq!(
        smartypants("see <code>x</code>'s"),
        "see <code>x</code>\u{2019}s"
    );
}

#[test]
fn gt_inside_attribute_value_does_not_end_the_tag() {
    // H1: the '>' inside the quoted title must not terminate the tag.
    assert_eq!(
        smartypants("<a title=\"a>b\">don't</a>"),
        "<a title=\"a>b\">don\u{2019}t</a>"
    );
    assert_eq!(smartypants("<img alt='x>y'>"), "<img alt='x>y'>");
}

#[test]
fn bare_less_than_is_literal_text() {
    // H2 / SP-1: a '<' that doesn't start a tag is ordinary text; education continues.
    assert_eq!(
        smartypants("5 < 10 so it's \"true\"..."),
        "5 < 10 so it\u{2019}s \u{201c}true\u{201d}\u{2026}"
    );
    assert_eq!(smartypants("x < y and 'a'"), "x < y and \u{2018}a\u{2019}");
}

#[test]
fn degenerate_comments_terminate() {
    // H5: abrupt-closing comment forms must not swallow the rest of the input.
    assert_eq!(smartypants("<!-->\"x\""), "<!-->\u{201c}x\u{201d}");
    assert_eq!(smartypants("<!--->\"x\""), "<!--->\u{201c}x\u{201d}");
    assert_eq!(
        smartypants("<!-- c -->\"x\""),
        "<!-- c -->\u{201c}x\u{201d}"
    );
}
