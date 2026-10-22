# cmd-find

`cmd-find` is like `which`, but the argument is a shell glob and every match on
`PATH` is printed, not just the first.

```
$ cmd-find 'cargo-c*'
/Users/hughbrown/.cargo/bin/cargo-chef
/Users/hughbrown/.cargo/bin/cargo-clippy
```

## Usage

```
cmd-find <PATTERN>
```

`PATTERN` is a glob matched against the file name of each entry in each `PATH`
directory. Quote it so the shell does not expand it first.

| Pattern | Matches |
|---|---|
| `git-*` | `git-lfs`, `git-flow`, ... |
| `python3.?` | `python3.9`, `python3.12` is **not** matched (`?` is one character) |
| `python3.1[0-9]` | `python3.10` through `python3.19` |
| `cargo` | exactly `cargo`, like plain `which` |

Supported syntax is that of the [`glob`](https://docs.rs/glob) crate: `*`, `?`,
`[abc]`, `[a-z]`, `[!abc]`. Matching is case-sensitive. The pattern may not
contain `/`, since only file names are matched.

Exit status:

| Status | Meaning |
|---|---|
| 0 | at least one match was printed |
| 1 | no match |
| 2 | bad arguments (invalid glob, missing pattern) |

## Behaviour

- **Each directory is searched once.** If a directory appears several times in
  `PATH`, only its first occurrence is used. Empty entries are ignored.
- **Searches run in parallel.** One task per distinct directory on a
  [rayon](https://docs.rs/rayon) thread pool.
- **Output is deterministic.** Results are printed in `PATH` order, and sorted
  by name within a directory.
- **Only executables count.** A match must be a regular file (symlinks are
  followed) with at least one execute bit set. Directories and plain data files
  are skipped.
- **Unreadable or missing directories are skipped silently**, as `which` does.

## Building and installing

Requires a Rust toolchain and [`just`](https://github.com/casey/just).

```
just check     # type-check without codegen
just test      # run unit, integration and doc tests
just lint      # clippy with warnings as errors, plus cargo fmt --check
just build     # release build into target/release/cmd-find
just deploy    # test + lint + build, then copy the binary to ~/.local/bin
just uninstall # remove ~/.local/bin/cmd-find
just run 'cargo-*'   # build and run with arguments
```

`just deploy` installs to `~/.local/bin`, which must be on your `PATH`.

Without `just`:

```
cargo build --release
install -m 755 target/release/cmd-find ~/.local/bin/
```

The project ships a `.cargo/config.toml` that links with `ld64.lld` on Apple
Silicon for faster builds. Install it with `brew install lld`, or delete that
file to use the default linker.

## Project layout

```
src/lib.rs      search logic: CommandGlob, unique_dirs, matches_in_dir, find
src/main.rs     clap argument parsing and output
tests/cli.rs    integration tests that run the real binary against a temporary PATH
docs/prompt.md  the original prompt this project was built from
justfile        build, test, lint and deploy recipes
```

The library is split from the binary so the search functions can be unit tested
with `tempfile` directories and reused from other crates.

## Dependencies

| Crate | Purpose |
|---|---|
| [clap](https://docs.rs/clap) | command line parsing |
| [rayon](https://docs.rs/rayon) | parallel directory search |
| [glob](https://docs.rs/glob) | glob pattern matching |
| [tempfile](https://docs.rs/tempfile) | test fixtures only |
