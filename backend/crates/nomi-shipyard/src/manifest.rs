//! What Shipyard is asked to run for a project: one web service on port 3000 built from a
//! Dockerfile, plus a Postgres database or a data volume when the app keeps data. Projects
//! without their own Dockerfile get one written for their stack. A project on Relay also gets a
//! second app, its backend: Relay, PostgREST and the functions runtime next to their own Postgres.

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

/// The Shipyard app that holds a project's Relay backend: the project's own app slug and this.
pub const BACKEND_SUFFIX: &str = "-backend";
/// Where a backend's images are (Admin → Configuration → Relay): Relay's and its functions
/// runtime's repositories in Shipyard's registry, PostgREST's image, and the tag the first two run
/// at. The version is raised there when Nomi should move projects to a newer Relay.
#[derive(Debug, Clone, PartialEq)]
pub struct Images {
    /// `<registry>/<prefix>/relay`, without a tag.
    pub relay: String,
    /// `<registry>/<prefix>/relay-functions`, without a tag.
    pub functions: String,
    /// A full reference, tag included.
    pub postgrest: String,
    pub version: String,
}

/// PostgREST's image when none is set.
pub const DEFAULT_POSTGREST_IMAGE: &str = "postgrest/postgrest:v12.2.3";
/// The Relay version to run when none is set (the npm packages' version when this was written).
pub const DEFAULT_RELAY_VERSION: &str = "0.1.5";
/// What a backend's services listen on.
pub const RELAY_PORT: u16 = 8080;
const POSTGREST_PORT: u16 = 3000;
const FUNCTIONS_PORT: u16 = 8000;
/// The secrets a backend environment binds: Nomi sets the control key (it calls Relay with it),
/// Shipyard generates the rest.
pub const SECRET_CONTROL_KEY: &str = "RELAY_CONTROL_KEY";
pub const SECRET_ENCRYPTION_KEY: &str = "RELAY_ENCRYPTION_KEY";
pub const SECRET_PGRST_PASSWORD: &str = "RELAY_PGRST_PASSWORD";
pub const SECRET_FUNCTIONS_SECRET: &str = "RELAY_FUNCTIONS_SECRET";
/// What a development environment's secret names start with. Secrets belong to the app, not to an
/// environment (an environment doesn't exist until its first deploy), so dev and production each
/// get their own names.
const DEV_SECRET_PREFIX: &str = "DEV_";

/// The name a backend secret has in `env`: production keeps `base`, development gets a prefix.
pub fn secret_name(base: &str, env: &str) -> String {
    if env == "production" {
        base.to_string()
    } else {
        format!("{DEV_SECRET_PREFIX}{base}")
    }
}
/// The login PostgREST uses (Relay creates it).
const AUTHENTICATOR: &str = "authenticator";

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

/// A backend environment's services: Relay (the environment's address), PostgREST and, when the
/// project has functions, the functions runtime, each built from the one-line Dockerfile in
/// `backend_source` (Shipyard's integration API can't run a prebuilt image yet), and the
/// environment's own Postgres. `env` is `dev` or `production`; Relay serves exactly this project
/// in exactly this environment.
pub fn relay_backend(project_id: &str, env: &str, has_functions: bool, version: &str) -> Value {
    let secret = |name: &str| json!({ "secret": secret_name(name, env) });
    let mut relay_env = json!({
        "RELAY_DATABASE_URL": { "database": "url" },
        "RELAY_ENCRYPTION_KEY": secret(SECRET_ENCRYPTION_KEY),
        "RELAY_CONTROL_KEY": secret(SECRET_CONTROL_KEY),
        "RELAY_PROJECT": project_id,
        "RELAY_ENV": env,
        "RELAY_POSTGREST_URL": format!("http://postgrest:{POSTGREST_PORT}"),
        "RELAY_PGRST_PASSWORD": secret(SECRET_PGRST_PASSWORD),
    });
    let mut services = vec![
        json!({ "name": "relay", "kind": "web", "path": "relay", "port": RELAY_PORT, "replicas": 1, "env": {} }),
        json!({
            "name": "postgrest", "kind": "worker", "path": "postgrest", "replicas": 1,
            "env": {
                "PGRST_DB_URI": "postgresql://",
                "PGHOST": { "database": "host" },
                "PGPORT": { "database": "port" },
                "PGDATABASE": { "database": "name" },
                "PGUSER": AUTHENTICATOR,
                "PGPASSWORD": secret(SECRET_PGRST_PASSWORD),
            },
        }),
    ];
    if has_functions {
        relay_env["RELAY_FUNCTIONS_URL"] = json!(format!("http://functions:{FUNCTIONS_PORT}"));
        relay_env["RELAY_FUNCTIONS_SECRET"] = secret(SECRET_FUNCTIONS_SECRET);
        services.push(json!({
            "name": "functions", "kind": "worker", "path": "functions", "replicas": 1,
            "env": { "RELAY_FUNCTIONS_SECRET": secret(SECRET_FUNCTIONS_SECRET) },
        }));
    }
    services[0]["env"] = relay_env;
    json!({
        "services": services,
        "database": { "engine": "postgres" },
        "labels": { "nomi.app/project": project_id, "nomi.app/role": "relay-backend", "nomi.app/relay-version": version },
    })
}

/// The source of a backend: one Dockerfile per service, each only naming its image, so building
/// them just pulls the image.
pub fn backend_source(images: &Images, has_functions: bool) -> Result<Vec<u8>, String> {
    let dockerfile = |image: &str| format!("FROM {image}\n").into_bytes();
    let mut files = vec![
        ("relay/Dockerfile".to_string(), dockerfile(&format!("{}:{}", images.relay, images.version))),
        ("postgrest/Dockerfile".to_string(), dockerfile(&images.postgrest)),
    ];
    if has_functions {
        files.push(("functions/Dockerfile".to_string(), dockerfile(&format!("{}:{}", images.functions, images.version))));
    }
    crate::bundle::zip(&files)
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
    fn a_backend_is_relay_postgrest_and_a_database() {
        let manifest = relay_backend("p1", "dev", false, "0.1.5");
        let services = manifest["services"].as_array().unwrap();
        let names: Vec<&str> = services.iter().map(|s| s["name"].as_str().unwrap()).collect();
        assert_eq!(names, vec!["relay", "postgrest"]);
        assert_eq!(manifest["database"]["engine"], "postgres");
        let relay = &services[0];
        assert_eq!((relay["kind"].as_str(), relay["port"].as_u64()), (Some("web"), Some(8080)));
        assert_eq!(relay["env"]["RELAY_PROJECT"], "p1");
        assert_eq!(relay["env"]["RELAY_ENV"], "dev");
        assert_eq!(relay["env"]["RELAY_DATABASE_URL"], json!({ "database": "url" }));
        assert_eq!(relay["env"]["RELAY_POSTGREST_URL"], "http://postgrest:3000");
        assert_eq!(relay["env"]["RELAY_CONTROL_KEY"], json!({ "secret": "RELAY_CONTROL_KEY" }));
        assert!(relay["env"].get("RELAY_FUNCTIONS_URL").is_none());
        let postgrest = &services[1];
        assert_eq!(postgrest["kind"], "worker");
        assert_eq!(postgrest["env"]["PGUSER"], "authenticator");
        assert_eq!(postgrest["env"]["PGPASSWORD"], relay["env"]["RELAY_PGRST_PASSWORD"], "both ends share one password");
        assert_eq!(postgrest["env"]["PGHOST"], json!({ "database": "host" }));
        assert!(!manifest.to_string().contains("DATABASE_URL\":\"postgres"), "no database address is written into the manifest");
    }

    #[test]
    fn a_backend_with_functions_runs_the_runtime_and_tells_relay_where() {
        let manifest = relay_backend("p1", "production", true, "0.1.5");
        let services = manifest["services"].as_array().unwrap();
        assert_eq!(services.len(), 3);
        assert_eq!(services[2]["name"], "functions");
        assert_eq!(services[0]["env"]["RELAY_ENV"], "production");
        assert_eq!(services[0]["env"]["RELAY_FUNCTIONS_URL"], "http://functions:8000");
        assert_eq!(services[0]["env"]["RELAY_FUNCTIONS_SECRET"], services[2]["env"]["RELAY_FUNCTIONS_SECRET"]);
    }

    #[test]
    fn a_backends_source_is_one_line_dockerfiles_naming_the_images() {
        let images = Images {
            relay: "registry.test/acme/relay".into(),
            functions: "registry.test/acme/relay-functions".into(),
            postgrest: "postgrest/postgrest:v12.2.3".into(),
            version: "0.1.5".into(),
        };
        let zip = backend_source(&images, true).unwrap();
        let files = crate::bundle::files_under(&zip, "").unwrap();
        let text = |path: &str| String::from_utf8(files.iter().find(|(p, _)| p == path).unwrap().1.clone()).unwrap();
        assert_eq!(text("relay/Dockerfile"), "FROM registry.test/acme/relay:0.1.5\n");
        assert_eq!(text("functions/Dockerfile"), "FROM registry.test/acme/relay-functions:0.1.5\n");
        assert!(text("postgrest/Dockerfile").starts_with("FROM postgrest/postgrest:v"));
        let without = crate::bundle::files_under(&backend_source(&images, false).unwrap(), "").unwrap();
        assert!(without.iter().all(|(p, _)| p != "functions/Dockerfile"));
        // Every service's `path` has its Dockerfile.
        for service in relay_backend("p1", "dev", true, "0.1.5")["services"].as_array().unwrap() {
            let dir = service["path"].as_str().unwrap();
            assert!(files.iter().any(|(p, _)| *p == format!("{dir}/Dockerfile")), "{dir}");
        }
    }

    #[test]
    fn labels_say_which_nomi_project_and_files_it_runs() {
        let mut manifest = build(Database::None, &[]);
        label(&mut manifest, "p1", "sveltekit", 7);
        assert_eq!(manifest["labels"], json!({ "nomi.app/project": "p1", "nomi.app/stack": "sveltekit", "nomi.app/files-version": "7" }));
    }

    #[test]
    fn dev_and_production_backends_bind_their_own_secret_names() {
        let secrets = |env: &str| relay_backend("p", env, false, "1")["services"][0]["env"]["RELAY_ENCRYPTION_KEY"].clone();
        assert_eq!(secrets("production"), json!({ "secret": "RELAY_ENCRYPTION_KEY" }));
        assert_eq!(secrets("dev"), json!({ "secret": "DEV_RELAY_ENCRYPTION_KEY" }));
    }
}
