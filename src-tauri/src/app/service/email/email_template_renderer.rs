//! `{placeholder}` substitution for the email template. A brace that does not
//! open a known name is left as written.

use crate::app::model::email::EmailPlaceholder;

/// Every `{name}` in the text, in order, known or not.
pub fn placeholders_in(text: &str) -> Vec<String> {
    let mut found = Vec::new();
    let mut rest = text;
    while let Some(open) = rest.find('{') {
        let after = &rest[open + 1..];
        match after.find(['}', '{']) {
            Some(close) if after.as_bytes()[close] == b'}' => {
                found.push(after[..close].trim().to_string());
                rest = &after[close + 1..];
            }
            Some(close) => rest = &after[close..],
            None => break,
        }
    }
    found
}

/// The names in the text that are not placeholders, each once.
pub fn unknown_placeholders(text: &str) -> Vec<String> {
    let mut unknown: Vec<String> = Vec::new();
    for name in placeholders_in(text) {
        if EmailPlaceholder::parse(&name).is_none() && !unknown.contains(&name) {
            unknown.push(name);
        }
    }
    unknown
}

pub fn render(text: &str, value: impl Fn(EmailPlaceholder) -> String) -> String {
    let mut rendered = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(open) = rest.find('{') {
        rendered.push_str(&rest[..open]);
        let after = &rest[open + 1..];
        let known = after
            .find('}')
            .and_then(|close| EmailPlaceholder::parse(after[..close].trim()).map(|p| (p, close)));
        match known {
            Some((placeholder, close)) => {
                rendered.push_str(&value(placeholder));
                rest = &after[close + 1..];
            }
            None => {
                rendered.push('{');
                rest = after;
            }
        }
    }
    rendered.push_str(rest);
    rendered
}

#[cfg(test)]
#[path = "email_template_renderer_test.rs"]
mod tests;
