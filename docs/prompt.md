# Original prompt

I want a Rust command line application that finds executables in the PATH. It's like unix `which` but it takes a file glob.

crates: clap for command line arguments; rayon for parallel processing.

The execution should search a directory only once even if it appears multiple times in the PATH.

Broadly, the main loop looks like:
```
use std::collections:HashSet;

let s = HashSet<string>();
let matching = PATH.split(':')
    .par_iter()
    .map_fold(|p| if !s.contains(p) {s.insert(p); cmd_in_path(p, cmd)} else { None })
# cmd_in_path returns an Option<(Path, string)> if a command matches `cmd` in the directory `path`
    .collect();

for m in matching {
    # Path.join joins a Path and a string together into a string
    println!("{Path.join(m.path, m.cmd)}");
}
```

Make a `justfile` for commands to build, test, and deploy the binary. Put the resulting executable in the PATH.

Make a directory `docs` under this directory. Save this prompt in `docs/prompt.md`
