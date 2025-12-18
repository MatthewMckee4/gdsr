# /// script
# requires-python = ">=3.13"
# dependencies = []
# ///

import re
from pathlib import Path

ROOT = Path(__file__).parent.parent

TAB = "    "


def prepare_index_file() -> None:
    """Copy root `README.md` to `docs/index.md`"""
    (ROOT / "docs" / "index.md").write_text((ROOT / "README.md").read_text())


def convert_markdown_admonitions(path: Path) -> None:
    """Convert GitHub-style alerts to MkDocs-style admonitions"""
    content = path.read_text()

    def replace_alert(match: re.Match[str]) -> str:
        alert_type = match.group(1).lower()
        lines = match.group(2).strip().split("\n")
        body_lines = [line.lstrip("> ").rstrip() for line in lines if line.strip()]
        body = f"\n{TAB}".join(body_lines)
        return f"!!! {alert_type}{TAB}{body}\n"

    pattern = r"> \[!(WARNING|NOTE|TIP|IMPORTANT|CAUTION)\]\n((?:>\s*.*\n?)*)"
    content = re.sub(pattern, replace_alert, content)

    path.write_text(content)


def main() -> None:
    prepare_index_file()
    convert_markdown_admonitions(ROOT / "docs" / "index.md")


if __name__ == "__main__":
    main()
