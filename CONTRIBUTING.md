# Contributing to `blender-mcp-rs`

We welcome contributions that adhere to our craftsmanship standards and formal release discipline.

## Prerequisites

- **Rust**: Version 1.88+ (2021/2024 edition).
- **Blender**: Version 4.0+.

## Pre-PR Verification Gates

Before submitting any pull request, ensure all gates pass cleanly:

```bash
# 1. Format check
cargo fmt --all -- --check

# 2. Clippy lint check (zero warnings tolerated)
cargo clippy --workspace --all-targets -- -D warnings

# 3. Unit and integration test suite
cargo test --workspace

# 4. Conformance verification
cargo run --bin blender-mcp -- client list --json
```

## Craft & Doctrine Compliance

- All code changes must adhere to `AGENTS.md` and `AUTHORITY.md`.
- Tool additions must use `blender-mcp-reflect` for typed schema derivation rather than ad-hoc JSON dictionaries.
- 3D pipeline features must satisfy the asset craftsmanship rules defined in `Dead Hand`.
- Commits must feature clean engineering prose without automated attribution trailers.
