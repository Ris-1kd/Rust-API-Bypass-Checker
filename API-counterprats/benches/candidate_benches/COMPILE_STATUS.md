# Candidate Bench Compile Status

Checked on `2026-03-30` in package
[API-counterprats](/home/yunlong/workspace/Bypassing/Rust-API-Bypass/API-counterprats).

Current status:
- `63 / 63` generated Criterion benches compile with `cargo bench --no-run --bench <name>`
- No remaining compile blockers in the generated candidate bench set

What changed:
- The package toolchain was switched from `nightly-2025-01-10` to `nightly` in
  [rust-toolchain.toml](/home/yunlong/workspace/Bypassing/Rust-API-Bypass/API-counterprats/rust-toolchain.toml)
- The placeholder analyzer code in
  [src/lib.rs](/home/yunlong/workspace/Bypassing/Rust-API-Bypass/API-counterprats/src/lib.rs)
  and
  [src/main.rs](/home/yunlong/workspace/Bypassing/Rust-API-Bypass/API-counterprats/src/main.rs)
  was reduced to minimal stubs so bench compilation no longer depends on
  `rustc_private`
- The generated bench headers were updated to enable the required unstable
  features on the current nightly

Validation command:
```bash
cd /home/yunlong/workspace/Bypassing/Rust-API-Bypass/API-counterprats
cargo bench --no-run --bench candidate_core_alloc_layout_from_size_align_unchecked_132
```
