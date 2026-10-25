//! What Shipyard is asked to run for a project: one web service on port 3000 built from a
//! Dockerfile, plus a Postgres database or a data volume when the app keeps data. Projects
//! without their own Dockerfile get one written for their stack.

use serde_json::{json, Value};

/// The port every generated image listens on.
pub const PORT: u16 = 3000;

/// Where an app's data lives, read from its files.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Database {
    None,
    Postgres,
    Sqlite,
}

/// Reads the database the app uses from its Drizzle config or its packages.
pub fn detect_database(files: &[(String, Vec<u8>)]) -> Database {
    let text = |name: &str| {
        files
            .iter()
            .find(|(path, _)| path == name)
            .map(|(_, bytes)| String::from_utf8_lossy(bytes).to_string())
    };
    let config = text("drizzle.config.ts").or_else(|| text("drizzle.config.js")).unwrap_or_default();
    if config.contains("postgresql") {
        return Database::Postgres;
    }
    if config.contains("sqlite") || config.contains("turso") {
        return Database::Sqlite;
    }
    let package = text("package.json").unwrap_or_default();
    if package.contains("\"postgres\"") || package.contains("@electric-sql/pglite") {
        Database::Postgres
    } else if package.contains("@libsql/client") {
        Database::Sqlite
    } else {
        Database::None
    }
}

fn has(files: &[(String, Vec<u8>)], path: &str) -> bool {
    files.iter().any(|(p, _)| p == path)
}

fn has_dir(files: &[(String, Vec<u8>)], dir: &str) -> bool {
    let prefix = format!("{dir}/");
    files.iter().any(|(p, _)| p.starts_with(&prefix))
}

/// A Dockerfile for the stack (sveltekit, astro, svelte, react, vue or static).
pub fn dockerfile(stack: &str, files: &[(String, Vec<u8>)]) -> String {
    let install = if has(files, "package-lock.json") { "npm ci" } else { "npm install" };
    let drizzle = if has_dir(files, "drizzle") { "COPY --from=build /app/drizzle ./drizzle\n" } else { "" };
    match stack {
        "sveltekit" => format!(
            "FROM node:22-alpine AS build\nWORKDIR /app\nCOPY package*.json ./\nRUN {install}\nCOPY . .\nRUN npm run build && npm prune --omit=dev\n\n\
             FROM node:22-alpine\nWORKDIR /app\nCOPY --from=build /app/build ./build\nCOPY --from=build /app/node_modules ./node_modules\nCOPY --from=build /app/package.json ./\n{drizzle}\
             ENV NODE_ENV=production HOST=0.0.0.0 PORT={PORT} PROTOCOL_HEADER=x-forwarded-proto HOST_HEADER=x-forwarded-host\nEXPOSE {PORT}\nCMD [\"node\", \"build\"]\n"
        ),
        "astro" => format!(
            "FROM node:22-alpine AS build\nWORKDIR /app\nCOPY package*.json ./\nRUN {install}\nCOPY . .\nRUN npm run build && npm prune --omit=dev\n\n\
             FROM node:22-alpine\nWORKDIR /app\nCOPY --from=build /app/dist ./dist\nCOPY --from=build /app/node_modules ./node_modules\nCOPY --from=build /app/package.json ./\n{drizzle}\
             ENV NODE_ENV=production HOST=0.0.0.0 PORT={PORT}\nEXPOSE {PORT}\nCMD [\"node\", \"./dist/server/entry.mjs\"]\n"
        ),
        "static" => format!(
            "FROM node:22-alpine\nWORKDIR /site\nRUN npm install -g serve@14\nCOPY . .\nEXPOSE {PORT}\nCMD [\"serve\", \"-l\", \"tcp://0.0.0.0:{PORT}\", \"/site\"]\n"
        ),
        // Single-page apps built by Vite (svelte, react, vue): served with SPA fallback.
        _ => format!(
            "FROM node:22-alpine AS build\nWORKDIR /app\nCOPY package*.json ./\nRUN {install}\nCOPY . .\nRUN npm run build\n\n\
             FROM node:22-alpine\nWORKDIR /site\nRUN npm install -g serve@14\nCOPY --from=build /app/dist ./\nEXPOSE {PORT}\nCMD [\"serve\", \"-s\", \"-l\", \"tcp://0.0.0.0:{PORT}\", \"/site\"]\n"
        ),
    }
}

/// The manifest for a project: a `web` service, a database or volume as its data needs, and
/// the secrets its owners stored for it (bound by name, never by value).
pub fn build(database: Database, secret_names: &[String]) -> Value {
    let mut env = serde_json::Map::new();
    env.insert("NODE_ENV".into(), json!("production"));
    env.insert("PORT".into(), json!(PORT.to_string()));
    let mut volumes = Vec::new();
    match database {
        Database::Postgres => {
            env.insert("DATABASE_URL".into(), json!({ "database": "url" }));
        }
        Database::Sqlite => {
            env.insert("DATABASE_URL".into(), json!("file:/data/app.db"));
            volumes.push(json!({ "path": "/data" }));
        }
        Database::None => {}
    }
    let mut manifest = json!({
        "services": [{
            "name": "web",
            "kind": "web",
            "path": ".",
            "port": PORT,
            "replicas": 1,
            "env": env,
            "volumes": volumes,
        }],
    });
    if database == Database::Postgres {
        manifest["database"] = json!({ "engine": "postgres" });
    }
    bind_secrets(&mut manifest, secret_names);
    manifest
}

/// Labels Shipyard shows next to the environment, so support can tell which Nomi project and
/// which version of its files it runs (keys under `nomi.app/`; Shipyard owns `shipyard.io/`).
pub fn label(manifest: &mut Value, project_id: &str, stack: &str, files_version: i64) {
    manifest["labels"] = json!({
        "nomi.app/project": project_id,
        "nomi.app/stack": stack,
        "nomi.app/files-version": files_version.to_string(),
    });
}

/// Binds each secret, by name, into the `web` service's environment. A name the manifest
/// already sets itself (DATABASE_URL, PORT) keeps its own value.
pub fn bind_secrets(manifest: &mut Value, secret_names: &[String]) {
    let Some(env) = manifest
        .get_mut("services")
        .and_then(Value::as_array_mut)
        .and_then(|services| services.iter_mut().find(|s| s.get("name").and_then(Value::as_str) == Some("web")))
        .and_then(|web| {
            if web.get("env").map_or(true, |e| !e.is_object()) {
                web["env"] = json!({});
            }
            web.get_mut("env").and_then(Value::as_object_mut)
        })
    else {
        return;
    };
    for name in secret_names {
        env.entry(name.clone()).or_insert_with(|| json!({ "secret": name }));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn file(path: &str, text: &str) -> (String, Vec<u8>) {
        (path.to_string(), text.as_bytes().to_vec())
    }

    #[test]
    fn reads_the_database_from_drizzle_or_packages() {
        assert_eq!(detect_database(&[file("drizzle.config.ts", "dialect: 'postgresql'")]), Database::Postgres);
        assert_eq!(detect_database(&[file("drizzle.config.ts", "dialect: 'sqlite'")]), Database::Sqlite);
        assert_eq!(detect_database(&[file("package.json", r#"{"dependencies":{"postgres":"^3"}}"#)]), Database::Postgres);
        assert_eq!(detect_database(&[file("package.json", r#"{"dependencies":{"@libsql/client":"^0.18"}}"#)]), Database::Sqlite);
        assert_eq!(detect_database(&[file("package.json", "{}")]), Database::None);
    }

    #[test]
    fn postgres_apps_get_a_database_bound_by_url() {
        let manifest = build(Database::Postgres, &["STRIPE_KEY".to_string()]);
        assert_eq!(manifest["database"]["engine"], "postgres");
        let env = &manifest["services"][0]["env"];
        assert_eq!(env["DATABASE_URL"], json!({ "database": "url" }));
        assert_eq!(env["STRIPE_KEY"], json!({ "secret": "STRIPE_KEY" }));
        assert_eq!(manifest["services"][0]["port"], 3000);
    }

    #[test]
    fn a_rollback_gets_todays_secrets_but_keeps_its_own_env() {
        let mut manifest = build(Database::Postgres, &[]);
        bind_secrets(&mut manifest, &["OPENAI_API_KEY".to_string(), "DATABASE_URL".to_string()]);
        let env = &manifest["services"][0]["env"];
        assert_eq!(env["OPENAI_API_KEY"], json!({ "secret": "OPENAI_API_KEY" }));
        assert_eq!(env["DATABASE_URL"], json!({ "database": "url" }));
    }

    #[test]
    fn sqlite_apps_keep_their_file_on_a_volume() {
        let manifest = build(Database::Sqlite, &[]);
        assert!(manifest.get("database").is_none());
        assert_eq!(manifest["services"][0]["volumes"][0]["path"], "/data");
        assert_eq!(manifest["services"][0]["env"]["DATABASE_URL"], "file:/data/app.db");
    }

    #[test]
    fn dockerfiles_fit_the_stack() {
        let files = [file("package-lock.json", "{}"), file("drizzle/0000.sql", "")];
        let svelte_kit = dockerfile("sveltekit", &files);
        assert!(svelte_kit.contains("FROM node:22-alpine"));
        assert!(svelte_kit.contains("npm ci"));
        assert!(svelte_kit.contains("COPY --from=build /app/drizzle ./drizzle"));
        assert!(svelte_kit.contains("CMD [\"node\", \"build\"]"));
        let spa = dockerfile("react", &[]);
        assert!(spa.contains("FROM node:22-alpine"));
        assert!(spa.contains("npm install\n"));
        assert!(spa.contains("serve\", \"-s\""));
        assert!(!dockerfile("sveltekit", &[]).contains("drizzle"));
    }

    #[test]
    fn labels_say_which_nomi_project_and_files_it_runs() {
        let mut manifest = build(Database::None, &[]);
        label(&mut manifest, "p1", "sveltekit", 7);
        assert_eq!(manifest["labels"], json!({ "nomi.app/project": "p1", "nomi.app/stack": "sveltekit", "nomi.app/files-version": "7" }));
    }
}
