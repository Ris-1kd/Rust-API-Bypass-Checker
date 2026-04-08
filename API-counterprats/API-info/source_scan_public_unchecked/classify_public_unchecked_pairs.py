#!/usr/bin/env python3
import csv
import re
from collections import Counter, defaultdict
from pathlib import Path


BASE_DIR = Path(__file__).resolve().parent
ROOT = Path(__file__).resolve().parents[3]
INPUT_CSV = BASE_DIR / "public_unchecked_pairs.csv"
CLASSIFIED_CSV = BASE_DIR / "public_unchecked_classified.csv"
CANDIDATES_CSV = BASE_DIR / "public_unchecked_candidates.csv"
EXCLUDED_CSV = BASE_DIR / "public_unchecked_excluded.csv"
FAMILIES_DIR = BASE_DIR / "public_unchecked_families"
SUMMARY_MD = BASE_DIR / "public_unchecked_classification.md"

FN_NAME_RE = re.compile(r"fn\s+([A-Za-z_][A-Za-z0-9_]*)\s*\(")

FAMILY_ORDER = [
    "Layout_Initialization_and_Shape",
    "Value_Validity_and_Representability",
    "Arithmetic_and_Bit_Preconditions",
    "Bounds_Length_and_Position",
    "Encoding_and_Text_Validity",
    "Aliasing_Pinning_and_Exclusive_Access",
    "Variant_and_Control_Flow_Assumptions",
    "Ordering_and_Cursor_Position",
]

EXCLUDED_NOTES = {
    "rust/library/core/src/array/iter.rs:141": (
        "Safe `new` consumes a fully initialized array, while `new_unchecked` rebuilds an "
        "iterator from a partially initialized buffer plus range state; not a clean safe/unsafe pair."
    ),
    "rust/library/core/src/char/methods.rs:2106": (
        "Safe `encode_utf8_raw` uses a slice and returns the written subslice, while the unchecked "
        "entry takes a raw pointer and only assumes capacity; interface mismatch for direct benchmarking."
    ),
    "rust/library/core/src/cell.rs:2533": (
        "No direct safe counterpart; this is an aliasing escape hatch on `UnsafeCell`."
    ),
    "rust/library/core/src/cell.rs:2562": (
        "No direct safe counterpart; this is an aliasing escape hatch on `UnsafeCell`."
    ),
    "rust/library/core/src/hint.rs:103": (
        "No direct safe counterpart; compiler-assumption intrinsic-style API."
    ),
    "rust/library/core/src/hint.rs:202": (
        "No direct safe counterpart; compiler-assumption intrinsic-style API."
    ),
    "rust/library/core/src/num/uint_macros.rs:1631": (
        "No direct checked/safe companion was found in the same numeric API family."
    ),
    "rust/library/core/src/pin/unsafe_pinned.rs:85": (
        "Unstable helper on `UnsafePinned`; no direct safe counterpart and not a good user-level benchmark target."
    ),
    "rust/library/core/src/ptr/unique.rs:86": (
        "Unstable `ptr_internals` helper type; not a good user-level benchmark target."
    ),
    "rust/library/core/src/str/mod.rs:769": (
        "No direct safe function counterpart; the safe alternative is slicing syntax / trait indexing, not an isomorphic API."
    ),
    "rust/library/core/src/str/mod.rs:803": (
        "No direct safe function counterpart; the safe alternative is slicing syntax / trait indexing, not an isomorphic API."
    ),
    "rust/library/alloc/src/ffi/c_str.rs:340": (
        "No direct clean safe counterpart; nearby safe constructors validate a different contract."
    ),
    "rust/library/alloc/src/str.rs:616": (
        "No direct safe boxed-UTF8 constructor exists in the same shape."
    ),
    "rust/library/std/src/ffi/os_str.rs:184": (
        "No clean safe counterpart; platform-specific encoded-byte reconstruction is intentionally unsafe-only."
    ),
    "rust/library/std/src/ffi/os_str.rs:881": (
        "No clean safe counterpart; platform-specific encoded-byte reconstruction is intentionally unsafe-only."
    ),
}


def extract_fn_name(source: str) -> str:
    match = FN_NAME_RE.search(source)
    return match.group(1) if match else ""


def classify_family(location: str, unsafe_name: str, safety: str) -> str:
    lower = safety.lower()

    if location in {
        "rust/library/core/src/alloc/layout.rs:132",
        "rust/library/core/src/alloc/layout.rs:156",
        "rust/library/core/src/array/iter.rs:141",
    }:
        return "Layout_Initialization_and_Shape"

    if unsafe_name in {
        "unchecked_add",
        "unchecked_sub",
        "unchecked_mul",
        "unchecked_div_exact",
        "unchecked_neg",
        "unchecked_shl",
        "unchecked_shl_exact",
        "unchecked_shr",
        "unchecked_shr_exact",
        "unchecked_funnel_shl",
        "unchecked_funnel_shr",
        "unchecked_disjoint_bitor",
    }:
        return "Arithmetic_and_Bit_Preconditions"

    if unsafe_name in {
        "swap_unchecked",
        "split_at_unchecked",
        "split_at_mut_unchecked",
        "slice_unchecked",
        "slice_mut_unchecked",
        "encode_utf8_raw_unchecked",
    }:
        return "Bounds_Length_and_Position"

    if unsafe_name in {
        "from_bytes_with_nul_unchecked",
        "from_utf8_unchecked",
        "from_utf8_unchecked_mut",
        "from_vec_unchecked",
        "from_vec_with_nul_unchecked",
        "from_boxed_utf8_unchecked",
        "from_encoded_bytes_unchecked",
    }:
        return "Encoding_and_Text_Validity"

    if unsafe_name in {
        "as_ref_unchecked",
        "as_mut_unchecked",
        "new_unchecked",
        "into_inner_unchecked",
        "get_unchecked_mut",
        "get_mut_unchecked",
    } and (
        "pin" in location
        or "cell.rs" in location
        or "rc.rs" in location
        or "sync.rs" in location
    ):
        return "Aliasing_Pinning_and_Exclusive_Access"

    if unsafe_name in {
        "unwrap_unchecked",
        "unwrap_err_unchecked",
        "unreachable_unchecked",
        "assert_unchecked",
    }:
        return "Variant_and_Control_Flow_Assumptions"

    if unsafe_name in {"insert_after_unchecked", "insert_before_unchecked"}:
        return "Ordering_and_Cursor_Position"

    if (
        unsafe_name in {
            "from_u8_unchecked",
            "digit_unchecked",
            "from_u32_unchecked",
            "as_ascii_unchecked",
            "from_mut_unchecked",
        }
        or "must be non-null" in lower
        or "must not be zero" in lower
        or "power of two" in lower
        or "invalid `char` values" in lower
        or "ascii" in lower
    ):
        return "Value_Validity_and_Representability"

    return "Value_Validity_and_Representability"


def benchmark_status(location: str, safe_source: str) -> tuple[str, str]:
    if location in EXCLUDED_NOTES:
        return "excluded", EXCLUDED_NOTES[location]
    if not safe_source.strip():
        return "excluded", "No direct same-file safe counterpart was found automatically in the source scan."
    return "candidate", "Same-file safe counterpart found; suitable for group-internal benchmark discussion."


def output_name(family: str) -> str:
    return f"{family}.csv"


def load_rows() -> list[dict[str, str]]:
    with INPUT_CSV.open(newline="", encoding="utf-8") as f:
        return list(csv.DictReader(f))


def main() -> None:
    rows = load_rows()
    classified_rows: list[dict[str, str]] = []

    for row in rows:
        unsafe_name = extract_fn_name(row["Unsafe Source"])
        safe_name = extract_fn_name(row["Safe Source"])
        family = classify_family(row["Location"], unsafe_name, row["Safety Condition"])
        status, note = benchmark_status(row["Location"], row["Safe Source"])
        classified_rows.append(
            {
                "Location": row["Location"],
                "Unsafe Name": unsafe_name,
                "Safe Name": safe_name,
                "Safety Family": family,
                "Benchmark Status": status,
                "Benchmark Note": note,
                "Safe Source": row["Safe Source"],
                "Unsafe Source": row["Unsafe Source"],
                "Safety Condition": row["Safety Condition"],
            }
        )

    classified_rows.sort(
        key=lambda row: (
            FAMILY_ORDER.index(row["Safety Family"]),
            row["Benchmark Status"] != "candidate",
            row["Location"],
        )
    )

    fieldnames = [
        "Location",
        "Unsafe Name",
        "Safe Name",
        "Safety Family",
        "Benchmark Status",
        "Benchmark Note",
        "Safe Source",
        "Unsafe Source",
        "Safety Condition",
    ]

    for path in [CLASSIFIED_CSV, CANDIDATES_CSV, EXCLUDED_CSV]:
        path.parent.mkdir(parents=True, exist_ok=True)

    with CLASSIFIED_CSV.open("w", newline="", encoding="utf-8") as f:
        writer = csv.DictWriter(f, fieldnames=fieldnames)
        writer.writeheader()
        writer.writerows(classified_rows)

    with CANDIDATES_CSV.open("w", newline="", encoding="utf-8") as f:
        writer = csv.DictWriter(f, fieldnames=fieldnames)
        writer.writeheader()
        writer.writerows(row for row in classified_rows if row["Benchmark Status"] == "candidate")

    with EXCLUDED_CSV.open("w", newline="", encoding="utf-8") as f:
        writer = csv.DictWriter(f, fieldnames=fieldnames)
        writer.writeheader()
        writer.writerows(row for row in classified_rows if row["Benchmark Status"] == "excluded")

    FAMILIES_DIR.mkdir(parents=True, exist_ok=True)
    by_family: dict[str, list[dict[str, str]]] = defaultdict(list)
    for row in classified_rows:
        by_family[row["Safety Family"]].append(row)
    for family in FAMILY_ORDER:
        family_rows = by_family.get(family, [])
        with (FAMILIES_DIR / output_name(family)).open("w", newline="", encoding="utf-8") as f:
            writer = csv.DictWriter(f, fieldnames=fieldnames)
            writer.writeheader()
            writer.writerows(family_rows)

    family_counts = Counter(row["Safety Family"] for row in classified_rows)
    candidate_counts = Counter(
        row["Safety Family"] for row in classified_rows if row["Benchmark Status"] == "candidate"
    )
    excluded_counts = Counter(
        row["Safety Family"] for row in classified_rows if row["Benchmark Status"] == "excluded"
    )

    lines = [
        "# Public `unchecked` Classification",
        "",
        "Derived from `public_unchecked_pairs.csv`, which was generated directly from the local "
        "`rust/library/{core,alloc,std}/src` tree.",
        "",
        f"- Total entries: `{len(classified_rows)}`",
        f"- Benchmark candidates: `{sum(1 for row in classified_rows if row['Benchmark Status'] == 'candidate')}`",
        f"- Excluded from clean benchmark pairing: `{sum(1 for row in classified_rows if row['Benchmark Status'] == 'excluded')}`",
        "",
        "## Outputs",
        "",
        f"- Classified CSV: `{CLASSIFIED_CSV.relative_to(ROOT).as_posix()}`",
        f"- Candidate CSV: `{CANDIDATES_CSV.relative_to(ROOT).as_posix()}`",
        f"- Excluded CSV: `{EXCLUDED_CSV.relative_to(ROOT).as_posix()}`",
        f"- Family directory: `{FAMILIES_DIR.relative_to(ROOT).as_posix()}`",
        "",
        "## Safety Families",
        "",
    ]

    for family in FAMILY_ORDER:
        lines.append(
            f"- `{family}`: total `{family_counts[family]}`, candidates `{candidate_counts[family]}`, excluded `{excluded_counts[family]}`"
        )

    SUMMARY_MD.write_text("\n".join(lines).rstrip() + "\n", encoding="utf-8")
    print(
        f"Classified {len(classified_rows)} entries: "
        f"{sum(1 for row in classified_rows if row['Benchmark Status'] == 'candidate')} candidates, "
        f"{sum(1 for row in classified_rows if row['Benchmark Status'] == 'excluded')} excluded"
    )


if __name__ == "__main__":
    main()
