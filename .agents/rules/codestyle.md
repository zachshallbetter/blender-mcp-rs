---
description: Code style, compiler lint requirements, formatting rules, unsafe code boundaries, and concurrency safety
globs: ["**/*"]
---

# Code Style & Engineering Standards

This document defines the authoritative coding conventions, compiler lint requirements, formatting rules, and safety boundaries for `blender-mcp-rs`.

---

## 1. Toolchain & Workspace Standards

- **Rust Edition**: 2021
- **Cargo Resolver**: Version 2
- **Monorepo Crate Inheritance**:
  All crates in `crates/` inherit workspace package metadata:
  ```toml
  [package]
  version.workspace = true
  edition.workspace = true
  license.workspace = true
  authors.workspace = true
  ```

---

## 2. Formatting Standards

All Rust code must adhere to standard formatting:
- Wide expressions where appropriate (`max_width = 120`).
- Trailing commas in multiline structs, enums, match arms, and function parameters.
- Clean module hierarchy and zero wildcard imports in public APIs.

---

## 3. Strict Quality Lints & Compiler Warning Zero-Tolerance

In CI and pre-commit checks, compiler warnings are strictly treated as build errors:
```bash
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

- **Never-Crash Policy in Core**:
  Production daemon code must model fallibility through explicit `Result<T, E>` and `Option<T>` types. Avoid uncontrolled panics in request handlers.

---

## 4. Unsafe Code Boundaries

- High-level logic and protocol handling are safe Rust.
- Where low-level memory mapping (`memmap2`) or hardware interaction requires `unsafe`:
  1. Isolate in foundation crates (`blender-mcp-ipc`).
  2. Document with explicit `// SAFETY:` rationale explaining pointer alignment, memory bounds, and concurrency guarantees.
  3. Validate through automated roundtrip tests.

---

## 5. Concurrency & IPC Safety

- **Graceful Lock Handling**:
  Never panic when acquiring mutexes. Use `lock().unwrap_or_else(PoisonError::into_inner)`.
- **Bounded Resource Allocations**:
  Memory-mapped ring buffers and TCP payload buffers must be explicitly bounded to prevent buffer overflow attacks.
