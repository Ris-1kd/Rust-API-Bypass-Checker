#!/usr/bin/env python3
import csv
import re
from dataclasses import dataclass
from pathlib import Path


BASE_DIR = Path(__file__).resolve().parent
ROOT = Path(__file__).resolve().parents[3]
RUST_ROOTS = [
    ROOT / "rust" / "library" / "core" / "src",
    ROOT / "rust" / "library" / "alloc" / "src",
    ROOT / "rust" / "library" / "std" / "src",
]
OUTPUT_CSV = BASE_DIR / "public_unchecked_pairs.csv"
OUTPUT_MD = BASE_DIR / "public_unchecked_pairs.md"

PUB_FN_RE = re.compile(
    r"^(?P<indent>\s*)pub\s+(?:(?:const|async)\s+)?(?:(?:unsafe)\s+)?fn\s+(?P<name>[A-Za-z_][A-Za-z0-9_]*)\s*\(",
    re.M,
)
STABILITY_RE = re.compile(r"#\[(stable|unstable)\(")
DOC_LINE_RE = re.compile(r"^\s*///\s?(.*)$")


@dataclass
class PublicFunction:
    path: str
    name: str
    start_line: int
    source: str
    docs: str
    attributes: str


def find_block_start(text: str, pos: int) -> int:
    return text.rfind("\n", 0, pos) + 1


def line_number_at(text: str, pos: int) -> int:
    return text.count("\n", 0, pos) + 1


def collect_prefix_lines(text: str, start: int) -> list[str]:
    lines = text[:start].splitlines()
    prefix: list[str] = []
    i = len(lines) - 1
    while i >= 0:
        line = lines[i]
        stripped = line.strip()
        if not stripped:
            if prefix:
                prefix.append(line)
            i -= 1
            continue
        if stripped.startswith("///") or stripped.startswith("#["):
            prefix.append(line)
            i -= 1
            continue
        break
    prefix.reverse()
    return prefix


def is_user_facing(text: str, start: int) -> bool:
    previous_lines = text[:start].splitlines()[-40:]
    return bool(STABILITY_RE.search("\n".join(previous_lines)))


def extract_docs(prefix_lines: list[str]) -> str:
    docs = []
    for line in prefix_lines:
        match = DOC_LINE_RE.match(line)
        if match:
            docs.append(match.group(1))
    return "\n".join(docs).strip()


def extract_source(text: str, start: int) -> str:
    i = start
    while i < len(text) and text[i] not in "{;":
        i += 1
    if i >= len(text):
        return text[start:].rstrip()
    if text[i] == ";":
        return text[start : i + 1].rstrip()

    depth = 0
    j = i
    in_string = False
    in_char = False
    in_line_comment = False
    in_block_comment = 0
    escaping = False

    while j < len(text):
        ch = text[j]
        nxt = text[j + 1] if j + 1 < len(text) else ""

        if in_line_comment:
            if ch == "\n":
                in_line_comment = False
            j += 1
            continue

        if in_block_comment:
            if ch == "/" and nxt == "*":
                in_block_comment += 1
                j += 2
                continue
            if ch == "*" and nxt == "/":
                in_block_comment -= 1
                j += 2
                continue
            j += 1
            continue

        if in_string:
            if escaping:
                escaping = False
            elif ch == "\\":
                escaping = True
            elif ch == '"':
                in_string = False
            j += 1
            continue

        if in_char:
            if escaping:
                escaping = False
            elif ch == "\\":
                escaping = True
            elif ch == "'":
                in_char = False
            j += 1
            continue

        if ch == "/" and nxt == "/":
            in_line_comment = True
            j += 2
            continue
        if ch == "/" and nxt == "*":
            in_block_comment = 1
            j += 2
            continue
        if ch == '"':
            in_string = True
            j += 1
            continue
        if ch == "'":
            in_char = True
            j += 1
            continue
        if ch == "{":
            depth += 1
        elif ch == "}":
            depth -= 1
            if depth == 0:
                return text[start : j + 1].rstrip()
        j += 1

    return text[start:].rstrip()


def scan_public_functions() -> list[PublicFunction]:
    functions: list[PublicFunction] = []
    for root in RUST_ROOTS:
        for path in sorted(root.rglob("*.rs")):
            text = path.read_text(encoding="utf-8", errors="ignore")
            for match in PUB_FN_RE.finditer(text):
                start = find_block_start(text, match.start())
                prefix_lines = collect_prefix_lines(text, start)
                if not is_user_facing(text, start):
                    continue
                functions.append(
                    PublicFunction(
                        path=path.relative_to(ROOT).as_posix(),
                        name=match.group("name"),
                        start_line=line_number_at(text, start),
                        source=extract_source(text, start),
                        docs=extract_docs(prefix_lines),
                        attributes="\n".join(prefix_lines).strip(),
                    )
                )
    return functions


def candidate_safe_names(unchecked_name: str) -> list[str]:
    candidates: list[str] = []
    if "_unchecked" in unchecked_name:
        candidates.append(unchecked_name.replace("_unchecked", ""))
    if unchecked_name.startswith("unchecked_"):
        tail = unchecked_name[len("unchecked_") :]
        candidates.append(tail)
        candidates.append(f"checked_{tail}")
        if tail.endswith("_exact"):
            prefix = tail[: -len("_exact")]
            candidates.append(f"checked_exact_{prefix}")
    if unchecked_name.endswith("_unchecked"):
        tail = unchecked_name[: -len("_unchecked")]
        candidates.append(tail)
    deduped = []
    for name in candidates:
        if name and name not in deduped:
            deduped.append(name)
    return deduped


def pick_safe_counterpart(unchecked: PublicFunction, all_functions: list[PublicFunction]) -> PublicFunction | None:
    candidates = candidate_safe_names(unchecked.name)
    same_file = [
        func
        for func in all_functions
        if func.path == unchecked.path and func.name in candidates and "unchecked" not in func.name
    ]
    if same_file:
        same_file.sort(key=lambda func: abs(func.start_line - unchecked.start_line))
        return same_file[0]
    return None


def extract_safety_condition(unchecked: PublicFunction) -> str:
    docs = unchecked.docs
    if docs:
        lines = docs.splitlines()
        capture = False
        collected: list[str] = []
        for line in lines:
            stripped = line.strip()
            if stripped == "# Safety":
                capture = True
                continue
            if capture:
                if stripped.startswith("# ") and stripped != "# Safety":
                    break
                if stripped.startswith("```"):
                    if collected:
                        break
                    continue
                collected.append(stripped)
        condition = " ".join(part for part in collected if part).strip()
        if condition:
            return condition

    precondition_match = re.search(
        r'assert_unsafe_precondition!\s*\([\s\S]*?"([^"]+)"',
        unchecked.source,
    )
    if precondition_match:
        return precondition_match.group(1).strip()

    first_sentence = docs.split(".")[0].strip() if docs else ""
    return first_sentence or "Not extracted automatically."


def write_outputs(rows: list[dict]) -> None:
    OUTPUT_CSV.parent.mkdir(parents=True, exist_ok=True)
    with OUTPUT_CSV.open("w", newline="", encoding="utf-8") as f:
        writer = csv.DictWriter(
            f,
            fieldnames=["Location", "Safe Source", "Unsafe Source", "Safety Condition"],
        )
        writer.writeheader()
        writer.writerows(rows)

    lines = [
        "# Public Explicit `unchecked` Function Catalog",
        "",
        "Generated directly from the local `rust/library/{core,alloc,std}/src` source tree.",
        "",
        f"- Total public explicit `unchecked` entries: `{len(rows)}`",
        f"- Entries with same-file safe counterpart found automatically: `{sum(1 for row in rows if row['Safe Source'])}`",
        f"- Entries without same-file safe counterpart: `{sum(1 for row in rows if not row['Safe Source'])}`",
        "",
        f"- CSV: `{OUTPUT_CSV.relative_to(ROOT).as_posix()}`",
    ]
    OUTPUT_MD.write_text("\n".join(lines).rstrip() + "\n", encoding="utf-8")


def main() -> None:
    functions = scan_public_functions()
    unchecked_functions = [func for func in functions if "unchecked" in func.name]
    rows = []
    for unchecked in unchecked_functions:
        safe = pick_safe_counterpart(unchecked, functions)
        rows.append(
            {
                "Location": f"{unchecked.path}:{unchecked.start_line}",
                "Safe Source": safe.source if safe else "",
                "Unsafe Source": unchecked.source,
                "Safety Condition": extract_safety_condition(unchecked),
            }
        )
    write_outputs(rows)
    print(f"Generated {len(rows)} public explicit unchecked entries")


if __name__ == "__main__":
    main()
