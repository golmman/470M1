//! Atomic writes for every output path: a unique temporary file in the
//! target's directory, synced and renamed, removed on any failure.

use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

use crate::app::error::Error;

/// The directory a path's temporary file belongs to; an empty parent is ".".
fn directory(path: &Path) -> PathBuf {
    path.parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."))
        .to_owned()
}

/// Writes `bytes` to `path` atomically.
pub(crate) fn atomic(path: &Path, bytes: &[u8]) -> io::Result<()> {
    replace(path, |file| file.write_all(bytes))
}

/// Writes through a fresh temporary file, then renames it over `path`; a
/// failure removes the temporary and leaves the target untouched.
fn replace(path: &Path, write: impl FnOnce(&mut File) -> io::Result<()>) -> io::Result<()> {
    static COUNTER: AtomicUsize = AtomicUsize::new(0);
    let name = path
        .file_name()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "not a file name"))?;
    let directory = directory(path);
    let mut file;
    let mut temp;
    loop {
        let unique = COUNTER.fetch_add(1, Ordering::Relaxed);
        temp = directory.join(format!(
            ".{}.{}.{}.tmp",
            name.to_string_lossy(),
            std::process::id(),
            unique
        ));
        match OpenOptions::new().write(true).create_new(true).open(&temp) {
            Ok(opened) => {
                file = opened;
                break;
            }
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(error),
        }
    }
    let result = write(&mut file).and_then(|()| file.sync_all());
    let result = result.and_then(|()| fs::rename(&temp, path));
    if result.is_err() {
        let _ = fs::remove_file(&temp);
    }
    result
}

/// Refuses an output path that is a directory or whose parent is missing,
/// before a search or a sub-process is started.
pub(crate) fn check_target(path: &Path) -> Result<(), Error> {
    if path.is_dir() {
        return Err(Error::Output {
            path: path.to_owned(),
            reason: "it is a directory".into(),
        });
    }
    if !directory(path).is_dir() {
        return Err(Error::Output {
            path: path.to_owned(),
            reason: "its parent directory does not exist".into(),
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    /// A target in its own directory, so parallel tests and other processes
    /// never share the temporaries `no_temps` looks for.
    fn target(name: &str) -> PathBuf {
        let directory = env::temp_dir().join(format!("470mi-write-{}-{name}", std::process::id()));
        fs::create_dir_all(&directory).unwrap();
        directory.join("out")
    }

    fn no_temps(path: &Path) -> bool {
        fs::read_dir(path.parent().unwrap()).unwrap().all(|entry| {
            !entry
                .unwrap()
                .file_name()
                .to_string_lossy()
                .ends_with(".tmp")
        })
    }

    #[test]
    fn replaces_the_target() {
        let path = target("replace");
        atomic(&path, b"old").unwrap();
        atomic(&path, b"new").unwrap();
        assert_eq!(fs::read(&path).unwrap(), b"new");
        assert!(no_temps(&path));
    }

    #[test]
    fn a_failed_write_keeps_the_old_bytes() {
        let path = target("failed");
        atomic(&path, b"old").unwrap();
        let result = replace(&path, |_| Err(io::Error::other("boom")));
        assert!(result.is_err());
        assert_eq!(fs::read(&path).unwrap(), b"old");
        assert!(no_temps(&path));
    }

    #[test]
    fn checks_the_target_before_work() {
        assert!(check_target(Path::new("/nonexistent-dir-9/x")).is_err());
        assert!(check_target(Path::new("/tmp")).is_err());
        assert!(check_target(&target("fine")).is_ok());
    }
}
