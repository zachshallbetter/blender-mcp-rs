# Authority and Conflict Resolution

**Status:** Canonical repository doctrine  
**Applies to:** `blender-mcp-rs`

## 1. Resolution Order

When repository artifacts, implementations, or AI agent outputs disagree, the canonical resolution order is:

$$\text{Architecture \& Doctrine} \to \text{AUTHORITY.md \& Governance} \to \text{Formal Protocols \& Schemas} \to \text{Substrate Code} \to \text{Generated Artifacts}$$

Strictly:
1. `AUTHORITY.md` and foundational architectural doctrine;
2. `AGENTS.md` for operational constraints and asset standards;
3. Normative schemas (`docs/schemas/`), protocol contracts, and MCP specifications;
4. Substrate Rust implementations (`crates/`) and empirical test suites;
5. Generated context (`llms.txt`, `llms-full.txt`), package manifests, release archives, and derivatives.

## 2. Epistemic Integrity & Claim Taxonomy

No release, PR, or artifact may conflate verification tiers:

$$\text{Documented} \neq \text{Implemented} \neq \text{Tested} \neq \text{Empirically Validated}$$

- **Documented**: Specified in an RFC, issue, or markdown document.
- **Implemented**: Compiling code exists in `crates/`.
- **Tested**: Verified via automated unit/integration tests with non-zero assertion counts.
- **Empirically Validated**: Verified in a live Blender runtime session with physical frame timing or game engine import.

## 3. Conflict Rule

A lower layer may expose a contradiction but cannot resolve it by altering the meaning of a higher layer. Record the conflict, preserve evidence, and resolve either by correcting the lower layer or approving a normative change at the controlling layer.

Generated artifacts never become authority by convenience. Edit the canonical source and regenerate.

## 4. Cited Protocols & Foundations

`blender-mcp-rs` derives its operational authority and contracts from:
- **`formal-project-bootstrap`**: Bound autonomous agent project substrate, authority resolution, and release governance.
- **`reflective-rust`**: Type reflection, Compiler Semantic Graph (`rr-csg-0.1`), access-aware symbol slicing, and prospective pre-registration.
- **`reflective-index`**: Deterministic local semantic code indexing and call graph attribution.
- **`Dead Hand` Asset Craftsmanship Protocol**: The 7-step Barrel Protocol for authentic game assets (2-material budget, quad UV mapping, physical 3D relief, and Rapier3D collision hulls).
