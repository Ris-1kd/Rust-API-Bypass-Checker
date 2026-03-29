#!/usr/bin/env python3
import csv
from collections import Counter, defaultdict
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
INPUT_DIR = ROOT / "API-counterprats" / "API-info" / "categories_from_stdlib_catalog"
OUTPUT_DIR = ROOT / "API-counterprats" / "API-info" / "normalized_families"
MANIFEST_PATH = ROOT / "API-counterprats" / "API-info" / "normalized_families.md"


def normalize_name(source: str) -> str:
    line = (source or "").splitlines()[0].strip()
    if " fn " in f" {line} ":
        after_fn = line.split("fn ", 1)[1]
        return after_fn.split("(", 1)[0].strip()
    return line


def classify(entry: dict) -> tuple[str, str]:
    category = entry.get("Safety Condition Category", "").strip()
    library = entry.get("Library", "").strip()
    checked_name = normalize_name(entry.get("Checked Func", ""))
    unchecked_name = normalize_name(entry.get("Unchecked Func", ""))
    names = f"{checked_name} {unchecked_name}"

    if category == "intrinsic":
        return (
            "Primitive_Escape_Hatches",
            "Original category is `intrinsic`; these are low-level escape hatches rather than clean checked wrappers.",
        )

    if category in {"pin"}:
        return (
            "Structural_Alias_Invariants",
            "Pin APIs encode pinning / move-prevention invariants rather than simple scalar predicates.",
        )

    if category in {"String", "CString", "buf", "slice/buf"}:
        return (
            "Content_Encoding_Validity",
            "String / byte-buffer APIs validate content or encoding properties.",
        )

    if category in {"Result/Option", "Reuslt/Option"}:
        return (
            "Variant_Type_Tag",
            "These APIs require a matching enum variant (`Some`, `Ok`, `Err`, etc.).",
        )

    if category in {"overflow", "overflow/nonzero"}:
        return (
            "Arithmetic_Representability",
            "These APIs guard arithmetic overflow / representability conditions.",
        )

    if category in {"nullptr", "nonzero", "type/nonzero"}:
        return (
            "Value_Validity",
            "These APIs check local value validity such as non-null, non-zero, or related invariants.",
        )

    if category == "type":
        if "float_to_int" in names or "to_int" in names:
            return (
                "Arithmetic_Representability",
                "Float/int conversion APIs are better modeled as representability checks than dynamic tags.",
            )
        return (
            "Variant_Type_Tag",
            "These APIs depend on dynamic type/tag agreement (e.g. downcast-style checks).",
        )

    if category in {"bound/nonzero"}:
        return (
            "Bounds_and_Ordering",
            "These APIs require non-zero sizes together with divisibility / in-bounds structural conditions.",
        )

    if category in {"vector"}:
        return (
            "Value_Validity",
            "These APIs require each lane/value to satisfy a local validity invariant.",
        )

    if category in {"index", "index?"}:
        if any(token in names for token in ["as_ascii", "from_u8", "digit_unchecked"]):
            return (
                "Content_Encoding_Validity",
                "Although previously grouped under `index`, the actual predicate is ASCII/content validity.",
            )
        if any(token in names for token in ["from_u32", "new_unchecked", "Alignment::new", "Alignment::new_unchecked"]):
            return (
                "Value_Validity",
                "These APIs check scalar validity such as Unicode scalar ranges or alignment validity.",
            )
        return (
            "Bounds_and_Ordering",
            "These APIs primarily require in-bounds indices or ordering/range constraints.",
        )

    if category == "itron":
        return (
            "Value_Validity",
            "These APIs mainly require non-null / valid handle-like values.",
        )

    if category == "sync":
        return (
            "Structural_Alias_Invariants",
            "These APIs depend on synchronization / initialization state invariants.",
        )

    if category == "thread":
        return (
            "Structural_Alias_Invariants",
            "These APIs rely on thread-local or lifecycle invariants rather than scalar arithmetic predicates.",
        )

    if category == "complex":
        if any(token in names for token in [
            "checked_add",
            "checked_sub",
            "checked_mul",
            "checked_neg",
            "checked_shl",
            "checked_shr",
            "unchecked_add",
            "unchecked_sub",
            "unchecked_mul",
            "unchecked_neg",
            "unchecked_shl",
            "unchecked_shr",
            "to_int_unchecked",
            "float_to_int_unchecked",
            "checked_disjoint_bitor",
            "unchecked_disjoint_bitor",
        ]):
            return (
                "Arithmetic_Representability",
                "These APIs are governed by arithmetic overflow or representability constraints.",
            )

        if any(token in names for token in [
            "from_utf8",
            "from_utf8_unchecked",
            "from_vec_unchecked",
            "from_vec_with_nul_unchecked",
            "from_encoded_bytes_unchecked",
            "from_boxed_utf8_unchecked",
        ]):
            return (
                "Content_Encoding_Validity",
                "These APIs validate UTF-8 / encoded-byte / C-string style content constraints.",
            )

        if any(token in names for token in [
            "insert_before",
            "insert_after",
            "get_disjoint",
            "map_unchecked",
            "map_unchecked_mut",
        ]):
            return (
                "Structural_Alias_Invariants",
                "These APIs preserve structural ordering, disjointness, or pinning/alias invariants.",
            )

        if any(token in names for token in [
            "get_unchecked",
            "get_unchecked_mut",
            "split_at_mut_unchecked",
            "advance_unchecked",
        ]):
            return (
                "Bounds_and_Ordering",
                "These APIs still fundamentally enforce in-bounds or range/order constraints.",
            )

        if any(token in names for token in ["from_mut_unchecked", "get_mut_unchecked", "as_ref_unchecked", "as_mut_unchecked"]):
            return (
                "Structural_Alias_Invariants",
                "These APIs depend on uniqueness, aliasing, or ownership-style invariants.",
            )

        return (
            "Structural_Alias_Invariants",
            "Residual `complex` entries mostly encode structural, aliasing, or multi-object invariants.",
        )

    return (
        "Unclassified_Review_Needed",
        "No normalization rule matched this entry; manual review is needed.",
    )


def read_entries() -> list[dict]:
    entries = []
    for path in sorted(INPUT_DIR.glob("*.csv")):
        with path.open(newline="", encoding="utf-8") as f:
            for row in csv.DictReader(f):
                family, rationale = classify(row)
                row["Normalized Family"] = family
                row["Normalization Rationale"] = rationale
                entries.append(row)
    return entries


def write_manifest(grouped: dict[str, list[dict]]) -> None:
    counts = Counter({family: len(items) for family, items in grouped.items()})
    lines = [
        "# Normalized Counterpart Families",
        "",
        "These files regroup the smaller legacy categories into larger predicate-oriented families.",
        "",
        "Family meanings:",
        "- `Bounds_and_Ordering`: index/range/order/divisibility style constraints.",
        "- `Arithmetic_Representability`: overflow / shift / conversion representability constraints.",
        "- `Value_Validity`: scalar validity such as non-null, non-zero, alignment, Unicode scalar validity.",
        "- `Variant_Type_Tag`: enum variant or dynamic type matching constraints.",
        "- `Content_Encoding_Validity`: UTF-8 / ASCII / encoded-byte / C-string style content checks.",
        "- `Structural_Alias_Invariants`: pinning, aliasing, ordering, disjointness, ownership-like structural invariants.",
        "- `Primitive_Escape_Hatches`: low-level intrinsics / hints / unchecked primitives not best treated as clean counterparts.",
        "",
        f"- Total families: `{len(grouped)}`",
        f"- Total entries: `{sum(counts.values())}`",
        "",
        "| Family | Entries | File |",
        "| --- | ---: | --- |",
    ]
    for family, count in sorted(counts.items(), key=lambda x: (-x[1], x[0].lower())):
        lines.append(f"| `{family}` | {count} | `{family}.csv` |")
    MANIFEST_PATH.write_text("\n".join(lines).rstrip() + "\n", encoding="utf-8")


def main() -> None:
    entries = read_entries()
    OUTPUT_DIR.mkdir(parents=True, exist_ok=True)
    grouped: dict[str, list[dict]] = defaultdict(list)
    for entry in entries:
        grouped[entry["Normalized Family"]].append(entry)

    headers = list(entries[0].keys())
    for family, items in grouped.items():
        out = OUTPUT_DIR / f"{family}.csv"
        with out.open("w", newline="", encoding="utf-8") as f:
            writer = csv.DictWriter(f, fieldnames=headers)
            writer.writeheader()
            writer.writerows(items)

    write_manifest(grouped)
    print(f"Generated {len(grouped)} normalized family CSV files in {OUTPUT_DIR.relative_to(ROOT)}")


if __name__ == "__main__":
    main()
