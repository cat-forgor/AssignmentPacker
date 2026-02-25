use crate::error::{Error, Result, io_err};
use std::fs::{self, File};
use std::io;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::Duration;
use walkdir::WalkDir;
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipWriter};

const DELETE_RETRIES: usize = 25;
const DELETE_DELAY: Duration = Duration::from_millis(80);

const BINARY_EXTENSIONS: &[&str] = &["exe", "com", "dll", "so", "dylib", "out", "bin", "msi"];

pub fn check_extension(path: &Path, allowed: &[&str], label: &str) -> Result<()> {
    if !path.exists() {
        return Err(Error::Validation(format!(
            "{label} not found: '{}'",
            path.display()
        )));
    }
    if !path.is_file() {
        return Err(Error::Validation(format!(
            "{label} is not a file: '{}'",
            path.display()
        )));
    }

    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_ascii_lowercase());

    if !ext.as_deref().is_some_and(|e| allowed.contains(&e)) {
        let expected = allowed
            .iter()
            .map(|e| format!(".{e}"))
            .collect::<Vec<_>>()
            .join("/");
        return Err(Error::Validation(format!(
            "{label} must be {expected}, got '{}'",
            path.display()
        )));
    }
    Ok(())
}

pub fn prepare_output(dir: &Path, zip: &Path, force: bool) -> Result<()> {
    if dir.exists() {
        if force {
            remove_dir_retry(dir)?;
        } else {
            return Err(Error::Validation(format!(
                "already exists: '{}' (use --force)",
                dir.display()
            )));
        }
    }
    if zip.exists() {
        if force {
            remove_file_retry(zip)?;
        } else {
            return Err(Error::Validation(format!(
                "already exists: '{}' (use --force)",
                zip.display()
            )));
        }
    }
    Ok(())
}

pub fn copy_non_binary_files(src: &Path, dst: &Path) -> Result<()> {
    let entries = fs::read_dir(src).map_err(|e| io_err(format!("reading {}", src.display()), e))?;

    for entry in entries {
        let entry = entry.map_err(|e| io_err("reading directory entry", e))?;
        let path = entry.path();
        if !path.is_file() || is_binary_ext(&path) {
            continue;
        }
        let name = file_name(&path)?;
        let dest = dst.join(name);
        if paths_equal(&path, &dest) {
            continue;
        }
        fs::copy(&path, &dest).map_err(|e| io_err(format!("copying '{}'", path.display()), e))?;
    }
    Ok(())
}

pub fn create_zip(source_dir: &Path, zip_path: &Path) -> Result<()> {
    let zip_file = File::create(zip_path)
        .map_err(|e| io_err(format!("creating {}", zip_path.display()), e))?;

    let mut zip = ZipWriter::new(zip_file);
    let options = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);

    for entry in WalkDir::new(source_dir) {
        let entry = entry.map_err(|e| io_err("walking directory", io::Error::other(e)))?;
        let path = entry.path();

        let rel = path
            .strip_prefix(source_dir)
            .map_err(|e| Error::Validation(format!("strip_prefix: {e}")))?;

        if rel.as_os_str().is_empty() {
            continue;
        }

        let zip_name = rel.to_string_lossy().replace('\\', "/");
        if path.is_dir() {
            zip.add_directory(&zip_name, options)
                .map_err(|e| Error::Validation(format!("zip add dir '{zip_name}': {e}")))?;
        } else {
            zip.start_file(&zip_name, options)
                .map_err(|e| Error::Validation(format!("zip add file '{zip_name}': {e}")))?;
            let mut f =
                File::open(path).map_err(|e| io_err(format!("opening '{}'", path.display()), e))?;
            io::copy(&mut f, &mut zip)
                .map_err(|e| io_err(format!("writing '{zip_name}' to zip"), e))?;
        }
    }

    zip.finish()
        .map_err(|e| Error::Validation(format!("finalizing zip: {e}")))?;
    Ok(())
}

pub fn resolve_c_file(provided: Option<&Path>) -> Result<PathBuf> {
    if let Some(p) = provided {
        return Ok(p.to_path_buf());
    }

    let cwd = std::env::current_dir().map_err(|e| io_err("current directory", e))?;
    let mut found: Vec<PathBuf> = Vec::new();

    for entry in fs::read_dir(&cwd).map_err(|e| io_err("reading cwd", e))? {
        let entry = entry.map_err(|e| io_err("directory entry", e))?;
        let path = entry.path();
        let is_c = path
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| e.eq_ignore_ascii_case("c"))
            .unwrap_or(false);
        if path.is_file() && is_c {
            found.push(path);
        }
    }

    found.sort_by(|a, b| a.to_string_lossy().cmp(&b.to_string_lossy()));

    match found.len() {
        0 => Err(Error::Validation(
            "no .c files found in current directory".into(),
        )),
        1 => Ok(found.remove(0)),
        _ => {
            let names: Vec<_> = found
                .iter()
                .filter_map(|p| p.file_name()?.to_str())
                .collect();
            Err(Error::Validation(format!(
                "multiple .c files found: {}, specify --c-file",
                names.join(", ")
            )))
        }
    }
}

pub fn resolve_doc_file(provided: Option<&Path>, expected_name: &str) -> Result<PathBuf> {
    if let Some(p) = provided {
        return Ok(p.to_path_buf());
    }
    let cwd = std::env::current_dir().map_err(|e| io_err("current directory", e))?;
    let expected = cwd.join(expected_name);
    if expected.exists() {
        Ok(expected)
    } else {
        Err(Error::Validation(format!(
            "expected doc not found: '{}'",
            expected.display()
        )))
    }
}

pub fn read_text_lossy(path: &Path) -> Result<String> {
    let bytes = fs::read(path).map_err(|e| io_err(format!("reading {}", path.display()), e))?;
    Ok(String::from_utf8_lossy(&bytes).into_owned())
}

pub fn file_name(path: &Path) -> Result<&str> {
    path.file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| Error::Validation(format!("invalid filename: '{}'", path.display())))
}

/// Returns true only if both paths exist and resolve to the same file.
/// Returns false if either path does not exist (safe for pre-copy checks).
pub fn paths_equal(a: &Path, b: &Path) -> bool {
    match (fs::canonicalize(a), fs::canonicalize(b)) {
        (Ok(ca), Ok(cb)) => ca == cb,
        _ => false,
    }
}

fn is_binary_ext(path: &Path) -> bool {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_ascii_lowercase());

    if let Some(ext) = ext.as_deref() {
        return BINARY_EXTENSIONS.contains(&ext);
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if let Ok(meta) = fs::metadata(path) {
            return meta.permissions().mode() & 0o111 != 0;
        }
    }

    false
}

pub fn remove_file_retry(path: &Path) -> Result<()> {
    retry_remove(path, |p| fs::remove_file(p))
}

pub fn remove_dir_retry(path: &Path) -> Result<()> {
    retry_remove(path, |p| fs::remove_dir_all(p))
}

fn retry_remove<F>(path: &Path, f: F) -> Result<()>
where
    F: Fn(&Path) -> io::Result<()>,
{
    if !path.exists() {
        return Ok(());
    }

    let mut last_err: Option<io::Error> = None;
    for i in 0..DELETE_RETRIES {
        match f(path) {
            Ok(()) => return Ok(()),
            Err(_) if !path.exists() => return Ok(()),
            Err(e)
                if matches!(
                    e.kind(),
                    io::ErrorKind::PermissionDenied | io::ErrorKind::Other
                ) =>
            {
                last_err = Some(e);
                if i + 1 < DELETE_RETRIES {
                    thread::sleep(DELETE_DELAY);
                }
            }
            Err(e) => return Err(io_err(format!("removing '{}'", path.display()), e)),
        }
    }

    Err(io_err(
        format!("timed out removing '{}'", path.display()),
        last_err.unwrap_or_else(|| io::Error::other("retry exhausted")),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn binary_extensions_detected() {
        assert!(is_binary_ext(Path::new("prog.exe")));
        assert!(is_binary_ext(Path::new("lib.dll")));
        assert!(is_binary_ext(Path::new("lib.DLL")));
        assert!(!is_binary_ext(Path::new("main.c")));
        assert!(!is_binary_ext(Path::new("notes.txt")));
        assert!(!is_binary_ext(Path::new("Makefile")));
    }
}
