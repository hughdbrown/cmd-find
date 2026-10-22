// Integration tests may panic freely; clippy.toml only covers #[cfg(test)] modules.
#![allow(clippy::unwrap_used)]

use std::fs::{self, File};
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process::Command;

use tempfile::TempDir;

fn touch(dir: &Path, name: &str, mode: u32) {
    let f = File::create(dir.join(name)).unwrap();
    f.set_permissions(fs::Permissions::from_mode(mode)).unwrap();
}

fn run(path: &Path, pattern: &str) -> (bool, String) {
    let out = Command::new(env!("CARGO_BIN_EXE_cmd-find"))
        .arg(pattern)
        .env("PATH", path)
        .output()
        .unwrap();
    (out.status.success(), String::from_utf8(out.stdout).unwrap())
}

#[test]
fn prints_matches_one_per_line() {
    let tmp = TempDir::new().unwrap();
    touch(tmp.path(), "git-foo", 0o755);
    touch(tmp.path(), "git-bar", 0o755);
    touch(tmp.path(), "git-readme", 0o644);

    let (ok, stdout) = run(tmp.path(), "git-*");
    assert!(ok);
    let expected = format!(
        "{}\n{}\n",
        tmp.path().join("git-bar").display(),
        tmp.path().join("git-foo").display()
    );
    assert_eq!(stdout, expected);
}

#[test]
fn no_match_exits_nonzero() {
    let tmp = TempDir::new().unwrap();
    let (ok, stdout) = run(tmp.path(), "nothing-here");
    assert!(!ok);
    assert_eq!(stdout, "");
}

#[test]
fn bad_glob_is_rejected() {
    let out = Command::new(env!("CARGO_BIN_EXE_cmd-find"))
        .arg("a/b")
        .output()
        .unwrap();
    assert!(!out.status.success());
    assert!(
        String::from_utf8(out.stderr)
            .unwrap()
            .contains("may not contain '/'")
    );
}
