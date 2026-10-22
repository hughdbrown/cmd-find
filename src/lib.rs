//! Find executables on `PATH` whose file name matches a shell glob.
//!
//! The search runs one task per distinct `PATH` directory on a rayon
//! thread pool; a directory listed more than once is searched only once.
//! Results are returned in `PATH` order, and sorted by name within a
//! directory, so output is deterministic.

use std::collections::HashSet;
use std::env;
use std::ffi::OsStr;
use std::fmt;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use glob::Pattern;
use rayon::prelude::*;

/// A validated shell glob that matches against a file name (not a path).
///
/// ```
/// use cmd_find::CommandGlob;
/// let g: CommandGlob = "cargo-*".parse().unwrap();
/// assert!(g.matches("cargo-clippy"));
/// assert!(!g.matches("cargo"));
/// ```
#[derive(Debug, Clone)]
pub struct CommandGlob(Pattern);

/// Why a string could not become a [`CommandGlob`].
#[derive(Debug)]
pub enum GlobError {
    /// The pattern was empty.
    Empty,
    /// The pattern contained a path separator; only file names are matched.
    ContainsSeparator,
    /// The glob syntax was invalid.
    Syntax(glob::PatternError),
}

impl fmt::Display for GlobError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => write!(f, "pattern must not be empty"),
            Self::ContainsSeparator => {
                write!(f, "pattern matches file names only and may not contain '/'")
            }
            Self::Syntax(e) => write!(f, "invalid glob: {e}"),
        }
    }
}

impl std::error::Error for GlobError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Syntax(e) => Some(e),
            Self::Empty | Self::ContainsSeparator => None,
        }
    }
}

impl FromStr for CommandGlob {
    type Err = GlobError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            return Err(GlobError::Empty);
        }
        if s.contains('/') {
            return Err(GlobError::ContainsSeparator);
        }
        Pattern::new(s).map(Self).map_err(GlobError::Syntax)
    }
}

impl CommandGlob {
    /// Whether `name` matches this glob.
    #[must_use]
    pub fn matches(&self, name: &str) -> bool {
        self.0.matches(name)
    }
}

/// Split a `PATH`-style value into directories, keeping only the first
/// occurrence of each and dropping empty entries.
///
/// ```
/// use cmd_find::unique_dirs;
/// use std::path::PathBuf;
/// let dirs = unique_dirs("/usr/bin:/bin:/usr/bin".as_ref());
/// assert_eq!(dirs, vec![PathBuf::from("/usr/bin"), PathBuf::from("/bin")]);
/// ```
#[must_use]
pub fn unique_dirs(path: &OsStr) -> Vec<PathBuf> {
    let mut seen: HashSet<PathBuf> = HashSet::new();
    env::split_paths(path)
        .filter(|dir: &PathBuf| !dir.as_os_str().is_empty())
        .filter(|dir: &PathBuf| seen.insert(dir.clone()))
        .collect()
}

/// Executables directly inside `dir` whose file name matches `glob`,
/// sorted by name. An unreadable or missing directory yields nothing.
#[must_use]
pub fn matches_in_dir(dir: &Path, glob: &CommandGlob) -> Vec<PathBuf> {
    let Ok(entries) = fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut found: Vec<PathBuf> = entries
        .filter_map(Result::ok)
        .filter(|entry: &fs::DirEntry| glob.matches(&entry.file_name().to_string_lossy()))
        .map(|entry: fs::DirEntry| entry.path())
        .filter(|path: &PathBuf| is_executable(path))
        .collect();
    found.sort();
    found
}

/// A regular file (following symlinks) with any execute bit set.
fn is_executable(path: &Path) -> bool {
    fs::metadata(path)
        .is_ok_and(|meta: fs::Metadata| meta.is_file() && meta.permissions().mode() & 0o111 != 0)
}

/// Search every distinct directory in `path` in parallel and return all
/// matching executables in `PATH` order.
#[must_use]
pub fn find(path: &OsStr, glob: &CommandGlob) -> Vec<PathBuf> {
    unique_dirs(path)
        .par_iter()
        .flat_map_iter(|dir: &PathBuf| matches_in_dir(dir, glob))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use tempfile::TempDir;

    fn touch(dir: &Path, name: &str, mode: u32) -> PathBuf {
        let p = dir.join(name);
        let f = File::create(&p).unwrap();
        f.set_permissions(fs::Permissions::from_mode(mode)).unwrap();
        p
    }

    #[test]
    fn glob_rejects_empty_and_separators() {
        assert!(matches!("".parse::<CommandGlob>(), Err(GlobError::Empty)));
        assert!(matches!(
            "bin/ls".parse::<CommandGlob>(),
            Err(GlobError::ContainsSeparator)
        ));
        assert!(matches!(
            "[".parse::<CommandGlob>(),
            Err(GlobError::Syntax(_))
        ));
    }

    #[test]
    fn unique_dirs_drops_duplicates_and_empties() {
        let dirs = unique_dirs("/a:/b::/a:/c:/b".as_ref());
        let expected: Vec<PathBuf> = ["/a", "/b", "/c"].iter().map(PathBuf::from).collect();
        assert_eq!(dirs, expected);
    }

    #[test]
    fn matches_in_dir_filters_by_glob_and_exec_bit() {
        let tmp = TempDir::new().unwrap();
        let exec = touch(tmp.path(), "cargo-run", 0o755);
        touch(tmp.path(), "cargo-doc.txt", 0o644);
        touch(tmp.path(), "rustc", 0o755);
        fs::create_dir(tmp.path().join("cargo-dir")).unwrap();

        let glob: CommandGlob = "cargo*".parse().unwrap();
        assert_eq!(matches_in_dir(tmp.path(), &glob), vec![exec]);
    }

    #[test]
    fn matches_in_dir_missing_directory_is_empty() {
        let glob: CommandGlob = "*".parse().unwrap();
        let found = matches_in_dir(Path::new("/definitely/not/here"), &glob);
        assert_eq!(found, Vec::<PathBuf>::new());
    }

    #[test]
    fn find_keeps_path_order_and_searches_duplicates_once() {
        let a = TempDir::new().unwrap();
        let b = TempDir::new().unwrap();
        let a_tool = touch(a.path(), "tool-x", 0o755);
        let b_tool = touch(b.path(), "tool-y", 0o755);

        let path = env::join_paths([b.path(), a.path(), b.path()]).unwrap();
        let glob: CommandGlob = "tool-?".parse().unwrap();
        assert_eq!(find(&path, &glob), vec![b_tool, a_tool]);
    }
}
