//! Tool crates sit below the agents: none may depend on `nomi-agent-core` or any agent crate.

use std::path::Path;

const AGENT_CRATE_PREFIX: &str = "nomi-agent-";

/// The dependency names listed in a manifest's dependency sections.
fn dependency_names(manifest: &str) -> Vec<String> {
    let mut in_deps = false;
    let mut names = Vec::new();
    for line in manifest.lines().map(str::trim) {
        if line.starts_with('[') {
            in_deps = line.ends_with("dependencies]");
            continue;
        }
        if in_deps {
            if let Some((name, _)) = line.split_once('=') {
                names.push(name.trim().to_string());
            }
        }
    }
    names
}

#[test]
fn no_tool_crate_depends_on_agent_crates() {
    let crates = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    let mut checked = 0;
    for entry in std::fs::read_dir(crates).unwrap() {
        let dir = entry.unwrap().path();
        let name = dir.file_name().unwrap().to_string_lossy().to_string();
        if !name.starts_with("nomi-tools-") {
            continue;
        }
        let manifest = std::fs::read_to_string(dir.join("Cargo.toml")).unwrap();
        for dependency in dependency_names(&manifest) {
            assert!(!dependency.starts_with(AGENT_CRATE_PREFIX), "{name} depends on {dependency}");
        }
        checked += 1;
    }
    assert!(checked > 0, "no nomi-tools-* crates found");
}
