#!/usr/bin/env python3
import csv
import re
from collections import Counter, defaultdict
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
CATALOG_PATH = ROOT / "unchecked_method_base" / "stdlib-safe-unsafe-counterparts.md"
OUTPUT_DIR = ROOT / "API-counterprats" / "API-info" / "categories_from_stdlib_catalog"
MANIFEST_PATH = ROOT / "API-counterprats" / "API-info" / "categories_from_stdlib_catalog.md"


ENTRY_RE = re.compile(
    r"^### (?P<title_category>.+?) #\d+: `(?P<checked>.*?)` -> `(?P<unchecked>.*?)`\n\n"
    r"- Source file: `(?P<path>.*?)`\n"
    r"- Dependent library / type: `(?P<library>.*?)`\n"
    r"- Counterpart status: `(?P<status>.*?)`\n"
    r"- Safe function lines: `(?P<safe_start>.*?)`-`(?P<safe_end>.*?)`\n"
    r"- Unsafe function lines: `(?P<unsafe_start>.*?)`-`(?P<unsafe_end>.*?)`\n"
    r"- Safety condition: (?P<safety>.*?)\n"
    r"- Checked-side note: (?P<checked_note>.*?)\n"
    r"- User visible: (?P<user_visible>.*?)\n"
    r"- Flag: (?P<flag>.*?)\n\n"
    r"Safe source:\n```rust\n(?P<safe_source>.*?)\n```\n\n"
    r"Unsafe source:\n```rust\n(?P<unsafe_source>.*?)\n```",
    re.M | re.S,
)


def sanitize_filename(category: str) -> str:
    name = category.strip() or "Uncategorized"
    name = name.replace("/", "_")
    name = name.replace("?", "_question")
    name = re.sub(r"[^A-Za-z0-9_.-]+", "_", name)
    name = re.sub(r"_+", "_", name).strip("_.")
    return f"{name or 'Uncategorized'}.csv"


def load_entries() -> list[dict]:
    text = CATALOG_PATH.read_text(encoding="utf-8")
    entries: list[dict] = []
    for match in ENTRY_RE.finditer(text):
        entries.append(
            {
                "Path": match.group("path"),
                "Library": match.group("library"),
                "Start_line": match.group("unsafe_start"),
                "End_line": match.group("unsafe_end"),
                "Unchecked Func": match.group("unsafe_source"),
                "Checked Func": match.group("safe_source"),
                "Safety Conditions": match.group("safety"),
                "Safety Condition Category": match.group("title_category"),
                "User visible": match.group("user_visible"),
                "Flag": match.group("flag"),
                "Safe_start_line": match.group("safe_start"),
                "Safe_end_line": match.group("safe_end"),
                "Counterpart status": match.group("status"),
                "Checked-side note": match.group("checked_note"),
            }
        )
    return entries


def write_manifest(grouped: dict[str, list[dict]]) -> None:
    counts = Counter({category: len(items) for category, items in grouped.items()})
    lines = [
        "# Stdlib Counterpart Category Export",
        "",
        "These CSV files are generated from:",
        f"- `{CATALOG_PATH.relative_to(ROOT).as_posix()}`",
        "",
        f"- Total categories: `{len(grouped)}`",
        f"- Total entries: `{sum(counts.values())}`",
        "",
        "| Category | Entries | File |",
        "| --- | ---: | --- |",
    ]
    for category, count in sorted(counts.items(), key=lambda x: (-x[1], x[0].lower())):
        lines.append(f"| `{category}` | {count} | `{sanitize_filename(category)}` |")
    MANIFEST_PATH.write_text("\n".join(lines).rstrip() + "\n", encoding="utf-8")


def main() -> None:
    entries = load_entries()
    if not entries:
        raise SystemExit(f"No entries parsed from {CATALOG_PATH}")

    OUTPUT_DIR.mkdir(parents=True, exist_ok=True)
    grouped: dict[str, list[dict]] = defaultdict(list)
    for entry in entries:
        grouped[entry["Safety Condition Category"]].append(entry)

    headers = [
        "Path",
        "Library",
        "Start_line",
        "End_line",
        "Unchecked Func",
        "Checked Func",
        "Safety Conditions",
        "Safety Condition Category",
        "User visible",
        "Flag",
        "Safe_start_line",
        "Safe_end_line",
        "Counterpart status",
        "Checked-side note",
    ]

    for category, items in grouped.items():
        output_path = OUTPUT_DIR / sanitize_filename(category)
        with output_path.open("w", newline="", encoding="utf-8") as f:
            writer = csv.DictWriter(f, fieldnames=headers)
            writer.writeheader()
            for item in items:
                writer.writerow(item)

    write_manifest(grouped)
    print(f"Generated {len(grouped)} category CSV files in {OUTPUT_DIR.relative_to(ROOT)}")


if __name__ == "__main__":
    main()
