//! What Koda found in an imported project, worked out from its files alone (no model): the
//! stack, how it builds and tests, where its data lives, which secrets its `.env.example` names
//! that the project doesn't have yet, and the steps Nomi will take to set it up.

use std::collections::HashSet;

use serde::Serialize;
use serde_json::Value;

/// The files the review reads the contents of.
pub const KEY_FILES: [&str; 6] = ["package.json", "drizzle.config.ts", "drizzle.config.js", ".env.example", ".env.sample", "requirements.txt"];

/// What Koda found.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ImportReview {
    pub stack: String,
    /// The command that builds the app; null when there is nothing to build.
    pub build: Option<String>,
    /// The command that runs its tests; null when it has none.
    pub tests: Option<String>,
    /// `postgres`, `sqlite` or `none`.
    pub database: String,
    /// Names its `.env.example` asks for that the project has no secret for.
    pub secrets_missing: Vec<String>,
    /// What Nomi will set up, in order.
    pub plan: Vec<String>,
}

/// The review of a tree: every `paths` entry, the contents of its `contents` (the [`KEY_FILES`]
/// that exist), and the names of the project's secrets.
pub fn review(paths: &[String], contents: &[(String, Vec<u8>)], secret_names: &[String]) -> ImportReview {
    let has = |name: &str| paths.iter().any(|p| p == name);
    let text = |name: &str| contents.iter().find(|(p, _)| p == name).map(|(_, bytes)| String::from_utf8_lossy(bytes).to_string());
    let package = text("package.json");
    let package_json: Value = package.as_deref().and_then(|t| serde_json::from_str(t).ok()).unwrap_or(Value::Null);
    let manager = package.is_some().then(|| package_manager(&has));
    let requirements = text("requirements.txt").unwrap_or_default();

    let stack = if let Some(package) = &package {
        crate::routes::project_import::detect_stack(&[("package.json".to_string(), package.clone())])
    } else if has("Cargo.toml") {
        "rust"
    } else if has("go.mod") {
        "go"
    } else if has("requirements.txt") || has("pyproject.toml") {
        "python"
    } else {
        "static"
    };

    let has_script = |name: &str| package_json["scripts"][name].is_string();
    let has_dependency = |name: &str| ["dependencies", "devDependencies"].iter().any(|group| package_json[*group].get(name).is_some());
    let build = match manager {
        Some(pm) if has_script("build") => Some(format!("{pm} run build")),
        _ if has("Cargo.toml") => Some("cargo build --release".to_string()),
        _ if has("go.mod") => Some("go build ./...".to_string()),
        _ => None,
    };
    let tests = match manager {
        Some("bun") if has_script("test") => Some("bun run test".to_string()),
        Some(pm) if has_script("test") => Some(format!("{pm} test")),
        Some(_) if has_dependency("vitest") => Some("npx vitest run".to_string()),
        Some(_) if has_dependency("jest") => Some("npx jest".to_string()),
        _ if has("Cargo.toml") => Some("cargo test".to_string()),
        _ if has("go.mod") => Some("go test ./...".to_string()),
        _ if requirements.lines().any(|l| l.trim().to_lowercase().starts_with("pytest")) => Some("pytest".to_string()),
        _ => None,
    };

    let database = crate::routes::deploy::database_name(nomi_shipyard::manifest::detect_database(contents)).to_string();
    let secrets_missing = ENV_EXAMPLES.iter().find_map(|name| text(name)).map(|t| missing_secrets(&t, secret_names)).unwrap_or_default();

    let mut plan = Vec::new();
    match (manager, stack) {
        (Some(pm), _) => plan.push(format!("Install the dependencies with {pm}")),
        (None, "python") => plan.push("Install the dependencies with pip".to_string()),
        _ => {}
    }
    if database != "none" {
        plan.push(format!("Set up the {database} database"));
    }
    if !secrets_missing.is_empty() {
        plan.push(format!("Ask you for {}: {}", if secrets_missing.len() == 1 { "a secret" } else { "secrets" }, secrets_missing.join(", ")));
    }
    if let Some(command) = &tests {
        plan.push(format!("Run the tests ({command})"));
    }
    if let Some(command) = &build {
        plan.push(format!("Build the app ({command})"));
    }
    plan.push(DEPLOY_STEP.to_string());

    ImportReview { stack: stack.to_string(), build, tests, database, secrets_missing, plan }
}

/// The last step of every plan.
const DEPLOY_STEP: &str = "Deploy the app";
/// The files a project lists the environment it wants in, best first.
const ENV_EXAMPLES: [&str; 2] = [".env.example", ".env.sample"];

/// The package manager a Node project uses, from its lock file.
fn package_manager(has: &impl Fn(&str) -> bool) -> &'static str {
    if has("pnpm-lock.yaml") {
        "pnpm"
    } else if has("yarn.lock") {
        "yarn"
    } else if has("bun.lockb") || has("bun.lock") {
        "bun"
    } else {
        "npm"
    }
}

/// The names in an `.env.example` that can be secrets here (capitals; not the ones Nomi sets for
/// every app) and aren't yet, first mention first.
fn missing_secrets(env_example: &str, secret_names: &[String]) -> Vec<String> {
    let have: HashSet<&str> = secret_names.iter().map(String::as_str).collect();
    let mut seen = HashSet::new();
    let mut missing = Vec::new();
    for line in env_example.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let Some((name, _)) = line.strip_prefix("export ").unwrap_or(line).split_once('=') else { continue };
        let name = name.trim();
        if crate::routes::project_secrets::check_name(name).is_ok() && !have.contains(name) && seen.insert(name.to_string()) {
            missing.push(name.to_string());
        }
    }
    missing
}

#[cfg(test)]
mod tests {
    use super::*;

    fn paths(list: &[&str]) -> Vec<String> {
        list.iter().map(|p| p.to_string()).collect()
    }

    fn file(path: &str, content: &str) -> (String, Vec<u8>) {
        (path.to_string(), content.as_bytes().to_vec())
    }

    const KIT_PACKAGE: &str = r#"{"scripts":{"build":"vite build","test":"vitest run"},"devDependencies":{"@sveltejs/kit":"^2","vitest":"^2"},"dependencies":{"postgres":"^3"}}"#;
    const ENV_EXAMPLE: &str = "# keys\nDATABASE_URL=postgres://localhost/app\nPORT=3000\nSTRIPE_KEY=\nexport RESEND_API_KEY=your-key\nSTRIPE_KEY=again\nlowercase=no\nPUBLIC_NAME=Warung\n";

    #[test]
    fn a_sveltekit_app_with_postgres_tests_and_secrets() {
        let tree = paths(&["package.json", "pnpm-lock.yaml", "src/routes/+page.svelte", "src/lib/menu.test.ts", ".env.example"]);
        let contents = [file("package.json", KIT_PACKAGE), file(".env.example", ENV_EXAMPLE)];
        let found = review(&tree, &contents, &["PUBLIC_NAME".to_string()]);
        assert_eq!(found.stack, "sveltekit");
        assert_eq!(found.build.as_deref(), Some("pnpm run build"));
        assert_eq!(found.tests.as_deref(), Some("pnpm test"));
        assert_eq!(found.database, "postgres");
        // Nomi sets DATABASE_URL and PORT itself, PUBLIC_NAME is already a secret, the repeat and the odd name are dropped.
        assert_eq!(found.secrets_missing, ["STRIPE_KEY", "RESEND_API_KEY"]);
        assert_eq!(found.plan.first().map(String::as_str), Some("Install the dependencies with pnpm"));
        assert!(found.plan.iter().any(|s| s.contains("STRIPE_KEY")));
        assert!(found.plan.iter().any(|s| s.to_lowercase().contains("database")));
        assert_eq!(found.plan.last().map(String::as_str), Some("Deploy the app"));
    }

    #[test]
    fn a_plain_site_has_nothing_to_build_test_or_store() {
        let found = review(&paths(&["index.html", "css/site.css"]), &[], &[]);
        assert_eq!((found.stack.as_str(), found.build, found.tests, found.database.as_str()), ("static", None, None, "none"));
        assert!(found.secrets_missing.is_empty());
        assert_eq!(found.plan, ["Deploy the app"]);
    }

    #[test]
    fn a_test_runner_without_a_script_still_counts_and_npm_is_the_default() {
        let package = r#"{"scripts":{"build":"tsc"},"devDependencies":{"vitest":"^2"},"dependencies":{"react":"^18"}}"#;
        let found = review(&paths(&["package.json"]), &[file("package.json", package)], &[]);
        assert_eq!(found.stack, "react");
        assert_eq!(found.build.as_deref(), Some("npm run build"));
        assert_eq!(found.tests.as_deref(), Some("npx vitest run"));
    }

    #[test]
    fn other_languages_are_named_by_their_manifest() {
        assert_eq!(review(&paths(&["Cargo.toml", "src/main.rs"]), &[], &[]).tests.as_deref(), Some("cargo test"));
        let python = review(&paths(&["requirements.txt", "app.py"]), &[file("requirements.txt", "flask\npytest\n")], &[]);
        assert_eq!((python.stack.as_str(), python.tests.as_deref()), ("python", Some("pytest")));
        assert_eq!(review(&paths(&["go.mod", "main.go"]), &[], &[]).stack, "go");
    }

    #[test]
    fn sqlite_is_read_from_the_drizzle_config() {
        let config = r#"export default { dialect: "sqlite" }"#;
        let found = review(&paths(&["drizzle.config.ts"]), &[file("drizzle.config.ts", config)], &[]);
        assert_eq!(found.database, "sqlite");
    }
}
