# Versioning Policy

`blender-mcp-rs` strictly enforces Semantic Versioning (SemVer 2.0.0).

- **Major (`X.0.0`)**: Breaking changes to the MCP tool protocol, IPC socket protocol, public Rust crate APIs, or command-line CLI interface.
- **Minor (`0.Y.0`)**: Backward-compatible additions of new MCP tools, geometry processors, agent client adapters, or performance optimizers.
- **Patch (`0.0.Z`)**: Backward-compatible bug fixes, test hardening, and performance corrections that do not alter the tool or schema contracts.

## Single Source of Truth

The canonical release version is stored exclusively in `VERSION`. 
Workspace `Cargo.toml` manifests read from this baseline; hardcoded version duplications across source files are forbidden.

## Independent Protocol Versioning

Following `formal-project-bootstrap` doctrine:
- External protocols bound by this project (MCP spec `2024-11-05`, glTF 2.0, Rapier3D) maintain their own independent versioning. Matching version numbers never imply cross-layer compatibility.
- Compatibility contracts must be verified by automated conformance tests before each release.

## Release Diligence & Pre-Release Gates

Before tagging any release:
1. `cargo test --workspace` passes with 0 failures and 0 warnings.
2. `cargo clippy --workspace -- -D warnings` executes with 0 violations.
3. Conformance tests verify serialization against standard MCP clients (Claude Code, Cursor, Antigravity, Codex, Gemini).
4. `llms-full.txt` context corpus is re-generated and checksum-verified.
5. `CHANGELOG.md` entry reflects reality under the Four-Tier Claim Taxonomy.
