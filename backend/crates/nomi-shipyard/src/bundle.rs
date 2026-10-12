//! The source zip Shipyard builds from: the project's files, minus what it must never get
//! (`node_modules`, `.git`, build output, `.env` files, local databases).

use std::io::Write;

/// Whether a project file is left out of the upload.
pub fn excluded(path: &str) -> bool {
    const DIRS: [&str; 9] = ["node_modules", ".git", ".svelte-kit", "build", "dist", ".data", ".astro", ".vercel", ".output"];
    let mut parts = path.split('/');
    let first = parts.next().unwrap_or_default();
    if DIRS.contains(&first) || path.split('/').any(|p| p == "node_modules" || p == ".git") {
        return true;
    }
    let name = path.rsplit('/').next().unwrap_or(path);
    (name == ".env" || name.starts_with(".env.")) && name != ".env.example"
        || name.ends_with(".db")
        || name.ends_with(".sqlite")
        || name.ends_with(".db-journal")
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;

    #[test]
    fn leaves_out_what_shipyard_must_not_get() {
        for path in ["node_modules/x/index.js", ".git/HEAD", "build/index.js", ".env", ".env.production", "local.db", "packages/a/node_modules/b.js"] {
            assert!(excluded(path), "{path} should be excluded");
        }
        for path in ["src/routes/+page.svelte", "package.json", ".env.example", "drizzle/0000_init.sql", "src/build/helper.ts"] {
            assert!(!excluded(path), "{path} should be kept");
        }
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
