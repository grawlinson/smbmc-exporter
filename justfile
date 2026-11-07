_default:
  just --list

[doc('Build a release binary')]
build:
    cargo build --release --locked

[doc('Build the Python wheel/sdist (wraps the binary)')]
wheel:
    uvx maturin build --release --locked --out dist
    uvx maturin sdist --out dist

[doc('Run the test suite')]
test *ARGS:
    cargo test --locked {{ ARGS }}

[doc('Regenerate tests/fixtures/metrics.txt')]
update-golden:
    UPDATE_GOLDEN=1 cargo test --locked --test exporter golden_output

[doc('Lint with `cargo fmt` and `cargo clippy`')]
check:
    cargo fmt --all --check
    cargo clippy --all-targets --locked -- -D warnings

[doc('Format Rust files with `cargo fmt`')]
format:
    cargo fmt --all

[doc('Generate coverage report')]
test-cov *ARGS:
    cargo llvm-cov --locked --lcov --output-path rust.lcov {{ ARGS }}

[doc('Run the exporter (configure with SMBMC_* environment variables)')]
run *ARGS:
    cargo run -- {{ ARGS }}

[doc('Remove all files ignored from git')]
clean:
  git clean -fX .
