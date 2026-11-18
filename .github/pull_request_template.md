<!-- Commit and PR titles are clear engineering prose. -->

## What and why

<!-- What does this change do, and what problem does it solve? -->

## How it was verified

- [ ] `cargo fmt --all -- --check` clean
- [ ] `cargo clippy --workspace --all-targets -- -D warnings` clean
- [ ] `cargo test --workspace` green
- [ ] `cargo run --bin blender-mcp -- client list --json` runs

## Notes

<!-- Trade-offs, follow-ups, anything reviewers should know. -->
