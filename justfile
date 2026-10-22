bin := "cmd-find"
install_dir := env_var("HOME") / ".local/bin"

# List recipes
default:
    @just --list

# Type-check without codegen
check:
    cargo check --all-targets

# Build the release binary
build:
    cargo build --release

# Run the test suite
test:
    cargo test

# Clippy (warnings are errors) and formatting check
lint:
    cargo clippy --all-targets -- -D warnings
    cargo fmt --check

# Run the binary with arguments, e.g. `just run 'cargo-*'`
run *ARGS:
    cargo run --release -- {{ARGS}}

# Test, lint, build, then copy the binary into ~/.local/bin (on PATH)
deploy: test lint build
    install -d {{install_dir}}
    install -m 755 target/release/{{bin}} {{install_dir}}/{{bin}}
    @echo "installed {{install_dir}}/{{bin}}"

# Remove the deployed binary
uninstall:
    rm -f {{install_dir}}/{{bin}}

# Remove build artifacts
clean:
    cargo clean
