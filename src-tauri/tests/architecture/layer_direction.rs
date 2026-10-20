use super::source_file::SourceFile;

/// The layers each layer may name besides itself. Calls point down: a repository never
/// reaches a service, a controller reaches the data only through a service. `app_state` and
/// `scheduler` are the wiring, which holds the gateways and runs the workers.
const MAY_NAME: [(&str, &[&str]); 9] = [
    ("common", &[]),
    ("db", &[]),
    ("entity", &[]),
    ("model", &["common"]),
    ("repository", &["common", "entity", "model"]),
    ("service", &["common", "entity", "model", "repository"]),
    ("controller", &["app_state", "common", "model", "service"]),
    ("app_state", &["common", "repository", "service"]),
    (
        "scheduler",
        &["common", "controller", "model", "repository", "service"],
    ),
];

fn may_name(layer: &str) -> Option<&'static [&'static str]> {
    MAY_NAME
        .iter()
        .find(|(name, _)| *name == layer)
        .map(|(_, layers)| *layers)
}

#[test]
fn every_layer_names_only_the_layers_below_it() {
    let mut violations = Vec::new();
    for file in SourceFile::all() {
        let Some(allowed) = may_name(&file.layer) else {
            violations.push(format!(
                "{}: layer `{}` is not in MAY_NAME",
                file.path, file.layer
            ));
            continue;
        };
        for reference in &file.references {
            if reference.layer != file.layer && !allowed.contains(&reference.layer.as_str()) {
                violations.push(format!(
                    "{}: {} names `{}`",
                    file.path, file.layer, reference.layer
                ));
            }
        }
    }
    violations.dedup();
    assert!(violations.is_empty(), "\n{}", violations.join("\n"));
}
