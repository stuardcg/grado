//! Markdown to HTML for player-facing content. Content is authored by us and
//! lives in the repo, so raw HTML in it is trusted.

use pulldown_cmark::{Options, Parser, html};

pub fn to_html(markdown: &str) -> String {
    let parser = Parser::new_ext(markdown, Options::ENABLE_TABLES | Options::ENABLE_STRIKETHROUGH);
    let mut out = String::new();
    html::push_html(&mut out, parser);
    out
}

/// Renders a one-line snippet without the wrapping paragraph.
pub fn inline_to_html(markdown: &str) -> String {
    let html = to_html(markdown);
    html.trim().trim_start_matches("<p>").trim_end_matches("</p>").to_owned()
}
