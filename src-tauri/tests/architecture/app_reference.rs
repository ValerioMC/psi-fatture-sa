const PREFIX: &str = "crate::app::";

/// One `crate::app::<layer>::<item>` path named in a source file. `item` is the module
/// right after the layer, absent when the path stops at the layer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppReference {
    pub layer: String,
    pub item: Option<String>,
}

impl AppReference {
    /// Every app path in `source`, one per module of a `{a, b}` group.
    pub fn all_in(source: &str) -> Vec<AppReference> {
        let mut found = Vec::new();
        let mut rest = source;
        while let Some(start) = rest.find(PREFIX) {
            rest = &rest[start + PREFIX.len()..];
            let layer = identifier(rest);
            for item in items_after(&rest[layer.len()..]) {
                found.push(AppReference {
                    layer: layer.to_string(),
                    item,
                });
            }
        }
        found
    }
}

fn identifier(text: &str) -> &str {
    let end = text
        .find(|c: char| !(c.is_alphanumeric() || c == '_'))
        .unwrap_or(text.len());
    &text[..end]
}

/// The modules named right after the layer: one, each member of a `{...}` group, or none.
fn items_after(text: &str) -> Vec<Option<String>> {
    let Some(path) = text.strip_prefix("::") else {
        return vec![None];
    };
    match path.strip_prefix('{') {
        Some(group) => group_members(group)
            .into_iter()
            .map(|member| Some(identifier(member).to_string()).filter(|name| name != "self"))
            .collect(),
        None => vec![Some(identifier(path).to_string())],
    }
}

/// The top-level members of a `{...}` group, whose opening brace is already consumed.
fn group_members(group: &str) -> Vec<&str> {
    let mut members = Vec::new();
    let mut depth = 0;
    let mut start = 0;
    for (index, c) in group.char_indices() {
        match c {
            '{' => depth += 1,
            '}' if depth == 0 => {
                members.push(group[start..index].trim());
                break;
            }
            '}' => depth -= 1,
            ',' if depth == 0 => {
                members.push(group[start..index].trim());
                start = index + 1;
            }
            _ => {}
        }
    }
    members.into_iter().filter(|m| !m.is_empty()).collect()
}

#[test]
fn reads_single_and_grouped_paths() {
    let source = "use crate::app::common::AppError;\n\
                  use crate::app::service::{client_service, ts::{a, b}, self};\n\
                  let x = crate::app::model::invoice::Invoice::default();";

    let found: Vec<(String, Option<String>)> = AppReference::all_in(source)
        .into_iter()
        .map(|r| (r.layer, r.item))
        .collect();

    let named = |layer: &str, item: Option<&str>| (layer.to_string(), item.map(str::to_string));
    assert_eq!(
        found,
        vec![
            named("common", Some("AppError")),
            named("service", Some("client_service")),
            named("service", Some("ts")),
            named("service", None),
            named("model", Some("invoice")),
        ]
    );
}
