//! Dateien schreiben, ohne Vorhandenes zu zerstören.

use std::ffi::{OsStr, OsString};
use std::fs::{File, OpenOptions};
use std::io::{BufWriter, ErrorKind};
use std::path::{Path, PathBuf};

use crate::Result;

/// Schreibt erst eine vollständige Temp-Datei neben `path` und ersetzt `path`
/// danach in einem Schritt. Schlägt etwas fehl, bleibt `path` unverändert und
/// die Temp-Datei wird entfernt.
pub fn write_atomically(
    path: &Path,
    write: impl FnOnce(&mut BufWriter<File>) -> Result<()>,
) -> Result<()> {
    let (temp_path, temp) = create_temp_beside(path)?;
    let written = write_and_replace(temp, &temp_path, path, write);
    if written.is_err() {
        let _ = std::fs::remove_file(&temp_path);
    }
    written
}

fn write_and_replace(
    temp: File,
    temp_path: &Path,
    path: &Path,
    write: impl FnOnce(&mut BufWriter<File>) -> Result<()>,
) -> Result<()> {
    let mut writer = BufWriter::new(temp);
    write(&mut writer)?;
    let temp = writer
        .into_inner()
        .map_err(std::io::IntoInnerError::into_error)?;
    temp.sync_all()?;
    drop(temp);
    // `std::fs::rename` ersetzt ein vorhandenes Ziel auch unter Windows.
    std::fs::rename(temp_path, path)?;
    Ok(())
}

fn create_temp_beside(path: &Path) -> std::io::Result<(PathBuf, File)> {
    let name = path.file_name().unwrap_or_default();
    let mut attempt: u32 = 1;
    loop {
        let mut temp_name = OsString::from(".");
        temp_name.push(name);
        temp_name.push(format!(".{attempt}.tmp"));
        let temp_path = path.with_file_name(temp_name);
        match OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temp_path)
        {
            Ok(file) => return Ok((temp_path, file)),
            Err(error) if error.kind() == ErrorKind::AlreadyExists => attempt += 1,
            Err(error) => return Err(error),
        }
    }
}

/// Erster freier Dateipfad aus `path`, `Name (2).ext`, `Name (3).ext`, …
/// Damit ersetzt eine neue Anleitung oder ein neuer Export nie eine vorhandene Datei.
pub fn unused_file_path(path: &Path) -> PathBuf {
    let stem = path.file_stem().unwrap_or_default();
    let extension = path.extension();
    first_unused(path, |number| {
        let mut name = numbered(stem, number);
        if let Some(extension) = extension {
            name.push(".");
            name.push(extension);
        }
        name
    })
}

/// Erster freier Ordnerpfad aus `path`, `Name (2)`, `Name (3)`, … Die Nummer
/// hängt am ganzen Namen, auch wenn er einen Punkt enthält.
pub fn unused_dir_path(path: &Path) -> PathBuf {
    let name = path.file_name().unwrap_or_default();
    first_unused(path, |number| numbered(name, number))
}

fn first_unused(path: &Path, name: impl Fn(u32) -> OsString) -> PathBuf {
    let mut candidate = path.to_owned();
    let mut number = 1;
    while candidate.symlink_metadata().is_ok() {
        number += 1;
        candidate = path.with_file_name(name(number));
    }
    candidate
}

fn numbered(name: &OsStr, number: u32) -> OsString {
    let mut numbered = name.to_os_string();
    numbered.push(format!(" ({number})"));
    numbered
}

#[cfg(test)]
mod tests {
    use super::{unused_dir_path, unused_file_path, write_atomically};
    use crate::StoreError;
    use std::io::Write;
    use std::path::{Path, PathBuf};

    fn scratch(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("steps-store-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("scratch dir");
        dir
    }

    fn names(dir: &Path) -> Vec<String> {
        let mut names: Vec<String> = std::fs::read_dir(dir)
            .expect("read dir")
            .map(|entry| {
                entry
                    .expect("entry")
                    .file_name()
                    .to_string_lossy()
                    .into_owned()
            })
            .collect();
        names.sort();
        names
    }

    #[test]
    fn failed_write_keeps_the_existing_file_and_leaves_no_temp() {
        let dir = scratch("failed-write");
        let path = dir.join("Guide.steps");
        std::fs::write(&path, b"valid guide").expect("seed");

        let error = write_atomically(&path, |writer| {
            writer.write_all(b"half a gui")?;
            Err(StoreError::Io(std::io::Error::other("disk full")))
        })
        .expect_err("write fails");

        assert_eq!(error.to_string(), "i/o error: disk full");
        assert_eq!(std::fs::read(&path).expect("read"), b"valid guide");
        assert_eq!(names(&dir), ["Guide.steps"]);
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn successful_write_replaces_the_file_and_leaves_no_temp() {
        let dir = scratch("replace");
        let path = dir.join("Guide.steps");
        std::fs::write(&path, b"old").expect("seed");

        write_atomically(&path, |writer| Ok(writer.write_all(b"new")?)).expect("write");

        assert_eq!(std::fs::read(&path).expect("read"), b"new");
        assert_eq!(names(&dir), ["Guide.steps"]);
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn taken_file_names_get_the_next_number_before_the_extension() {
        let dir = scratch("file-names");
        let desired = dir.join("Guide v1.2.steps");
        assert_eq!(unused_file_path(&desired), desired);

        std::fs::write(&desired, b"first").expect("seed");
        assert_eq!(unused_file_path(&desired), dir.join("Guide v1.2 (2).steps"));

        std::fs::write(dir.join("Guide v1.2 (2).steps"), b"second").expect("seed");
        std::fs::create_dir(dir.join("Guide v1.2 (3).steps")).expect("seed");
        assert_eq!(unused_file_path(&desired), dir.join("Guide v1.2 (4).steps"));
        assert_eq!(std::fs::read(&desired).expect("read"), b"first");
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn taken_folder_names_get_the_number_after_the_whole_name() {
        let dir = scratch("dir-names");
        let desired = dir.join("Guide v1.2-markdown");
        assert_eq!(unused_dir_path(&desired), desired);

        std::fs::create_dir(&desired).expect("seed");
        assert_eq!(
            unused_dir_path(&desired),
            dir.join("Guide v1.2-markdown (2)")
        );
        std::fs::remove_dir_all(&dir).ok();
    }
}
