//! The source zip Shipyard builds from: the project's files, minus what it must never get
//! (`node_modules`, `.git`, build output, `.env` files, local databases).

use std::io::Write;

/// Folders anywhere in the project tree that are dependencies, version control, caches, or build outputs.
const EXCLUDED_ANYWHERE: &[&str] = &[
    "node_modules",
    ".git",
    ".svn",
    ".hg",
    ".next",
    ".nuxt",
    ".svelte-kit",
    ".astro",
    ".vercel",
    ".output",
    ".turbo",
    ".cache",
    ".parcel-cache",
    ".angular",
    "coverage",
    ".nyc_output",
    "__pycache__",
    ".pytest_cache",
    ".mypy_cache",
    ".venv",
    "venv",
    "__MACOSX",
    ".idea",
    ".vscode",
];

/// Folders at the project root or top-level packages that represent local builds, test/data dirs, or temp files.
const EXCLUDED_ROOT_DIRS: &[&str] = &[
    "build",
    "dist",
    "out",
    "target",
    ".data",
    "tmp",
    ".tmp",
];

/// Whether a project file is left out of the upload.
pub fn excluded(path: &str) -> bool {
    let parts: Vec<&str> = path.split('/').collect();
    if parts.is_empty() {
        return true;
    }

    // Exclude ignored directories anywhere in the path hierarchy
    if parts.iter().any(|p| EXCLUDED_ANYWHERE.contains(p)) {
        return true;
    }

    let first = parts[0];
    if EXCLUDED_ROOT_DIRS.contains(&first) {
        return true;
    }

    // Exclude build/dist output folders inside packages (e.g. packages/web/dist or frontend/build),
    // but keep source directories like src/build/helper.ts or app/build/...
    if parts.len() > 1 {
        for (i, &part) in parts.iter().enumerate() {
            if (part == "dist" || part == "build" || part == "out") && i > 0 {
                let prev = parts[i - 1];
                if prev != "src" && prev != "lib" && prev != "app" && prev != "routes" && prev != "pages" {
                    return true;
                }
            }
        }
    }

    let name = parts.last().copied().unwrap_or(path);

    // OS junk
    if name == ".DS_Store" || name == "Thumbs.db" || name == "desktop.ini" {
        return true;
    }

    // Secrets & Environment files
    if (name == ".env" || name.starts_with(".env.")) && name != ".env.example" && name != ".env.sample" {
        return true;
    }
    if name.ends_with(".pem") || name.ends_with(".key") || name.ends_with(".cert") || name.ends_with(".crt") {
        return true;
    }

    // Databases, SQLite WAL/journal, and dumps
    if name.ends_with(".db")
        || name.ends_with(".sqlite")
        || name.ends_with(".sqlite3")
        || name.ends_with(".db3")
        || name.ends_with(".db-journal")
        || name.ends_with(".sqlite-journal")
        || name.ends_with(".db-wal")
        || name.ends_with(".sqlite-wal")
        || name.ends_with(".db-shm")
        || name.ends_with(".sqlite-shm")
        || name.ends_with(".sql.gz")
        || name.ends_with(".dump")
    {
        return true;
    }

    // Logs
    if name.ends_with(".log")
        || name.starts_with("npm-debug.log")
        || name.starts_with("yarn-debug.log")
        || name.starts_with("yarn-error.log")
        || name.starts_with("pnpm-debug.log")
    {
        return true;
    }

    // Python bytecode
    if name.ends_with(".pyc") || name.ends_with(".pyo") {
        return true;
    }

    // Editor swap and backup files
    if name.ends_with('~') || name.ends_with(".swp") || name.ends_with(".swo") {
        return true;
    }

    false
}

/// Zips `files` (path, contents), skipping excluded paths. Paths are kept as given.
pub fn zip(files: &[(String, Vec<u8>)]) -> Result<Vec<u8>, String> {
    let mut writer = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
    let options = zip::write::SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);
    for (path, bytes) in files {
        if excluded(path) {
            continue;
        }
        writer.start_file(path.as_str(), options).map_err(|e| e.to_string())?;
        writer.write_all(bytes).map_err(|e| e.to_string())?;
    }
    writer.finish().map(|cursor| cursor.into_inner()).map_err(|e| e.to_string())
}

/// The files of a source zip under `prefix` (`nomi/functions/`), as `(path, contents)`.
pub fn files_under(zip_bytes: &[u8], prefix: &str) -> Result<Vec<(String, Vec<u8>)>, String> {
    use std::io::Read;
    let mut archive = zip::ZipArchive::new(std::io::Cursor::new(zip_bytes)).map_err(|e| e.to_string())?;
    let mut out = Vec::new();
    for index in 0..archive.len() {
        let mut entry = archive.by_index(index).map_err(|e| e.to_string())?;
        let name = entry.name().to_string();
        if entry.is_dir() || !name.starts_with(prefix) {
            continue;
        }
        let mut bytes = Vec::new();
        entry.read_to_end(&mut bytes).map_err(|e| e.to_string())?;
        out.push((name, bytes));
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;

    #[test]
    fn leaves_out_what_shipyard_must_not_get() {
        for path in [
            "node_modules/x/index.js",
            ".git/HEAD",
            "build/index.js",
            "dist/index.js",
            "packages/a/build/index.js",
            "packages/a/dist/index.js",
            ".env",
            ".env.production",
            ".env.local",
            "local.db",
            "local.sqlite",
            "local.sqlite3",
            "local.db3",
            "local.db-wal",
            "packages/a/node_modules/b.js",
            ".next/cache/turbopack.bin",
            "apps/web/.next/standalone/server.js",
            "frontend/.svelte-kit/output/client.js",
            "coverage/lcov.info",
            ".turbo/cache/123",
            ".cache/turbopack",
            "__pycache__/file.cpython-311.pyc",
            ".venv/bin/python",
            "venv/bin/activate",
            ".DS_Store",
            "src/.DS_Store",
            "debug.log",
            "npm-debug.log.123",
            "secret.pem",
            "private.key",
            ".data/db.sqlite",
            "target/debug/nomi",
            "out/static/main.js",
        ] {
            assert!(excluded(path), "{path} should be excluded");
        }
        for path in [
            "src/routes/+page.svelte",
            "package.json",
            "pnpm-lock.yaml",
            ".env.example",
            ".env.sample",
            "drizzle/0000_init.sql",
            "src/build/helper.ts",
            "app/build/tools.ts",
            "public/images/logo.png",
            "README.md",
        ] {
            assert!(!excluded(path), "{path} should be kept");
        }
    }

    #[test]
    fn reads_back_the_files_under_a_folder() {
        let bytes = zip(&[("nomi/functions/a.ts".into(), b"a".to_vec()), ("nomi/functions/_shared/b.ts".into(), b"b".to_vec()), ("src/app.ts".into(), b"app".to_vec())]).unwrap();
        let mut found = files_under(&bytes, "nomi/functions/").unwrap();
        found.sort();
        assert_eq!(found, vec![("nomi/functions/_shared/b.ts".to_string(), b"b".to_vec()), ("nomi/functions/a.ts".to_string(), b"a".to_vec())]);
        assert!(files_under(b"not a zip", "x/").is_err());
    }

    #[test]
    fn zips_the_kept_files() {
        let bytes = zip(&[("package.json".into(), b"{}".to_vec()), (".env".into(), b"SECRET=1".to_vec())]).unwrap();
        let mut archive = zip::ZipArchive::new(std::io::Cursor::new(bytes)).unwrap();
        assert_eq!(archive.len(), 1);
        let mut content = String::new();
        archive.by_name("package.json").unwrap().read_to_string(&mut content).unwrap();
        assert_eq!(content, "{}");
    }
}
