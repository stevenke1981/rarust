from __future__ import annotations

import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]

REQUIRED_FILES = [
    "AGENTS.md",
    "TEAM.md",
    "README.md",
    "usage.md",
    "opencode.jsonc",
    "manifest.json",
    "scripts/validate.ps1",
    "scripts/validate.py",
    ".agents/skills/coding-improve-team/SKILL.md",
]

REQUIRED_DIRS = [
    "roles",
    "workflows",
    "policies",
    "templates",
    ".opencode/commands",
]

REQUIRED_TEMPLATES = [
    "spec.md",
    "plan.md",
    "todos.md",
    "test.md",
    "final.md",
]


def fail(message: str) -> None:
    print(f"[FAIL] {message}")
    sys.exit(1)


def strip_jsonc(text: str) -> str:
    text = re.sub(r"//.*", "", text)
    text = re.sub(r"/\*.*?\*/", "", text, flags=re.S)
    return text


def check_required_files() -> None:
    for rel in REQUIRED_FILES:
        if not (ROOT / rel).is_file():
            fail(f"Missing required file: {rel}")


def check_required_dirs() -> None:
    for rel in REQUIRED_DIRS:
        path = ROOT / rel
        if not path.is_dir():
            fail(f"Missing required directory: {rel}")
        if not list(path.glob("*.md")):
            fail(f"Directory has no markdown files: {rel}")


def check_templates() -> None:
    for name in REQUIRED_TEMPLATES:
        if not (ROOT / "templates" / name).is_file():
            fail(f"Missing template: templates/{name}")


def check_markdown_structure() -> None:
    for path in ROOT.rglob("*.md"):
        text = path.read_text(encoding="utf-8")
        if not text.strip():
            fail(f"Empty markdown file: {path.relative_to(ROOT)}")
        if not re.search(r"^#\s+", text, flags=re.M):
            fail(f"Markdown file missing H1 heading: {path.relative_to(ROOT)}")
        for i, line in enumerate(text.splitlines(), start=1):
            if line.rstrip() != line:
                fail(f"Trailing whitespace: {path.relative_to(ROOT)}:{i}")


def check_opencode() -> None:
    path = ROOT / "opencode.jsonc"
    data = json.loads(strip_jsonc(path.read_text(encoding="utf-8")))
    instructions = data.get("instructions")
    if not isinstance(instructions, list) or not instructions:
        fail("opencode.jsonc must contain non-empty instructions list")


def check_manifest() -> None:
    data = json.loads((ROOT / "manifest.json").read_text(encoding="utf-8"))
    if data.get("name") != "coding-improve-team":
        fail("manifest.json name must be coding-improve-team")
    for rel in data.get("required_files", []):
        if not (ROOT / rel).exists():
            fail(f"manifest required file missing: {rel}")


def main() -> None:
    check_required_files()
    check_required_dirs()
    check_templates()
    check_markdown_structure()
    check_opencode()
    check_manifest()
    print("Validation passed.")


if __name__ == "__main__":
    main()
