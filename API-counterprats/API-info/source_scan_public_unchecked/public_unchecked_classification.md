# Public `unchecked` Classification

Derived from `public_unchecked_pairs.csv`, which was generated directly from the local `rust/library/{core,alloc,std}/src` tree.

- Total entries: `78`
- Benchmark candidates: `63`
- Excluded from clean benchmark pairing: `15`

## Outputs

- Classified CSV: `API-counterprats/API-info/source_scan_public_unchecked/public_unchecked_classified.csv`
- Candidate CSV: `API-counterprats/API-info/source_scan_public_unchecked/public_unchecked_candidates.csv`
- Excluded CSV: `API-counterprats/API-info/source_scan_public_unchecked/public_unchecked_excluded.csv`
- Family directory: `API-counterprats/API-info/source_scan_public_unchecked/public_unchecked_families`

## Safety Families

- `Layout_Initialization_and_Shape`: total `3`, candidates `2`, excluded `1`
- `Value_Validity_and_Representability`: total `14`, candidates `13`, excluded `1`
- `Arithmetic_and_Bit_Preconditions`: total `22`, candidates `21`, excluded `1`
- `Bounds_Length_and_Position`: total `7`, candidates `4`, excluded `3`
- `Encoding_and_Text_Validity`: total `11`, candidates `7`, excluded `4`
- `Aliasing_Pinning_and_Exclusive_Access`: total `8`, candidates `5`, excluded `3`
- `Variant_and_Control_Flow_Assumptions`: total `5`, candidates `3`, excluded `2`
- `Ordering_and_Cursor_Position`: total `8`, candidates `8`, excluded `0`
