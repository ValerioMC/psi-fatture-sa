use std::collections::{BTreeMap, BTreeSet};

use super::source_file::{domain_of, SourceFile, DOMAIN_LAYERS};

/// Which domains each domain may use, in any layer. A new edge is a design decision: change
/// this table and the one in the README together, or change the code. `support` holds the
/// pure helpers every domain shares.
const ALLOWED: [(&str, &[&str]); 16] = [
    ("appointment", &["support"]),
    ("backup", &[]),
    ("client", &["support"]),
    ("config", &["support"]),
    ("dashboard", &["invoice"]),
    (
        "email",
        &["client", "config", "invoice", "pdf", "secret", "support"],
    ),
    (
        "invoice",
        &["appointment", "config", "service", "support", "tax"],
    ),
    ("invoice_change", &["invoice", "ts"]),
    ("pdf", &["client", "config", "invoice", "support"]),
    ("secret", &[]),
    ("service", &[]),
    ("support", &[]),
    ("tax", &["config"]),
    ("terms", &[]),
    ("ts", &["client", "config", "invoice", "secret", "support"]),
    ("update", &[]),
];

fn allowed() -> BTreeMap<&'static str, BTreeSet<&'static str>> {
    ALLOWED
        .iter()
        .map(|(domain, uses)| (*domain, uses.iter().copied().collect()))
        .collect()
}

/// Every domain-to-domain use the code makes, with the file that makes it.
fn edges_in_code() -> BTreeMap<(String, String), String> {
    let mut edges = BTreeMap::new();
    for file in SourceFile::all() {
        let Some(domain) = &file.domain else {
            continue;
        };
        for reference in &file.references {
            let Some(item) = &reference.item else {
                continue;
            };
            if !DOMAIN_LAYERS.contains(&reference.layer.as_str()) {
                continue;
            }
            let used = domain_of(item);
            if used != *domain {
                edges
                    .entry((domain.clone(), used))
                    .or_insert_with(|| file.path.clone());
            }
        }
    }
    edges
}

#[test]
fn every_domain_uses_only_what_the_table_allows() {
    let allowed = allowed();
    let violations: Vec<String> = edges_in_code()
        .into_iter()
        .filter(|((from, to), _)| {
            !allowed
                .get(from.as_str())
                .is_some_and(|uses| uses.contains(to.as_str()))
        })
        .map(|((from, to), path)| format!("{from} -> {to} ({path})"))
        .collect();
    assert!(violations.is_empty(), "\n{}", violations.join("\n"));
}

#[test]
fn every_allowed_edge_is_one_the_code_uses() {
    let used: BTreeSet<(String, String)> = edges_in_code().into_keys().collect();
    let unused: Vec<String> = ALLOWED
        .iter()
        .flat_map(|(from, uses)| uses.iter().map(move |to| (*from, *to)))
        .filter(|(from, to)| !used.contains(&(from.to_string(), to.to_string())))
        .map(|(from, to)| format!("{from} -> {to}"))
        .collect();
    assert!(unused.is_empty(), "\n{}", unused.join("\n"));
}

#[test]
fn the_table_lists_exactly_the_domains_in_the_code() {
    let in_code: BTreeSet<String> = SourceFile::all()
        .into_iter()
        .filter_map(|file| file.domain)
        .collect();
    let in_table: BTreeSet<String> = ALLOWED.iter().map(|(d, _)| d.to_string()).collect();
    assert_eq!(in_code, in_table);
}

#[test]
fn no_domain_reaches_itself_through_others() {
    let allowed = allowed();
    for start in allowed.keys() {
        if let Some(cycle) = path_back_to(&allowed, start, start, &mut Vec::new()) {
            panic!("domain cycle: {start} -> {}", cycle.join(" -> "));
        }
    }
}

/// A path of allowed edges from `from` back to `target`, if the table has one.
fn path_back_to(
    allowed: &BTreeMap<&'static str, BTreeSet<&'static str>>,
    from: &str,
    target: &str,
    visited: &mut Vec<&'static str>,
) -> Option<Vec<&'static str>> {
    for next in allowed.get(from).into_iter().flatten() {
        if *next == target {
            return Some(vec![next]);
        }
        if visited.contains(next) {
            continue;
        }
        visited.push(next);
        if let Some(mut rest) = path_back_to(allowed, next, target, visited) {
            rest.insert(0, next);
            return Some(rest);
        }
    }
    None
}
