# BMCP-001: Standardize Namespace & Attribution to `/zachshallbetter`

**Epic:** EPIC-1 (Namespace & Attribution Standardization)  
**Owner:** A (Autonomous Agent)  
**Status:** doing  
**Target:** 2026-10-17  

---

## 1. Problem Statement

The user has directed that all repositories, authors, packages, and namespaces must be under `/zachshallbetter` (`Zachary Shallbetter`), with no references to any other organization or previous owners.

## 2. Scope of Changes

1. `Cargo.toml`:
   - `repository = "https://github.com/zachshallbetter/blender-mcp-rs"`
   - `authors = ["Zachary Shallbetter <zach@shallbetter.com>"]` (or `zachshallbetter`)
2. `CITATION.cff`:
   - Update repository-code and url to `https://github.com/zachshallbetter/blender-mcp-rs`.
   - Remove any non-`zachshallbetter` affiliation.
3. `.zenodo.json`:
   - Update description and creators to `Zachary Shallbetter`.
4. `AGENTS.md` & `README.md`:
   - Update repository URLs and owner tags to `zachshallbetter`.
5. `crates/blender-mcp-synth/src/biological.rs`:
   - Update manifest author field to `Zachary Shallbetter`.
6. `addon/blender_mcp/__init__.py`:
   - Update `bl_info["author"]` to `Zachary Shallbetter`.
7. `crates/blender-mcp-cli/src/main.rs`:
   - Update `#[command(author = "Zachary Shallbetter")]`.
8. `scripts/gen_llms.py`:
   - Update author string and regenerate `llms.txt` and `llms-full.txt`.

## 3. Success Criteria

- `grep -rnI "SnackStudio" .` returns 0 results.
- `cargo test --workspace` passes 100%.
- `cargo clippy --workspace --all-targets -- -D warnings` passes with 0 warnings.
