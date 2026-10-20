use std::path::{Path, PathBuf};

use super::app_reference::AppReference;

/// The layers cut by domain: each has one folder, or one flat file, per domain.
pub const DOMAIN_LAYERS: [&str; 4] = ["model", "repository", "service", "controller"];

/// `src/app/mod.rs` only declares the layers, so it belongs to none.
const ROOT_MODULE: &str = "mod.rs";

const ROLE_SUFFIXES: [&str; 3] = ["_controller", "_service", "_repository"];

/// A file under `src/app`, with the layer and domain its place gives it and the app paths it names.
#[derive(Debug)]
pub struct SourceFile {
    pub path: String,
    pub layer: String,
    pub domain: Option<String>,
    pub references: Vec<AppReference>,
}

impl SourceFile {
    pub fn all() -> Vec<SourceFile> {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/app");
        let mut paths = Vec::new();
        collect_rust_files(&root, &mut paths);
        paths.sort();
        paths
            .iter()
            .map(|path| SourceFile::read(&root, path))
            .filter(|file| file.path != ROOT_MODULE)
            .collect()
    }

    fn read(root: &Path, path: &Path) -> SourceFile {
        let relative = path.strip_prefix(root).unwrap();
        let parts: Vec<String> = relative
            .components()
            .map(|c| c.as_os_str().to_string_lossy().into_owned())
            .collect();
        let source = std::fs::read_to_string(path).unwrap();
        SourceFile {
            path: parts.join("/"),
            layer: parts[0].trim_end_matches(".rs").to_string(),
            domain: domain_of_place(&parts),
            references: AppReference::all_in(&source),
        }
    }
}

/// The domain a module name stands for: `client_service` and `client_service_test` are `client`.
pub fn domain_of(module: &str) -> String {
    let name = module.trim_end_matches(".rs").trim_end_matches("_test");
    ROLE_SUFFIXES
        .iter()
        .find_map(|suffix| name.strip_suffix(suffix))
        .unwrap_or(name)
        .to_string()
}

/// A folder inside a domain layer is the domain; so is a flat file, by its name.
fn domain_of_place(parts: &[String]) -> Option<String> {
    if parts.len() < 2 || !DOMAIN_LAYERS.contains(&parts[0].as_str()) {
        return None;
    }
    let is_folder = parts.len() > 2;
    match parts[1].as_str() {
        "mod.rs" if !is_folder => None,
        name => Some(domain_of(name)),
    }
}

fn collect_rust_files(folder: &Path, found: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(folder).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            collect_rust_files(&path, found);
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            found.push(path);
        }
    }
}

#[test]
fn a_flat_file_and_its_test_belong_to_the_domain_it_is_named_for() {
    assert_eq!(domain_of("client_service"), "client");
    assert_eq!(domain_of("config_service_test.rs"), "config");
    assert_eq!(domain_of("backup_repository.rs"), "backup");
    assert_eq!(domain_of("invoice"), "invoice");
}

#[test]
fn a_folder_is_its_own_domain_and_a_layer_root_has_none() {
    let place = |path: &str| path.split('/').map(str::to_string).collect::<Vec<_>>();

    assert_eq!(
        domain_of_place(&place("service/ts/ts_dispatch_service.rs")),
        Some("ts".to_string())
    );
    assert_eq!(domain_of_place(&place("service/mod.rs")), None);
    assert_eq!(domain_of_place(&place("scheduler/ts_worker.rs")), None);
}
