#!/usr/bin/env python3
import os
from pathlib import Path

ROOT_DIR = Path(__file__).resolve().parent.parent

def build_llms_full():
    version_file = ROOT_DIR / "VERSION"
    version = version_file.read_text().strip() if version_file.exists() else "0.1.0"

    output_lines = [
        "# blender-mcp-rs: Full Technical Context Corpus",
        f"Version: v{version}",
        "Authors: Zachary Shallbetter (@zachshallbetter)",
        "License: MIT OR Apache-2.0",
        "",
        "This file is a programmatically generated, consolidated single-file context corpus",
        "for zero-search AI agent resolution. Do not edit manually.",
        "",
        "=" * 80,
        "",
    ]

    docs_to_include = [
        "AUTHORITY.md",
        "VERSIONING.md",
        "AGENTS.md",
        "README.md",
        "CONTRIBUTING.md",
        "SECURITY.md",
        "Cargo.toml",
    ]

    for doc in docs_to_include:
        doc_path = ROOT_DIR / doc
        if doc_path.exists():
            output_lines.append(f"## FILE: {doc}\n")
            output_lines.append(doc_path.read_text().strip())
            output_lines.append("\n" + "=" * 80 + "\n")

    crates_dir = ROOT_DIR / "crates"
    if crates_dir.exists():
        for crate in sorted(os.listdir(crates_dir)):
            crate_path = crates_dir / crate
            if crate_path.is_dir():
                for root, _, files in os.walk(crate_path):
                    for file in sorted(files):
                        if file.endswith((".rs", ".toml")):
                            fp = Path(root) / file
                            rel_path = fp.relative_to(ROOT_DIR)
                            output_lines.append(f"## CRATE FILE: {rel_path}\n")
                            output_lines.append(fp.read_text().strip())
                            output_lines.append("\n" + "=" * 80 + "\n")

    full_text = "\n".join(output_lines) + "\n"
    out_path = ROOT_DIR / "llms-full.txt"
    out_path.write_text(full_text)
    print(f"Generated {out_path} ({len(full_text.encode('utf-8'))} bytes)")

if __name__ == "__main__":
    build_llms_full()
