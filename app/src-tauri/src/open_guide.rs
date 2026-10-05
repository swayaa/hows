//! Open / re-open `.steps` packages.
//!
//! Boundary helpers for path checks and Recent MRU; loading lives in `commands`.

use std::path::{Path, PathBuf};

/// Why a path can't be opened; maps to an [`crate::error::ErrorCode`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpenErrorKind {
    /// Path missing on disk.
    NotFound,
    /// Exists but is not a `.steps` file.
    NotSteps,
}

/// Validates a candidate open path before ZIP load.
pub fn classify_open_path(path: &Path) -> Result<(), OpenErrorKind> {
    if !path.exists() {
        return Err(OpenErrorKind::NotFound);
    }
    if !path.is_file() {
        return Err(OpenErrorKind::NotSteps);
    }
    let is_steps = path
        .extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| ext.eq_ignore_ascii_case("steps"));
    if !is_steps {
        return Err(OpenErrorKind::NotSteps);
    }
    Ok(())
}

/// Prepends `path` to Recent, dedupes, keeps at most `limit` entries.
pub fn push_recent(recent: &mut Vec<String>, path: impl Into<String>, limit: usize) {
    let path = path.into();
    recent.retain(|entry| entry != &path);
    recent.insert(0, path);
    recent.truncate(limit);
}

/// Removes a path from Recent (missing / failed open).
pub fn prune_recent(recent: &mut Vec<String>, path: &str) {
    recent.retain(|entry| entry != path);
}

/// First `.steps` path among process args (skip argv[0]).
pub fn steps_path_from_args<I, S>(args: I) -> Option<PathBuf>
where
    I: IntoIterator<Item = S>,
    S: AsRef<std::ffi::OsStr>,
{
    let mut iter = args.into_iter();
    let _exe = iter.next();
    iter.map(|arg| PathBuf::from(arg.as_ref())).find(|path| {
        path.extension()
            .and_then(|ext| ext.to_str())
            .is_some_and(|ext| ext.eq_ignore_ascii_case("steps"))
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_dir() -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let dir = std::env::temp_dir().join(format!("steps-open-{nanos}"));
        fs::create_dir_all(&dir).expect("temp dir");
        dir
    }

    #[test]
    fn classify_missing_not_found() {
        let missing = std::env::temp_dir().join("definitely-missing-gro191.steps");
        let _ = fs::remove_file(&missing);
        assert_eq!(classify_open_path(&missing), Err(OpenErrorKind::NotFound));
    }

    #[test]
    fn classify_wrong_extension() {
        let dir = temp_dir();
        let path = dir.join("guide.txt");
        fs::write(&path, b"nope").unwrap();
        assert_eq!(classify_open_path(&path), Err(OpenErrorKind::NotSteps));
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn classify_steps_ok() {
        let dir = temp_dir();
        let path = dir.join("guide.steps");
        fs::write(&path, b"zip-placeholder").unwrap();
        assert_eq!(classify_open_path(&path), Ok(()));
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn recent_mru_dedupes_and_caps() {
        let mut recent = Vec::new();
        for i in 0..10 {
            push_recent(&mut recent, format!("/tmp/a{i}.steps"), 8);
        }
        assert_eq!(recent.len(), 8);
        assert_eq!(recent[0], "/tmp/a9.steps");
        push_recent(&mut recent, "/tmp/a7.steps", 8);
        assert_eq!(recent[0], "/tmp/a7.steps");
        assert_eq!(recent.iter().filter(|p| *p == "/tmp/a7.steps").count(), 1);
    }

    #[test]
    fn argv_picks_first_steps() {
        let path =
            steps_path_from_args(["app.exe", "--flag", r"C:\Guides\demo.steps", "other.txt"]);
        assert_eq!(path, Some(PathBuf::from(r"C:\Guides\demo.steps")));
        assert!(steps_path_from_args(["app.exe", "readme.md"]).is_none());
    }
}
