#!/usr/bin/env python3
import csv
import re
from collections import Counter, defaultdict
from pathlib import Path


ROOT = Path(__file__).resolve().parent.parent
CSV_PATH = ROOT / "unchecked_method_base" / "base.csv"
RUST_ROOT = ROOT / "rust"
OUTPUT_PATH = ROOT / "unchecked_method_base" / "stdlib-safe-unsafe-counterparts.md"


def normalize_source_path(raw: str) -> Path:
    raw = raw.strip()
    if raw.startswith("../../refs/rust/"):
        return ROOT / raw.replace("../../refs/rust/", "rust/", 1)
    if raw.startswith("./rust/"):
        return ROOT / raw[2:]
    return ROOT / raw


def first_signature_line(snippet: str) -> str:
    for line in (snippet or "").splitlines():
        s = line.strip()
        if s:
            return s
    return ""


def extract_fn_name(snippet: str) -> str | None:
    m = re.search(r"\bfn\s+([A-Za-z0-9_]+)\b", snippet or "")
    return m.group(1) if m else None


def derive_checked_name(unchecked_name: str | None) -> str | None:
    if not unchecked_name:
        return None
    if unchecked_name.endswith("_unchecked"):
        return unchecked_name[: -len("_unchecked")]
    if unchecked_name.startswith("unchecked_"):
        return "checked_" + unchecked_name[len("unchecked_") :]
    return None


def extract_block(lines: list[str], start_idx: int) -> tuple[int, int, str] | None:
    start = start_idx
    saw_open_brace = False
    brace_depth = 0
    out: list[str] = []

    for i in range(start_idx, len(lines)):
        line = lines[i]
        out.append(line)
        if "{" in line:
            saw_open_brace = True
        if saw_open_brace:
            brace_depth += line.count("{") - line.count("}")
            if brace_depth <= 0:
                return start + 1, i + 1, "".join(out).rstrip()

    return None


def find_function_in_file(path: Path, fn_name: str, unsafe_required: bool | None) -> tuple[int, int, str] | None:
    if not path.exists():
        return None

    lines = path.read_text(encoding="utf-8", errors="replace").splitlines(keepends=True)
    pattern = re.compile(rf"\bfn\s+{re.escape(fn_name)}\b")

    for i, line in enumerate(lines):
        if not pattern.search(line):
            continue
        is_unsafe = "unsafe fn" in line or "unsafe extern" in line
        if unsafe_required is True and not is_unsafe:
            continue
        if unsafe_required is False and is_unsafe:
            continue
        block = extract_block(lines, i)
        if block:
            return block
    return None


def clean_text(text: str) -> str:
    text = (text or "").strip()
    if not text:
        return "Not recorded."
    text = text.replace("\uFFFD", "?")
    try:
        text = text.encode("latin1").decode("gb18030")
    except (UnicodeEncodeError, UnicodeDecodeError):
        pass
    return text


def derive_checked_candidates(unchecked_name: str | None) -> list[str]:
    if not unchecked_name:
        return []

    candidates: list[str] = []
    if unchecked_name.endswith("_unchecked"):
        candidates.append(unchecked_name[: -len("_unchecked")])
    if "_unchecked_" in unchecked_name:
        candidates.append(unchecked_name.replace("_unchecked_", "_"))
    if unchecked_name.startswith("unchecked_"):
        candidates.append("checked_" + unchecked_name[len("unchecked_") :])

    seen = set()
    ordered: list[str] = []
    for item in candidates:
        if item and item not in seen:
            seen.add(item)
            ordered.append(item)
    return ordered


def load_rows() -> list[dict]:
    with CSV_PATH.open(encoding="iso-8859-1", errors="replace") as f:
        return list(csv.DictReader(f))


def resolve_entry(row: dict) -> dict:
    source_path = normalize_source_path(row["Path"])
    unsafe_name = extract_fn_name(row.get("Unchecked Func", ""))
    checked_name = extract_fn_name(row.get("Checked Func", ""))

    unsafe_source = clean_text(row.get("Unchecked Func", ""))
    checked_field_text = clean_text(row.get("Checked Func", ""))
    safe_source = checked_field_text
    unsafe_start = row.get("Start_line", "").strip() or "?"
    unsafe_end = row.get("End_line", "").strip() or "?"
    safe_start = "?"
    safe_end = "?"

    resolved_unsafe = find_function_in_file(source_path, unsafe_name, True) if unsafe_name else None
    if resolved_unsafe:
        unsafe_start, unsafe_end, unsafe_source = resolved_unsafe

    checked_candidates = []
    if checked_name:
        checked_candidates.append(checked_name)
    checked_candidates.extend(derive_checked_candidates(unsafe_name))

    resolved_safe = None
    for candidate in checked_candidates:
        resolved_safe = find_function_in_file(source_path, candidate, False)
        if resolved_safe:
            checked_name = candidate
            break
    if resolved_safe:
        safe_start, safe_end, safe_source = resolved_safe
    elif not checked_name and checked_candidates:
        checked_name = checked_candidates[0]

    counterpart_status = "confirmed_pair" if resolved_safe else "unsafe_entry_only_or_unclear_pair"

    return {
        "library": clean_text(row.get("Library", "")),
        "category": clean_text(row.get("Safety Condition Category", "")) or "Uncategorized",
        "source_path": source_path.relative_to(ROOT).as_posix() if source_path.exists() else row.get("Path", "").strip(),
        "unsafe_name": unsafe_name or "?",
        "checked_name": checked_name or "?",
        "unsafe_start": unsafe_start,
        "unsafe_end": unsafe_end,
        "safe_start": safe_start,
        "safe_end": safe_end,
        "unsafe_source": unsafe_source,
        "safe_source": safe_source,
        "checked_note": checked_field_text if not extract_fn_name(row.get("Checked Func", "")) else "Not recorded.",
        "safety_conditions": clean_text(row.get("Safety Conditions", "")),
        "user_visible": clean_text(row.get("User visible", "")),
        "flag": clean_text(row.get("Flag", "")),
        "counterpart_status": counterpart_status,
    }


def sort_key(entry: dict) -> tuple:
    return (
        entry["category"].lower(),
        entry["library"].lower(),
        entry["source_path"],
        entry["checked_name"],
        entry["unsafe_name"],
    )


def render(entries: list[dict]) -> str:
    total = len(entries)
    by_category = Counter(entry["category"] for entry in entries)
    by_library = Counter(entry["library"] for entry in entries)
    by_status = Counter(entry["counterpart_status"] for entry in entries)

    lines: list[str] = []
    lines.append("# Rust `core` / `alloc` / `std` Safe/Unsafe Counterpart Catalog")
    lines.append("")
    lines.append("This document is generated from `unchecked_method_base/base.csv` and cross-checked against the local Rust source tree under `./rust/library` when possible.")
    lines.append("")
    lines.append("Status meaning:")
    lines.append("- `confirmed_pair`: both checked and unchecked function implementations were located in the Rust source tree.")
    lines.append("- `unsafe_entry_only_or_unclear_pair`: the CSV records an unchecked entry, but a clean checked-side counterpart was not located automatically in the same file.")
    lines.append("")
    lines.append(f"- Total recorded pairs: `{total}`")
    lines.append(f"- Confirmed pairs by source lookup: `{by_status['confirmed_pair']}`")
    lines.append(f"- Unclear / unsafe-only entries: `{by_status['unsafe_entry_only_or_unclear_pair']}`")
    lines.append(f"- Categories: `{len(by_category)}`")
    lines.append(f"- Library/type buckets: `{len(by_library)}`")
    lines.append("")
    lines.append("## Summary by Category")
    lines.append("")
    lines.append("| Category | Count |")
    lines.append("| --- | ---: |")
    for category, count in sorted(by_category.items(), key=lambda x: (-x[1], x[0].lower())):
        lines.append(f"| `{category}` | {count} |")
    lines.append("")
    lines.append("## Summary by Library / Dependent Type")
    lines.append("")
    lines.append("| Library / Dependent Type | Count |")
    lines.append("| --- | ---: |")
    for library, count in sorted(by_library.items(), key=lambda x: (-x[1], x[0].lower())):
        lines.append(f"| `{library}` | {count} |")
    lines.append("")

    grouped: dict[str, list[dict]] = defaultdict(list)
    for entry in entries:
        grouped[entry["category"]].append(entry)

    for category in sorted(grouped.keys(), key=lambda x: x.lower()):
        lines.append(f"## Category: `{category}`")
        lines.append("")
        for idx, entry in enumerate(grouped[category], 1):
            lines.append(f"### {category} #{idx}: `{entry['checked_name']}` -> `{entry['unsafe_name']}`")
            lines.append("")
            lines.append(f"- Source file: `{entry['source_path']}`")
            lines.append(f"- Dependent library / type: `{entry['library']}`")
            lines.append(f"- Counterpart status: `{entry['counterpart_status']}`")
            lines.append(f"- Safe function lines: `{entry['safe_start']}`-`{entry['safe_end']}`")
            lines.append(f"- Unsafe function lines: `{entry['unsafe_start']}`-`{entry['unsafe_end']}`")
            lines.append(f"- Safety condition: {entry['safety_conditions']}")
            lines.append(f"- Checked-side note: {entry['checked_note']}")
            lines.append(f"- User visible: {entry['user_visible']}")
            lines.append(f"- Flag: {entry['flag']}")
            lines.append("")
            lines.append("Safe source:")
            lines.append("```rust")
            lines.append(entry["safe_source"])
            lines.append("```")
            lines.append("")
            lines.append("Unsafe source:")
            lines.append("```rust")
            lines.append(entry["unsafe_source"])
            lines.append("```")
            lines.append("")

    return "\n".join(lines).rstrip() + "\n"


def main() -> None:
    rows = load_rows()
    entries = [resolve_entry(row) for row in rows]
    entries.sort(key=sort_key)
    OUTPUT_PATH.write_text(render(entries), encoding="utf-8")
    print(f"Generated {OUTPUT_PATH.relative_to(ROOT)} with {len(entries)} entries.")


if __name__ == "__main__":
    main()
