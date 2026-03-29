# Rust `core` / `alloc` / `std` Safe/Unsafe Counterpart Catalog

This document is generated from `unchecked_method_base/base.csv` and cross-checked against the local Rust source tree under `./rust/library` when possible.

Status meaning:
- `confirmed_pair`: both checked and unchecked function implementations were located in the Rust source tree.
- `unsafe_entry_only_or_unclear_pair`: the CSV records an unchecked entry, but a clean checked-side counterpart was not located automatically in the same file.

- Total recorded pairs: `125`
- Confirmed pairs by source lookup: `81`
- Unclear / unsafe-only entries: `44`
- Categories: `22`
- Library/type buckets: `44`

## Summary by Category

| Category | Count |
| --- | ---: |
| `complex` | 53 |
| `index` | 17 |
| `type` | 12 |
| `intrinsic` | 10 |
| `nullptr` | 5 |
| `String` | 4 |
| `buf` | 3 |
| `pin` | 3 |
| `bound/nonzero` | 2 |
| `itron` | 2 |
| `overflow/nonzero` | 2 |
| `Result/Option` | 2 |
| `CString` | 1 |
| `index?` | 1 |
| `nonzero` | 1 |
| `overflow` | 1 |
| `Reuslt/Option` | 1 |
| `slice/buf` | 1 |
| `sync` | 1 |
| `thread` | 1 |
| `type/nonzero` | 1 |
| `vector` | 1 |

## Summary by Library / Dependent Type

| Library / Dependent Type | Count |
| --- | ---: |
| `algorithm` | 14 |
| `slice/string` | 13 |
| `null` | 12 |
| `core::intrinsic` | 8 |
| `num/nonzero` | 7 |
| `ptr` | 6 |
| `any` | 5 |
| `buf` | 4 |
| `simd` | 4 |
| `Option/Result` | 3 |
| `pin` | 3 |
| `slice/String` | 3 |
| `vector` | 3 |
| `ascii` | 2 |
| `cell` | 2 |
| `char` | 2 |
| `core::hint` | 2 |
| `CursorMutKey` | 2 |
| `itron` | 2 |
| `RC` | 2 |
| `str` | 2 |
| `unicode/char` | 2 |
| `alloc` | 1 |
| `any+send` | 1 |
| `Arc/sync` | 1 |
| `array` | 1 |
| `array_iter` | 1 |
| `Box` | 1 |
| `Box+Send` | 1 |
| `Box+send+Sync` | 1 |
| `btreeMap` | 1 |
| `Cstr` | 1 |
| `Cstring` | 1 |
| `CString` | 1 |
| `CursoeMut` | 1 |
| `CursorMut` | 1 |
| `ffi` | 1 |
| `hashmap` | 1 |
| `os_str/buf` | 1 |
| `os_str/slice` | 1 |
| `sync` | 1 |
| `thread` | 1 |
| `Unicode./char` | 1 |
| `uniode/char` | 1 |

## Category: `bound/nonzero`

### bound/nonzero #1: `as_chunks` -> `as_chunks_unchecked`

- Source file: `rust/library/core/src/slice/mod.rs`
- Dependent library / type: `slice/String`
- Counterpart status: `confirmed_pair`
- Safe function lines: `1396`-`1406`
- Unsafe function lines: `1338`-`1349`
- Safety condition: 用户必须保证N不是零并且可以整除
- Checked-side note: Not recorded.
- User visible: Not recorded.
- Flag: Not recorded.

Safe source:
```rust
    pub const fn as_chunks<const N: usize>(&self) -> (&[[T; N]], &[T]) {
        assert!(N != 0, "chunk size must be non-zero");
        let len_rounded_down = self.len() / N * N;
        // SAFETY: The rounded-down value is always the same or smaller than the
        // original length, and thus must be in-bounds of the slice.
        let (multiple_of_n, remainder) = unsafe { self.split_at_unchecked(len_rounded_down) };
        // SAFETY: We already panicked for zero, and ensured by construction
        // that the length of the subslice is a multiple of N.
        let array_slice = unsafe { multiple_of_n.as_chunks_unchecked() };
        (array_slice, remainder)
    }
```

Unsafe source:
```rust
    pub const unsafe fn as_chunks_unchecked<const N: usize>(&self) -> &[[T; N]] {
        assert_unsafe_precondition!(
            check_language_ub,
            "slice::as_chunks_unchecked requires `N != 0` and the slice to split exactly into `N`-element chunks",
            (n: usize = N, len: usize = self.len()) => n != 0 && len.is_multiple_of(n),
        );
        // SAFETY: Caller must guarantee that `N` is nonzero and exactly divides the slice length
        let new_len = unsafe { exact_div(self.len(), N) };
        // SAFETY: We cast a slice of `new_len * N` elements into
        // a slice of `new_len` many `N` elements chunks.
        unsafe { from_raw_parts(self.as_ptr().cast(), new_len) }
    }
```

### bound/nonzero #2: `as_chunks_mut` -> `as_chunks_unchecked_mut`

- Source file: `rust/library/core/src/slice/mod.rs`
- Dependent library / type: `slice/String`
- Counterpart status: `confirmed_pair`
- Safe function lines: `1552`-`1562`
- Unsafe function lines: `1498`-`1509`
- Safety condition: 用户必须保证N不是零并且可以整除
- Checked-side note: Not recorded.
- User visible: Not recorded.
- Flag: Not recorded.

Safe source:
```rust
    pub const fn as_chunks_mut<const N: usize>(&mut self) -> (&mut [[T; N]], &mut [T]) {
        assert!(N != 0, "chunk size must be non-zero");
        let len_rounded_down = self.len() / N * N;
        // SAFETY: The rounded-down value is always the same or smaller than the
        // original length, and thus must be in-bounds of the slice.
        let (multiple_of_n, remainder) = unsafe { self.split_at_mut_unchecked(len_rounded_down) };
        // SAFETY: We already panicked for zero, and ensured by construction
        // that the length of the subslice is a multiple of N.
        let array_slice = unsafe { multiple_of_n.as_chunks_unchecked_mut() };
        (array_slice, remainder)
    }
```

Unsafe source:
```rust
    pub const unsafe fn as_chunks_unchecked_mut<const N: usize>(&mut self) -> &mut [[T; N]] {
        assert_unsafe_precondition!(
            check_language_ub,
            "slice::as_chunks_unchecked requires `N != 0` and the slice to split exactly into `N`-element chunks",
            (n: usize = N, len: usize = self.len()) => n != 0 && len.is_multiple_of(n)
        );
        // SAFETY: Caller must guarantee that `N` is nonzero and exactly divides the slice length
        let new_len = unsafe { exact_div(self.len(), N) };
        // SAFETY: We cast a slice of `new_len * N` elements into
        // a slice of `new_len` many `N` elements chunks.
        unsafe { from_raw_parts_mut(self.as_mut_ptr().cast(), new_len) }
    }
```

## Category: `buf`

### buf #1: `from_encoded_bytes` -> `from_encoded_bytes_unchecked`

- Source file: `rust/library/std/src/sys/os_str/wtf8.rs`
- Dependent library / type: `buf`
- Counterpart status: `unsafe_entry_only_or_unclear_pair`
- Safe function lines: `?`-`?`
- Unsafe function lines: `87`-`89`
- Safety condition: Not recorded.
- Checked-side note: 未找到相关safe
- User visible: Not recorded.
- Flag: Not recorded.

Safe source:
```rust
未找到相关safe
```

Unsafe source:
```rust
    pub unsafe fn from_encoded_bytes_unchecked(s: Vec<u8>) -> Self {
        unsafe { Self { inner: Wtf8Buf::from_bytes_unchecked(s) } }
    }
```

### buf #2: `from_encoded_bytes` -> `from_encoded_bytes_unchecked`

- Source file: `rust/library/std/src/sys/os_str/wtf8.rs`
- Dependent library / type: `buf`
- Counterpart status: `unsafe_entry_only_or_unclear_pair`
- Safe function lines: `?`-`?`
- Unsafe function lines: `87`-`89`
- Safety condition: Not recorded.
- Checked-side note: 未找到相关safe
- User visible: Not recorded.
- Flag: Not recorded.

Safe source:
```rust
未找到相关safe
```

Unsafe source:
```rust
    pub unsafe fn from_encoded_bytes_unchecked(s: Vec<u8>) -> Self {
        unsafe { Self { inner: Wtf8Buf::from_bytes_unchecked(s) } }
    }
```

### buf #3: `from_encoded_bytes` -> `from_encoded_bytes_unchecked`

- Source file: `rust/library/std/src/sys/os_str/bytes.rs`
- Dependent library / type: `os_str/buf`
- Counterpart status: `unsafe_entry_only_or_unclear_pair`
- Safe function lines: `?`-`?`
- Unsafe function lines: `90`-`92`
- Safety condition: Not recorded.
- Checked-side note: 未找到相关safe
- User visible: Not recorded.
- Flag: Not recorded.

Safe source:
```rust
未找到相关safe
```

Unsafe source:
```rust
    pub unsafe fn from_encoded_bytes_unchecked(s: Vec<u8>) -> Self {
        Self { inner: s }
    }
```

## Category: `complex`

### complex #1: `checked_mul` -> `unchecked_mul`

- Source file: `rust/library/core/src/num/int_macros.rs`
- Dependent library / type: `algorithm`
- Counterpart status: `confirmed_pair`
- Safe function lines: `803`-`806`
- Unsafe function lines: `863`-`877`
- Safety condition: Not recorded.
- Checked-side note: Not recorded.
- User visible: Not recorded.
- Flag: Not recorded.

Safe source:
```rust
        pub const fn checked_mul(self, rhs: Self) -> Option<Self> {
            let (a, b) = self.overflowing_mul(rhs);
            if intrinsics::unlikely(b) { None } else { Some(a) }
        }
```

Unsafe source:
```rust
        pub const unsafe fn unchecked_mul(self, rhs: Self) -> Self {
            assert_unsafe_precondition!(
                check_language_ub,
                concat!(stringify!($SelfT), "::unchecked_mul cannot overflow"),
                (
                    lhs: $SelfT = self,
                    rhs: $SelfT = rhs,
                ) => !lhs.overflowing_mul(rhs).1,
            );

            // SAFETY: this is guaranteed to be safe by the caller.
            unsafe {
                intrinsics::unchecked_mul(self, rhs)
            }
        }
```

### complex #2: `checked_neg` -> `unchecked_neg`

- Source file: `rust/library/core/src/num/int_macros.rs`
- Dependent library / type: `algorithm`
- Counterpart status: `confirmed_pair`
- Safe function lines: `1264`-`1267`
- Unsafe function lines: `1284`-`1297`
- Safety condition: Not recorded.
- Checked-side note: Not recorded.
- User visible: Not recorded.
- Flag: Not recorded.

Safe source:
```rust
        pub const fn checked_neg(self) -> Option<Self> {
            let (a, b) = self.overflowing_neg();
            if intrinsics::unlikely(b) { None } else { Some(a) }
        }
```

Unsafe source:
```rust
        pub const unsafe fn unchecked_neg(self) -> Self {
            assert_unsafe_precondition!(
                check_language_ub,
                concat!(stringify!($SelfT), "::unchecked_neg cannot overflow"),
                (
                    lhs: $SelfT = self,
                ) => !lhs.overflowing_neg().1,
            );

            // SAFETY: this is guaranteed to be safe by the caller.
            unsafe {
                intrinsics::unchecked_sub(0, self)
            }
        }
```

### complex #3: `checked_shl` -> `unchecked_shl`

- Source file: `rust/library/core/src/num/int_macros.rs`
- Dependent library / type: `algorithm`
- Counterpart status: `confirmed_pair`
- Safe function lines: `1344`-`1352`
- Unsafe function lines: `1401`-`1414`
- Safety condition: Not recorded.
- Checked-side note: Not recorded.
- User visible: Not recorded.
- Flag: Not recorded.

Safe source:
```rust
        pub const fn checked_shl(self, rhs: u32) -> Option<Self> {
            // Not using overflowing_shl as that's a wrapping shift
            if rhs < Self::BITS {
                // SAFETY: just checked the RHS is in-range
                Some(unsafe { self.unchecked_shl(rhs) })
            } else {
                None
            }
        }
```

Unsafe source:
```rust
        pub const unsafe fn unchecked_shl(self, rhs: u32) -> Self {
            assert_unsafe_precondition!(
                check_language_ub,
                concat!(stringify!($SelfT), "::unchecked_shl cannot overflow"),
                (
                    rhs: u32 = rhs,
                ) => rhs < <$ActualT>::BITS,
            );

            // SAFETY: this is guaranteed to be safe by the caller.
            unsafe {
                intrinsics::unchecked_shl(self, rhs)
            }
        }
```

### complex #4: `checked_shr` -> `unchecked_shr`

- Source file: `rust/library/core/src/num/int_macros.rs`
- Dependent library / type: `algorithm`
- Counterpart status: `confirmed_pair`
- Safe function lines: `1523`-`1531`
- Unsafe function lines: `1580`-`1593`
- Safety condition: Not recorded.
- Checked-side note: Not recorded.
- User visible: Not recorded.
- Flag: Not recorded.

Safe source:
```rust
        pub const fn checked_shr(self, rhs: u32) -> Option<Self> {
            // Not using overflowing_shr as that's a wrapping shift
            if rhs < Self::BITS {
                // SAFETY: just checked the RHS is in-range
                Some(unsafe { self.unchecked_shr(rhs) })
            } else {
                None
            }
        }
```

Unsafe source:
```rust
        pub const unsafe fn unchecked_shr(self, rhs: u32) -> Self {
            assert_unsafe_precondition!(
                check_language_ub,
                concat!(stringify!($SelfT), "::unchecked_shr cannot overflow"),
                (
                    rhs: u32 = rhs,
                ) => rhs < <$ActualT>::BITS,
            );

            // SAFETY: this is guaranteed to be safe by the caller.
            unsafe {
                intrinsics::unchecked_shr(self, rhs)
            }
        }
```

### complex #5: `checked_sub` -> `unchecked_sub`

- Source file: `rust/library/core/src/num/int_macros.rs`
- Dependent library / type: `algorithm`
- Counterpart status: `confirmed_pair`
- Safe function lines: `663`-`666`
- Unsafe function lines: `723`-`737`
- Safety condition: Not recorded.
- Checked-side note: Not recorded.
- User visible: Not recorded.
- Flag: Not recorded.

Safe source:
```rust
        pub const fn checked_sub(self, rhs: Self) -> Option<Self> {
            let (a, b) = self.overflowing_sub(rhs);
            if intrinsics::unlikely(b) { None } else { Some(a) }
        }
```

Unsafe source:
```rust
        pub const unsafe fn unchecked_sub(self, rhs: Self) -> Self {
            assert_unsafe_precondition!(
                check_language_ub,
                concat!(stringify!($SelfT), "::unchecked_sub cannot overflow"),
                (
                    lhs: $SelfT = self,
                    rhs: $SelfT = rhs,
                ) => !lhs.overflowing_sub(rhs).1,
            );

            // SAFETY: this is guaranteed to be safe by the caller.
            unsafe {
                intrinsics::unchecked_sub(self, rhs)
            }
        }
```

### complex #6: `checked_add` -> `unchecked_add`

- Source file: `rust/library/core/src/num/uint_macros.rs`
- Dependent library / type: `algorithm`
- Counterpart status: `confirmed_pair`
- Safe function lines: `815`-`829`
- Unsafe function lines: `886`-`900`
- Safety condition: Not recorded.
- Checked-side note: 不重复
- User visible: Not recorded.
- Flag: Not recorded.

Safe source:
```rust
        pub const fn checked_add(self, rhs: Self) -> Option<Self> {
            // This used to use `overflowing_add`, but that means it ends up being
            // a `wrapping_add`, losing some optimization opportunities. Notably,
            // phrasing it this way helps `.checked_add(1)` optimize to a check
            // against `MAX` and a `add nuw`.
            // Per <https://github.com/rust-lang/rust/pull/124114#issuecomment-2066173305>,
            // LLVM is happy to re-form the intrinsic later if useful.

            if intrinsics::unlikely(intrinsics::add_with_overflow(self, rhs).1) {
                None
            } else {
                // SAFETY: Just checked it doesn't overflow
                Some(unsafe { intrinsics::unchecked_add(self, rhs) })
            }
        }
```

Unsafe source:
```rust
        pub const unsafe fn unchecked_add(self, rhs: Self) -> Self {
            assert_unsafe_precondition!(
                check_language_ub,
                concat!(stringify!($SelfT), "::unchecked_add cannot overflow"),
                (
                    lhs: $SelfT = self,
                    rhs: $SelfT = rhs,
                ) => !lhs.overflowing_add(rhs).1,
            );

            // SAFETY: this is guaranteed to be safe by the caller.
            unsafe {
                intrinsics::unchecked_add(self, rhs)
            }
        }
```

### complex #7: `checked_disjoint_bitor` -> `unchecked_disjoint_bitor`

- Source file: `rust/library/core/src/num/uint_macros.rs`
- Dependent library / type: `algorithm`
- Counterpart status: `unsafe_entry_only_or_unclear_pair`
- Safe function lines: `?`-`?`
- Unsafe function lines: `1631`-`1643`
- Safety condition: Not recorded.
- Checked-side note: 不重复
- User visible: Not recorded.
- Flag: Not recorded.

Safe source:
```rust
不重复
```

Unsafe source:
```rust
        pub const unsafe fn unchecked_disjoint_bitor(self, other: Self) -> Self {
            assert_unsafe_precondition!(
                check_language_ub,
                concat!(stringify!($SelfT), "::unchecked_disjoint_bitor cannot have overlapping bits"),
                (
                    lhs: $SelfT = self,
                    rhs: $SelfT = other,
                ) => (lhs & rhs) == 0,
            );

            // SAFETY: Same precondition
            unsafe { intrinsics::disjoint_bitor(self, other) }
        }
```

### complex #8: `checked_mul` -> `unchecked_mul`

- Source file: `rust/library/core/src/num/uint_macros.rs`
- Dependent library / type: `algorithm`
- Counterpart status: `confirmed_pair`
- Safe function lines: `1214`-`1217`
- Unsafe function lines: `1274`-`1288`
- Safety condition: Not recorded.
- Checked-side note: 不重复
- User visible: Not recorded.
- Flag: Not recorded.

Safe source:
```rust
        pub const fn checked_mul(self, rhs: Self) -> Option<Self> {
            let (a, b) = self.overflowing_mul(rhs);
            if intrinsics::unlikely(b) { None } else { Some(a) }
        }
```

Unsafe source:
```rust
        pub const unsafe fn unchecked_mul(self, rhs: Self) -> Self {
            assert_unsafe_precondition!(
                check_language_ub,
                concat!(stringify!($SelfT), "::unchecked_mul cannot overflow"),
                (
                    lhs: $SelfT = self,
                    rhs: $SelfT = rhs,
                ) => !lhs.overflowing_mul(rhs).1,
            );

            // SAFETY: this is guaranteed to be safe by the caller.
            unsafe {
                intrinsics::unchecked_mul(self, rhs)
            }
        }
```

### complex #9: `checked_shl` -> `unchecked_shl`

- Source file: `rust/library/core/src/num/uint_macros.rs`
- Dependent library / type: `algorithm`
- Counterpart status: `confirmed_pair`
- Safe function lines: `1902`-`1910`
- Unsafe function lines: `1959`-`1972`
- Safety condition: Not recorded.
- Checked-side note: 不重复
- User visible: Not recorded.
- Flag: Not recorded.

Safe source:
```rust
        pub const fn checked_shl(self, rhs: u32) -> Option<Self> {
            // Not using overflowing_shl as that's a wrapping shift
            if rhs < Self::BITS {
                // SAFETY: just checked the RHS is in-range
                Some(unsafe { self.unchecked_shl(rhs) })
            } else {
                None
            }
        }
```

Unsafe source:
```rust
        pub const unsafe fn unchecked_shl(self, rhs: u32) -> Self {
            assert_unsafe_precondition!(
                check_language_ub,
                concat!(stringify!($SelfT), "::unchecked_shl cannot overflow"),
                (
                    rhs: u32 = rhs,
                ) => rhs < <$ActualT>::BITS,
            );

            // SAFETY: this is guaranteed to be safe by the caller.
            unsafe {
                intrinsics::unchecked_shl(self, rhs)
            }
        }
```

### complex #10: `checked_shr` -> `unchecked_shr`

- Source file: `rust/library/core/src/num/uint_macros.rs`
- Dependent library / type: `algorithm`
- Counterpart status: `confirmed_pair`
- Safe function lines: `2087`-`2095`
- Unsafe function lines: `2144`-`2157`
- Safety condition: Not recorded.
- Checked-side note: 不重复
- User visible: Not recorded.
- Flag: Not recorded.

Safe source:
```rust
        pub const fn checked_shr(self, rhs: u32) -> Option<Self> {
            // Not using overflowing_shr as that's a wrapping shift
            if rhs < Self::BITS {
                // SAFETY: just checked the RHS is in-range
                Some(unsafe { self.unchecked_shr(rhs) })
            } else {
                None
            }
        }
```

Unsafe source:
```rust
        pub const unsafe fn unchecked_shr(self, rhs: u32) -> Self {
            assert_unsafe_precondition!(
                check_language_ub,
                concat!(stringify!($SelfT), "::unchecked_shr cannot overflow"),
                (
                    rhs: u32 = rhs,
                ) => rhs < <$ActualT>::BITS,
            );

            // SAFETY: this is guaranteed to be safe by the caller.
            unsafe {
                intrinsics::unchecked_shr(self, rhs)
            }
        }
```

### complex #11: `checked_sub` -> `unchecked_sub`

- Source file: `rust/library/core/src/num/uint_macros.rs`
- Dependent library / type: `algorithm`
- Counterpart status: `confirmed_pair`
- Safe function lines: `971`-`983`
- Unsafe function lines: `1065`-`1079`
- Safety condition: Not recorded.
- Checked-side note: 不重复
- User visible: Not recorded.
- Flag: Not recorded.

Safe source:
```rust
        pub const fn checked_sub(self, rhs: Self) -> Option<Self> {
            // Per PR#103299, there's no advantage to the `overflowing` intrinsic
            // for *unsigned* subtraction and we just emit the manual check anyway.
            // Thus, rather than using `overflowing_sub` that produces a wrapping
            // subtraction, check it ourself so we can use an unchecked one.

            if self < rhs {
                None
            } else {
                // SAFETY: just checked this can't overflow
                Some(unsafe { intrinsics::unchecked_sub(self, rhs) })
            }
        }
```

Unsafe source:
```rust
        pub const unsafe fn unchecked_sub(self, rhs: Self) -> Self {
            assert_unsafe_precondition!(
                check_language_ub,
                concat!(stringify!($SelfT), "::unchecked_sub cannot overflow"),
                (
                    lhs: $SelfT = self,
                    rhs: $SelfT = rhs,
                ) => !lhs.overflowing_sub(rhs).1,
            );

            // SAFETY: this is guaranteed to be safe by the caller.
            unsafe {
                intrinsics::unchecked_sub(self, rhs)
            }
        }
```

### complex #12: `new` -> `new_unchecked`

- Source file: `rust/library/core/src/array/iter.rs`
- Dependent library / type: `array_iter`
- Counterpart status: `confirmed_pair`
- Safe function lines: `80`-`82`
- Unsafe function lines: `141`-`150`
- Safety condition: 需要确定参数的范围是规范的
- Checked-side note: 无直接对应安全函数
- User visible: Not recorded.
- Flag: Not recorded.

Safe source:
```rust
    pub fn new(array: [T; N]) -> Self {
        IntoIterator::into_iter(array)
    }
```

Unsafe source:
```rust
    pub const unsafe fn new_unchecked(
        buffer: [MaybeUninit<T>; N],
        initialized: Range<usize>,
    ) -> Self {
        // SAFETY: one of our safety conditions is that the range is canonical.
        let alive = unsafe { IndexRange::new_unchecked(initialized.start, initialized.end) };
        // SAFETY: one of our safety condition is that these items are initialized.
        let inner = unsafe { InnerSized::new_unchecked(alive, buffer) };
        IntoIter { inner: ManuallyDrop::new(inner) }
    }
```

### complex #13: `insert_before` -> `insert_before_unchecked`

- Source file: `rust/library/alloc/src/collections/btree/map.rs`
- Dependent library / type: `btreeMap`
- Counterpart status: `confirmed_pair`
- Safe function lines: `3532`-`3547`
- Unsafe function lines: `3461`-`3491`
- Safety condition: Not recorded.
- Checked-side note: Not recorded.
- User visible: Not recorded.
- Flag: Not recorded.

Safe source:
```rust
    pub fn insert_before(&mut self, key: K, value: V) -> Result<(), UnorderedKeyError> {
        if let Some((prev, _)) = self.peek_prev() {
            if &key <= prev {
                return Err(UnorderedKeyError {});
            }
        }
        if let Some((next, _)) = self.peek_next() {
            if &key >= next {
                return Err(UnorderedKeyError {});
            }
        }
        unsafe {
            self.insert_before_unchecked(key, value);
        }
        Ok(())
    }
```

Unsafe source:
```rust
    pub unsafe fn insert_before_unchecked(&mut self, key: K, value: V) {
        let edge = match self.current.take() {
            None => {
                // SAFETY: We have no other reference to the tree.
                match unsafe { self.root.reborrow() } {
                    root @ None => {
                        // Tree is empty, allocate a new root.
                        let mut node = NodeRef::new_leaf(self.alloc.clone());
                        // SAFETY: We don't touch the root while the handle is alive.
                        let handle = unsafe { node.borrow_mut().push_with_handle(key, value) };
                        *root = Some(node.forget_type());
                        *self.length += 1;
                        self.current = Some(handle.right_edge());
                        return;
                    }
                    Some(root) => root.borrow_mut().last_leaf_edge(),
                }
            }
            Some(current) => current,
        };

        let handle = edge.insert_recursing(key, value, self.alloc.clone(), |ins| {
            drop(ins.left);
            // SAFETY: The handle to the newly inserted value is always on a
            // leaf node, so adding a new root node doesn't invalidate it.
            let root = unsafe { self.root.reborrow().as_mut().unwrap() };
            root.push_internal_level(self.alloc.clone()).push(ins.kv.0, ins.kv.1, ins.right)
        });
        self.current = Some(handle.right_edge());
        *self.length += 1;
    }
```

### complex #14: `advance` -> `advance_unchecked`

- Source file: `rust/library/core/src/io/borrowed_buf.rs`
- Dependent library / type: `buf`
- Counterpart status: `confirmed_pair`
- Safe function lines: `275`-`281`
- Unsafe function lines: `294`-`298`
- Safety condition: 如果没有n个初始化, 则panic
- Checked-side note: Not recorded.
- User visible: Not recorded.
- Flag: Not recorded.

Safe source:
```rust
    pub fn advance(&mut self, n: usize) -> &mut Self {
        // The subtraction cannot underflow by invariant of this type.
        assert!(n <= self.buf.init - self.buf.filled);

        self.buf.filled += n;
        self
    }
```

Unsafe source:
```rust
    pub unsafe fn advance_unchecked(&mut self, n: usize) -> &mut Self {
        self.buf.filled += n;
        self.buf.init = cmp::max(self.buf.init, self.buf.filled);
        self
    }
```

### complex #15: `from_encoded_bytes` -> `from_encoded_bytes_unchecked`

- Source file: `rust/library/std/src/ffi/os_str.rs`
- Dependent library / type: `buf`
- Counterpart status: `unsafe_entry_only_or_unclear_pair`
- Safe function lines: `?`-`?`
- Unsafe function lines: `184`-`186`
- Safety condition: Not recorded.
- Checked-side note: 未找到相关safe
- User visible: Not recorded.
- Flag: Not recorded.

Safe source:
```rust
未找到相关safe
```

Unsafe source:
```rust
    pub unsafe fn from_encoded_bytes_unchecked(bytes: Vec<u8>) -> Self {
        OsString { inner: unsafe { Buf::from_encoded_bytes_unchecked(bytes) } }
    }
```

### complex #16: `as_mut` -> `as_mut_unchecked`

- Source file: `rust/library/core/src/cell.rs`
- Dependent library / type: `cell`
- Counterpart status: `unsafe_entry_only_or_unclear_pair`
- Safe function lines: `?`-`?`
- Unsafe function lines: `2562`-`2565`
- Safety condition: null
- Checked-side note: 无直接对应的unsafe版本, 与RefMut有关
- User visible: Not recorded.
- Flag: Not recorded.

Safe source:
```rust
无直接对应的unsafe版本, 与RefMut有关
```

Unsafe source:
```rust
    pub const unsafe fn as_mut_unchecked(&self) -> &mut T {
        // SAFETY: pointer comes from `&self` so naturally satisfies ptr-to-ref invariants.
        unsafe { self.get().as_mut_unchecked() }
    }
```

### complex #17: `as_ref` -> `as_ref_unchecked`

- Source file: `rust/library/core/src/cell.rs`
- Dependent library / type: `cell`
- Counterpart status: `confirmed_pair`
- Safe function lines: `695`-`697`
- Unsafe function lines: `2533`-`2536`
- Safety condition: null
- Checked-side note: 无直接对应的unsafe版本, 与RefCell有关
- User visible: Not recorded.
- Flag: Not recorded.

Safe source:
```rust
    fn as_ref(&self) -> &[Cell<T>; N] {
        self.as_array_of_cells()
    }
```

Unsafe source:
```rust
    pub const unsafe fn as_ref_unchecked(&self) -> &T {
        // SAFETY: pointer comes from `&self` so naturally satisfies ptr-to-ref invariants.
        unsafe { self.get().as_ref_unchecked() }
    }
```

### complex #18: `from_vec` -> `from_vec_unchecked`

- Source file: `rust/library/alloc/src/ffi/c_str.rs`
- Dependent library / type: `Cstring`
- Counterpart status: `unsafe_entry_only_or_unclear_pair`
- Safe function lines: `?`-`?`
- Unsafe function lines: `340`-`343`
- Safety condition: Not recorded.
- Checked-side note: 未找到相关safe
- User visible: Not recorded.
- Flag: Not recorded.

Safe source:
```rust
未找到相关safe
```

Unsafe source:
```rust
    pub unsafe fn from_vec_unchecked(v: Vec<u8>) -> Self {
        debug_assert!(memchr::memchr(0, &v).is_none());
        unsafe { Self::_from_vec_unchecked(v) }
    }
```

### complex #19: `from_vec_with_nul` -> `from_vec_with_nul_unchecked`

- Source file: `rust/library/alloc/src/ffi/c_str.rs`
- Dependent library / type: `CString`
- Counterpart status: `confirmed_pair`
- Safe function lines: `678`-`695`
- Unsafe function lines: `635`-`638`
- Safety condition: Not recorded.
- Checked-side note: Not recorded.
- User visible: Not recorded.
- Flag: Not recorded.

Safe source:
```rust
    pub fn from_vec_with_nul(v: Vec<u8>) -> Result<Self, FromVecWithNulError> {
        let nul_pos = memchr::memchr(0, &v);
        match nul_pos {
            Some(nul_pos) if nul_pos + 1 == v.len() => {
                // SAFETY: We know there is only one nul byte, at the end
                // of the vec.
                Ok(unsafe { Self::_from_vec_with_nul_unchecked(v) })
            }
            Some(nul_pos) => Err(FromVecWithNulError {
                error_kind: FromBytesWithNulErrorKind::InteriorNul(nul_pos),
                bytes: v,
            }),
            None => Err(FromVecWithNulError {
                error_kind: FromBytesWithNulErrorKind::NotNulTerminated,
                bytes: v,
            }),
        }
    }
```

Unsafe source:
```rust
    pub unsafe fn from_vec_with_nul_unchecked(v: Vec<u8>) -> Self {
        debug_assert!(memchr::memchr(0, &v).unwrap() + 1 == v.len());
        unsafe { Self::_from_vec_with_nul_unchecked(v) }
    }
```

### complex #20: `insert_after` -> `insert_after_unchecked`

- Source file: `rust/library/alloc/src/collections/btree/set.rs`
- Dependent library / type: `CursoeMut`
- Counterpart status: `confirmed_pair`
- Safe function lines: `2404`-`2406`
- Unsafe function lines: `2371`-`2373`
- Safety condition: BTreeSet不变量必须被保持
- Checked-side note: Not recorded.
- User visible: Not recorded.
- Flag: Not recorded.

Safe source:
```rust
    pub fn insert_after(&mut self, value: T) -> Result<(), UnorderedKeyError> {
        self.inner.insert_after(value, SetValZST)
    }
```

Unsafe source:
```rust
    pub unsafe fn insert_after_unchecked(&mut self, value: T) {
        unsafe { self.inner.insert_after_unchecked(value, SetValZST) }
    }
```

### complex #21: `insert_after` -> `insert_after_unchecked`

- Source file: `rust/library/alloc/src/collections/btree/map.rs`
- Dependent library / type: `CursorMut`
- Counterpart status: `confirmed_pair`
- Safe function lines: `3504`-`3519`
- Unsafe function lines: `3418`-`3445`
- Safety condition: Not recorded.
- Checked-side note: Not recorded.
- User visible: Not recorded.
- Flag: Not recorded.

Safe source:
```rust
    pub fn insert_after(&mut self, key: K, value: V) -> Result<(), UnorderedKeyError> {
        if let Some((prev, _)) = self.peek_prev() {
            if &key <= prev {
                return Err(UnorderedKeyError {});
            }
        }
        if let Some((next, _)) = self.peek_next() {
            if &key >= next {
                return Err(UnorderedKeyError {});
            }
        }
        unsafe {
            self.insert_after_unchecked(key, value);
        }
        Ok(())
    }
```

Unsafe source:
```rust
    pub unsafe fn insert_after_unchecked(&mut self, key: K, value: V) {
        let edge = match self.current.take() {
            None => {
                // Tree is empty, allocate a new root.
                // SAFETY: We have no other reference to the tree.
                let root = unsafe { self.root.reborrow() };
                debug_assert!(root.is_none());
                let mut node = NodeRef::new_leaf(self.alloc.clone());
                // SAFETY: We don't touch the root while the handle is alive.
                let handle = unsafe { node.borrow_mut().push_with_handle(key, value) };
                *root = Some(node.forget_type());
                *self.length += 1;
                self.current = Some(handle.left_edge());
                return;
            }
            Some(current) => current,
        };

        let handle = edge.insert_recursing(key, value, self.alloc.clone(), |ins| {
            drop(ins.left);
            // SAFETY: The handle to the newly inserted value is always on a
            // leaf node, so adding a new root node doesn't invalidate it.
            let root = unsafe { self.root.reborrow().as_mut().unwrap() };
            root.push_internal_level(self.alloc.clone()).push(ins.kv.0, ins.kv.1, ins.right)
        });
        self.current = Some(handle.left_edge());
        *self.length += 1;
    }
```

### complex #22: `insert_after` -> `insert_after_unchecked`

- Source file: `rust/library/alloc/src/collections/btree/map.rs`
- Dependent library / type: `CursorMutKey`
- Counterpart status: `confirmed_pair`
- Safe function lines: `3504`-`3519`
- Unsafe function lines: `3418`-`3445`
- Safety condition: 必须保证BTreeMap不变量被包含
新插入的元素必须得是书中独特的元素
树中所有的键必须以排序顺序保存
- Checked-side note: Not recorded.
- User visible: Not recorded.
- Flag: Not recorded.

Safe source:
```rust
    pub fn insert_after(&mut self, key: K, value: V) -> Result<(), UnorderedKeyError> {
        if let Some((prev, _)) = self.peek_prev() {
            if &key <= prev {
                return Err(UnorderedKeyError {});
            }
        }
        if let Some((next, _)) = self.peek_next() {
            if &key >= next {
                return Err(UnorderedKeyError {});
            }
        }
        unsafe {
            self.insert_after_unchecked(key, value);
        }
        Ok(())
    }
```

Unsafe source:
```rust
    pub unsafe fn insert_after_unchecked(&mut self, key: K, value: V) {
        let edge = match self.current.take() {
            None => {
                // Tree is empty, allocate a new root.
                // SAFETY: We have no other reference to the tree.
                let root = unsafe { self.root.reborrow() };
                debug_assert!(root.is_none());
                let mut node = NodeRef::new_leaf(self.alloc.clone());
                // SAFETY: We don't touch the root while the handle is alive.
                let handle = unsafe { node.borrow_mut().push_with_handle(key, value) };
                *root = Some(node.forget_type());
                *self.length += 1;
                self.current = Some(handle.left_edge());
                return;
            }
            Some(current) => current,
        };

        let handle = edge.insert_recursing(key, value, self.alloc.clone(), |ins| {
            drop(ins.left);
            // SAFETY: The handle to the newly inserted value is always on a
            // leaf node, so adding a new root node doesn't invalidate it.
            let root = unsafe { self.root.reborrow().as_mut().unwrap() };
            root.push_internal_level(self.alloc.clone()).push(ins.kv.0, ins.kv.1, ins.right)
        });
        self.current = Some(handle.left_edge());
        *self.length += 1;
    }
```

### complex #23: `insert_before` -> `insert_before_unchecked`

- Source file: `rust/library/alloc/src/collections/btree/map.rs`
- Dependent library / type: `CursorMutKey`
- Counterpart status: `confirmed_pair`
- Safe function lines: `3532`-`3547`
- Unsafe function lines: `3461`-`3491`
- Safety condition: 必须保证BTreeMap不变量被包含
新插入的元素必须得是书中独特的元素
树中所有的键必须以排序顺序保存
- Checked-side note: Not recorded.
- User visible: Not recorded.
- Flag: Not recorded.

Safe source:
```rust
    pub fn insert_before(&mut self, key: K, value: V) -> Result<(), UnorderedKeyError> {
        if let Some((prev, _)) = self.peek_prev() {
            if &key <= prev {
                return Err(UnorderedKeyError {});
            }
        }
        if let Some((next, _)) = self.peek_next() {
            if &key >= next {
                return Err(UnorderedKeyError {});
            }
        }
        unsafe {
            self.insert_before_unchecked(key, value);
        }
        Ok(())
    }
```

Unsafe source:
```rust
    pub unsafe fn insert_before_unchecked(&mut self, key: K, value: V) {
        let edge = match self.current.take() {
            None => {
                // SAFETY: We have no other reference to the tree.
                match unsafe { self.root.reborrow() } {
                    root @ None => {
                        // Tree is empty, allocate a new root.
                        let mut node = NodeRef::new_leaf(self.alloc.clone());
                        // SAFETY: We don't touch the root while the handle is alive.
                        let handle = unsafe { node.borrow_mut().push_with_handle(key, value) };
                        *root = Some(node.forget_type());
                        *self.length += 1;
                        self.current = Some(handle.right_edge());
                        return;
                    }
                    Some(root) => root.borrow_mut().last_leaf_edge(),
                }
            }
            Some(current) => current,
        };

        let handle = edge.insert_recursing(key, value, self.alloc.clone(), |ins| {
            drop(ins.left);
            // SAFETY: The handle to the newly inserted value is always on a
            // leaf node, so adding a new root node doesn't invalidate it.
            let root = unsafe { self.root.reborrow().as_mut().unwrap() };
            root.push_internal_level(self.alloc.clone()).push(ins.kv.0, ins.kv.1, ins.right)
        });
        self.current = Some(handle.right_edge());
        *self.length += 1;
    }
```

### complex #24: `from_encoded_bytes` -> `from_encoded_bytes_unchecked`

- Source file: `rust/library/std/src/ffi/os_str.rs`
- Dependent library / type: `ffi`
- Counterpart status: `unsafe_entry_only_or_unclear_pair`
- Safe function lines: `?`-`?`
- Unsafe function lines: `184`-`186`
- Safety condition: Not recorded.
- Checked-side note: 未找到相关safe
- User visible: Not recorded.
- Flag: Not recorded.

Safe source:
```rust
未找到相关safe
```

Unsafe source:
```rust
    pub unsafe fn from_encoded_bytes_unchecked(bytes: Vec<u8>) -> Self {
        OsString { inner: unsafe { Buf::from_encoded_bytes_unchecked(bytes) } }
    }
```

### complex #25: `get_disjoint_mut` -> `get_disjoint_unchecked_mut`

- Source file: `rust/library/std/src/collections/hash/map.rs`
- Dependent library / type: `hashmap`
- Counterpart status: `confirmed_pair`
- Safe function lines: `1138`-`1147`
- Unsafe function lines: `1205`-`1214`
- Safety condition: Not recorded.
- Checked-side note: Not recorded.
- User visible: Not recorded.
- Flag: Not recorded.

Safe source:
```rust
    pub fn get_disjoint_mut<Q: ?Sized, const N: usize>(
        &mut self,
        ks: [&Q; N],
    ) -> [Option<&'_ mut V>; N]
    where
        K: Borrow<Q>,
        Q: Hash + Eq,
    {
        self.base.get_disjoint_mut(ks)
    }
```

Unsafe source:
```rust
    pub unsafe fn get_disjoint_unchecked_mut<Q: ?Sized, const N: usize>(
        &mut self,
        ks: [&Q; N],
    ) -> [Option<&'_ mut V>; N]
    where
        K: Borrow<Q>,
        Q: Hash + Eq,
    {
        unsafe { self.base.get_disjoint_unchecked_mut(ks) }
    }
```

### complex #26: `insert_after` -> `insert_after_unchecked`

- Source file: `rust/library/alloc/src/collections/btree/set.rs`
- Dependent library / type: `null`
- Counterpart status: `confirmed_pair`
- Safe function lines: `2404`-`2406`
- Unsafe function lines: `2371`-`2373`
- Safety condition: 同上
- Checked-side note: 同上
- User visible: Not recorded.
- Flag: Not recorded.

Safe source:
```rust
    pub fn insert_after(&mut self, value: T) -> Result<(), UnorderedKeyError> {
        self.inner.insert_after(value, SetValZST)
    }
```

Unsafe source:
```rust
    pub unsafe fn insert_after_unchecked(&mut self, value: T) {
        unsafe { self.inner.insert_after_unchecked(value, SetValZST) }
    }
```

### complex #27: `insert_before` -> `insert_before_unchecked`

- Source file: `rust/library/alloc/src/collections/btree/set.rs`
- Dependent library / type: `null`
- Counterpart status: `confirmed_pair`
- Safe function lines: `2419`-`2421`
- Unsafe function lines: `2389`-`2391`
- Safety condition: 同上
- Checked-side note: 同上
- User visible: Not recorded.
- Flag: Not recorded.

Safe source:
```rust
    pub fn insert_before(&mut self, value: T) -> Result<(), UnorderedKeyError> {
        self.inner.insert_before(value, SetValZST)
    }
```

Unsafe source:
```rust
    pub unsafe fn insert_before_unchecked(&mut self, value: T) {
        unsafe { self.inner.insert_before_unchecked(value, SetValZST) }
    }
```

### complex #28: `insert_before` -> `insert_before_unchecked`

- Source file: `rust/library/alloc/src/collections/btree/set.rs`
- Dependent library / type: `null`
- Counterpart status: `confirmed_pair`
- Safe function lines: `2419`-`2421`
- Unsafe function lines: `2389`-`2391`
- Safety condition: 同上
- Checked-side note: 同上
- User visible: Not recorded.
- Flag: Not recorded.

Safe source:
```rust
    pub fn insert_before(&mut self, value: T) -> Result<(), UnorderedKeyError> {
        self.inner.insert_before(value, SetValZST)
    }
```

Unsafe source:
```rust
    pub unsafe fn insert_before_unchecked(&mut self, value: T) {
        unsafe { self.inner.insert_before_unchecked(value, SetValZST) }
    }
```

### complex #29: `get_mut` -> `get_mut_unchecked`

- Source file: `rust/library/alloc/src/sync.rs`
- Dependent library / type: `null`
- Counterpart status: `confirmed_pair`
- Safe function lines: `2613`-`2624`
- Unsafe function lines: `2688`-`2692`
- Safety condition: null
- Checked-side note: Not recorded.
- User visible: Not recorded.
- Flag: Not recorded.

Safe source:
```rust
    pub fn get_mut(this: &mut Self) -> Option<&mut T> {
        if Self::is_unique(this) {
            // This unsafety is ok because we're guaranteed that the pointer
            // returned is the *only* pointer that will ever be returned to T. Our
            // reference count is guaranteed to be 1 at this point, and we required
            // the Arc itself to be `mut`, so we're returning the only possible
            // reference to the inner data.
            unsafe { Some(Arc::get_mut_unchecked(this)) }
        } else {
            None
        }
    }
```

Unsafe source:
```rust
    pub unsafe fn get_mut_unchecked(this: &mut Self) -> &mut T {
        // We are careful to *not* create a reference covering the "count" fields, as
        // this would alias with concurrent access to the reference counts (e.g. by `Weak`).
        unsafe { &mut (*this.ptr.as_ptr()).data }
    }
```

### complex #30: `from_mut` -> `from_mut_unchecked`

- Source file: `rust/library/core/src/num/nonzero.rs`
- Dependent library / type: `null`
- Counterpart status: `confirmed_pair`
- Safe function lines: `441`-`447`
- Unsafe function lines: `460`-`475`
- Safety condition: Not recorded.
- Checked-side note: Not recorded.
- User visible: Not recorded.
- Flag: Not recorded.

Safe source:
```rust
    pub fn from_mut(n: &mut T) -> Option<&mut Self> {
        // SAFETY: Memory layout optimization guarantees that `Option<NonZero<T>>` has
        //         the same layout and size as `T`, with `0` representing `None`.
        let opt_n = unsafe { &mut *(ptr::from_mut(n).cast::<Option<Self>>()) };

        opt_n.as_mut()
    }
```

Unsafe source:
```rust
    pub unsafe fn from_mut_unchecked(n: &mut T) -> &mut Self {
        match Self::from_mut(n) {
            Some(n) => n,
            None => {
                // SAFETY: The caller guarantees that `n` references a value that is non-zero, so this is unreachable.
                unsafe {
                    ub_checks::assert_unsafe_precondition!(
                        check_library_ub,
                        "NonZero::from_mut_unchecked requires the argument to dereference as non-zero",
                        () => false,
                    );
                    intrinsics::unreachable()
                }
            }
        }
    }
```

### complex #31: `map` -> `map_unchecked`

- Source file: `rust/library/core/src/pin.rs`
- Dependent library / type: `null`
- Counterpart status: `unsafe_entry_only_or_unclear_pair`
- Safe function lines: `?`-`?`
- Unsafe function lines: `1534`-`1545`
- Safety condition: Not recorded.
- Checked-side note: 未找到相关safe
- User visible: Not recorded.
- Flag: Not recorded.

Safe source:
```rust
未找到相关safe
```

Unsafe source:
```rust
    pub unsafe fn map_unchecked<U, F>(self, func: F) -> Pin<&'a U>
    where
        U: ?Sized,
        F: FnOnce(&T) -> &U,
    {
        let pointer = &*self.pointer;
        let new_pointer = func(pointer);

        // SAFETY: the safety contract for `new_unchecked` must be
        // upheld by the caller.
        unsafe { Pin::new_unchecked(new_pointer) }
    }
```

### complex #32: `map_mut` -> `map_unchecked_mut`

- Source file: `rust/library/core/src/pin.rs`
- Dependent library / type: `null`
- Counterpart status: `unsafe_entry_only_or_unclear_pair`
- Safe function lines: `?`-`?`
- Unsafe function lines: `1638`-`1650`
- Safety condition: Not recorded.
- Checked-side note: 未找到相关safe
- User visible: Not recorded.
- Flag: Not recorded.

Safe source:
```rust
未找到相关safe
```

Unsafe source:
```rust
    pub unsafe fn map_unchecked_mut<U, F>(self, func: F) -> Pin<&'a mut U>
    where
        U: ?Sized,
        F: FnOnce(&mut T) -> &mut U,
    {
        // SAFETY: the caller is responsible for not moving the
        // value out of this reference.
        let pointer = unsafe { Pin::get_unchecked_mut(self) };
        let new_pointer = func(pointer);
        // SAFETY: as the value of `this` is guaranteed to not have
        // been moved out, this call to `new_unchecked` is safe.
        unsafe { Pin::new_unchecked(new_pointer) }
    }
```

### complex #33: `get_mut` -> `get_unchecked_mut`

- Source file: `rust/library/core/src/ptr/mut_ptr.rs`
- Dependent library / type: `null`
- Counterpart status: `unsafe_entry_only_or_unclear_pair`
- Safe function lines: `?`-`?`
- Unsafe function lines: `1913`-`1919`
- Safety condition: Not recorded.
- Checked-side note: 未找到相关safe
- User visible: Not recorded.
- Flag: Not recorded.

Safe source:
```rust
未找到相关safe
```

Unsafe source:
```rust
    pub const unsafe fn get_unchecked_mut<I>(self, index: I) -> *mut I::Output
    where
        I: [const] SliceIndex<[T]>,
    {
        // SAFETY: the caller ensures that `self` is dereferenceable and `index` in-bounds.
        unsafe { index.get_unchecked_mut(self) }
    }
```

### complex #34: `split_at_mut` -> `split_at_mut_unchecked`

- Source file: `rust/library/core/src/ptr/mut_ptr.rs`
- Dependent library / type: `null`
- Counterpart status: `unsafe_entry_only_or_unclear_pair`
- Safe function lines: `?`-`?`
- Unsafe function lines: `1859`-`1869`
- Safety condition: Not recorded.
- Checked-side note: Not recorded.
- User visible: Not recorded.
- Flag: Not recorded.

Safe source:
```rust
pub unsafe fn split_at_mut(self, mid: usize) -> (*mut [T], *mut [T]) {
        assert!(mid <= self.len());
        // SAFETY: The assert above is only a safety-net as long as `self.len()` is correct
        // The actual safety requirements of this function are the same as for `split_at_mut_unchecked`
        unsafe { self.split_at_mut_unchecked(mid) }
    }
```

Unsafe source:
```rust
    pub unsafe fn split_at_mut_unchecked(self, mid: usize) -> (*mut [T], *mut [T]) {
        let len = self.len();
        let ptr = self.as_mut_ptr();

        // SAFETY: Caller must pass a valid pointer and an index that is in-bounds.
        let tail = unsafe { ptr.add(mid) };
        (
            crate::ptr::slice_from_raw_parts_mut(ptr, mid),
            crate::ptr::slice_from_raw_parts_mut(tail, len - mid),
        )
    }
```

### complex #35: `get_mut` -> `get_unchecked_mut`

- Source file: `rust/library/core/src/ptr/non_null.rs`
- Dependent library / type: `null`
- Counterpart status: `unsafe_entry_only_or_unclear_pair`
- Safe function lines: `?`-`?`
- Unsafe function lines: `1664`-`1671`
- Safety condition: Not recorded.
- Checked-side note: 未找到相关safe
- User visible: Not recorded.
- Flag: Not recorded.

Safe source:
```rust
未找到相关safe
```

Unsafe source:
```rust
    pub const unsafe fn get_unchecked_mut<I>(self, index: I) -> NonNull<I::Output>
    where
        I: [const] SliceIndex<[T]>,
    {
        // SAFETY: the caller ensures that `self` is dereferenceable and `index` in-bounds.
        // As a consequence, the resulting pointer cannot be null.
        unsafe { NonNull::new_unchecked(self.as_ptr().get_unchecked_mut(index)) }
    }
```

### complex #36: `to_int` -> `to_int_unchecked`

- Source file: `rust/library/core/src/num/f128.rs`
- Dependent library / type: `num/nonzero`
- Counterpart status: `unsafe_entry_only_or_unclear_pair`
- Safe function lines: `?`-`?`
- Unsafe function lines: `952`-`959`
- Safety condition: Not recorded.
- Checked-side note: 未找到相关safe
- User visible: Not recorded.
- Flag: Not recorded.

Safe source:
```rust
未找到相关safe
```

Unsafe source:
```rust
    pub unsafe fn to_int_unchecked<Int>(self) -> Int
    where
        Self: FloatToInt<Int>,
    {
        // SAFETY: the caller must uphold the safety contract for
        // `FloatToInt::to_int_unchecked`.
        unsafe { FloatToInt::<Int>::to_int_unchecked(self) }
    }
```

### complex #37: `to_int` -> `to_int_unchecked`

- Source file: `rust/library/core/src/num/f16.rs`
- Dependent library / type: `num/nonzero`
- Counterpart status: `unsafe_entry_only_or_unclear_pair`
- Safe function lines: `?`-`?`
- Unsafe function lines: `946`-`953`
- Safety condition: Not recorded.
- Checked-side note: Not recorded.
- User visible: Not recorded.
- Flag: Not recorded.

Safe source:
```rust
Not recorded.
```

Unsafe source:
```rust
    pub unsafe fn to_int_unchecked<Int>(self) -> Int
    where
        Self: FloatToInt<Int>,
    {
        // SAFETY: the caller must uphold the safety contract for
        // `FloatToInt::to_int_unchecked`.
        unsafe { FloatToInt::<Int>::to_int_unchecked(self) }
    }
```

### complex #38: `to_int` -> `to_int_unchecked`

- Source file: `rust/library/core/src/num/f32.rs`
- Dependent library / type: `num/nonzero`
- Counterpart status: `unsafe_entry_only_or_unclear_pair`
- Safe function lines: `?`-`?`
- Unsafe function lines: `1150`-`1157`
- Safety condition: Not recorded.
- Checked-side note: 未找到相关safe
- User visible: Not recorded.
- Flag: Not recorded.

Safe source:
```rust
未找到相关safe
```

Unsafe source:
```rust
    pub unsafe fn to_int_unchecked<Int>(self) -> Int
    where
        Self: FloatToInt<Int>,
    {
        // SAFETY: the caller must uphold the safety contract for
        // `FloatToInt::to_int_unchecked`.
        unsafe { FloatToInt::<Int>::to_int_unchecked(self) }
    }
```

### complex #39: `to_int` -> `to_int_unchecked`

- Source file: `rust/library/core/src/num/f64.rs`
- Dependent library / type: `num/nonzero`
- Counterpart status: `unsafe_entry_only_or_unclear_pair`
- Safe function lines: `?`-`?`
- Unsafe function lines: `1149`-`1156`
- Safety condition: Not recorded.
- Checked-side note: 未找到相关safe
- User visible: Not recorded.
- Flag: Not recorded.

Safe source:
```rust
未找到相关safe
```

Unsafe source:
```rust
    pub unsafe fn to_int_unchecked<Int>(self) -> Int
    where
        Self: FloatToInt<Int>,
    {
        // SAFETY: the caller must uphold the safety contract for
        // `FloatToInt::to_int_unchecked`.
        unsafe { FloatToInt::<Int>::to_int_unchecked(self) }
    }
```

### complex #40: `get_mut` -> `get_mut_unchecked`

- Source file: `rust/library/alloc/src/rc.rs`
- Dependent library / type: `RC`
- Counterpart status: `confirmed_pair`
- Safe function lines: `1917`-`1919`
- Unsafe function lines: `1983`-`1987`
- Safety condition: /// If any other `Rc` or [`Weak`] pointers to the same allocation exist, then
    /// they must not be dereferenced or have active borrows for the duration
    /// of the returned borrow, and their inner type must be exactly the same as the
    /// inner type of this Rc (including lifetimes). This is trivially the case if no
    /// such pointers exist, for example immediately after `Rc::new`.
- Checked-side note: Not recorded.
- User visible: Not recorded.
- Flag: Not recorded.

Safe source:
```rust
    pub fn get_mut(this: &mut Self) -> Option<&mut T> {
        if Rc::is_unique(this) { unsafe { Some(Rc::get_mut_unchecked(this)) } } else { None }
    }
```

Unsafe source:
```rust
    pub unsafe fn get_mut_unchecked(this: &mut Self) -> &mut T {
        // We are careful to *not* create a reference covering the "count" fields, as
        // this would conflict with accesses to the reference counts (e.g. by `Weak`).
        unsafe { &mut (*this.ptr.as_ptr()).value }
    }
```

### complex #41: `gather_select` -> `gather_select_unchecked`

- Source file: `rust/library/portable-simd/crates/core_simd/src/vector.rs`
- Dependent library / type: `simd`
- Counterpart status: `confirmed_pair`
- Safe function lines: `527`-`536`
- Unsafe function lines: `568`-`579`
- Safety condition: Not recorded.
- Checked-side note: 暂不处理
- User visible: Not recorded.
- Flag: Not recorded.

Safe source:
```rust
    pub fn gather_select(
        slice: &[T],
        enable: Mask<isize, N>,
        idxs: Simd<usize, N>,
        or: Self,
    ) -> Self {
        let enable: Mask<isize, N> = enable & idxs.simd_lt(Simd::splat(slice.len()));
        // Safety: We have masked-off out-of-bounds indices.
        unsafe { Self::gather_select_unchecked(slice, enable, idxs, or) }
    }
```

Unsafe source:
```rust
    pub unsafe fn gather_select_unchecked(
        slice: &[T],
        enable: Mask<isize, N>,
        idxs: Simd<usize, N>,
        or: Self,
    ) -> Self {
        let base_ptr = Simd::<*const T, N>::splat(slice.as_ptr());
        // Ferris forgive me, I have done pointer arithmetic here.
        let ptrs = base_ptr.wrapping_add(idxs);
        // Safety: The caller is responsible for determining the indices are okay to read
        unsafe { Self::gather_select_ptr(ptrs, enable, or) }
    }
```

### complex #42: `load_select` -> `load_select_unchecked`

- Source file: `rust/library/portable-simd/crates/core_simd/src/vector.rs`
- Dependent library / type: `simd`
- Counterpart status: `confirmed_pair`
- Safe function lines: `411`-`420`
- Unsafe function lines: `432`-`440`
- Safety condition: Not recorded.
- Checked-side note: 暂不处理
- User visible: Not recorded.
- Flag: Not recorded.

Safe source:
```rust
    pub fn load_select(
        slice: &[T],
        mut enable: Mask<<T as SimdElement>::Mask, N>,
        or: Self,
    ) -> Self {
        enable &= mask_up_to(slice.len());
        // SAFETY: We performed the bounds check by updating the mask. &[T] is properly aligned to
        // the element.
        unsafe { Self::load_select_ptr(slice.as_ptr(), enable, or) }
    }
```

Unsafe source:
```rust
    pub unsafe fn load_select_unchecked(
        slice: &[T],
        enable: Mask<<T as SimdElement>::Mask, N>,
        or: Self,
    ) -> Self {
        let ptr = slice.as_ptr();
        // SAFETY: The safety of reading elements from `slice` is ensured by the caller.
        unsafe { Self::load_select_ptr(ptr, enable, or) }
    }
```

### complex #43: `scatter_select` -> `scatter_select_unchecked`

- Source file: `rust/library/portable-simd/crates/core_simd/src/vector.rs`
- Dependent library / type: `simd`
- Counterpart status: `confirmed_pair`
- Safe function lines: `764`-`768`
- Unsafe function lines: `801`-`826`
- Safety condition: Not recorded.
- Checked-side note: 暂不处理
- User visible: Not recorded.
- Flag: Not recorded.

Safe source:
```rust
    pub fn scatter_select(self, slice: &mut [T], enable: Mask<isize, N>, idxs: Simd<usize, N>) {
        let enable: Mask<isize, N> = enable & idxs.simd_lt(Simd::splat(slice.len()));
        // Safety: We have masked-off out-of-bounds indices.
        unsafe { self.scatter_select_unchecked(slice, enable, idxs) }
    }
```

Unsafe source:
```rust
    pub unsafe fn scatter_select_unchecked(
        self,
        slice: &mut [T],
        enable: Mask<isize, N>,
        idxs: Simd<usize, N>,
    ) {
        // Safety: This block works with *mut T derived from &mut 'a [T],
        // which means it is delicate in Rust's borrowing model, circa 2021:
        // &mut 'a [T] asserts uniqueness, so deriving &'a [T] invalidates live *mut Ts!
        // Even though this block is largely safe methods, it must be exactly this way
        // to prevent invalidating the raw ptrs while they're live.
        // Thus, entering this block requires all values to use being already ready:
        // 0. idxs we want to write to, which are used to construct the mask.
        // 1. enable, which depends on an initial &'a [T] and the idxs.
        // 2. actual values to scatter (self).
        // 3. &mut [T] which will become our base ptr.
        unsafe {
            // Now Entering ☢️ *mut T Zone
            let base_ptr = Simd::<*mut T, N>::splat(slice.as_mut_ptr());
            // Ferris forgive me, I have done pointer arithmetic here.
            let ptrs = base_ptr.wrapping_add(idxs);
            // The ptrs have been bounds-masked to prevent memory-unsafe writes insha'allah
            self.scatter_select_ptr(ptrs, enable);
            // Cleared ☢️ *mut T Zone
        }
    }
```

### complex #44: `store_select` -> `store_select_unchecked`

- Source file: `rust/library/portable-simd/crates/core_simd/src/vector.rs`
- Dependent library / type: `simd`
- Counterpart status: `confirmed_pair`
- Safe function lines: `664`-`669`
- Unsafe function lines: `692`-`700`
- Safety condition: Not recorded.
- Checked-side note: 暂不处理
- User visible: Not recorded.
- Flag: Not recorded.

Safe source:
```rust
    pub fn store_select(self, slice: &mut [T], mut enable: Mask<<T as SimdElement>::Mask, N>) {
        enable &= mask_up_to(slice.len());
        // SAFETY: We performed the bounds check by updating the mask. &[T] is properly aligned to
        // the element.
        unsafe { self.store_select_ptr(slice.as_mut_ptr(), enable) }
    }
```

Unsafe source:
```rust
    pub unsafe fn store_select_unchecked(
        self,
        slice: &mut [T],
        enable: Mask<<T as SimdElement>::Mask, N>,
    ) {
        let ptr = slice.as_mut_ptr();
        // SAFETY: The safety of writing elements in `slice` is ensured by the caller.
        unsafe { self.store_select_ptr(ptr, enable) }
    }
```

### complex #45: `from_boxed_utf8` -> `from_boxed_utf8_unchecked`

- Source file: `rust/library/alloc/src/str.rs`
- Dependent library / type: `slice/string`
- Counterpart status: `unsafe_entry_only_or_unclear_pair`
- Safe function lines: `?`-`?`
- Unsafe function lines: `616`-`618`
- Safety condition: Not recorded.
- Checked-side note: 未找到相关safe
- User visible: Not recorded.
- Flag: Not recorded.

Safe source:
```rust
未找到相关safe
```

Unsafe source:
```rust
pub unsafe fn from_boxed_utf8_unchecked(v: Box<[u8]>) -> Box<str> {
    unsafe { Box::from_raw(Box::into_raw(v) as *mut str) }
}
```

### complex #46: `from_utf8` -> `from_utf8_unchecked`

- Source file: `rust/library/alloc/src/string.rs`
- Dependent library / type: `slice/string`
- Counterpart status: `confirmed_pair`
- Safe function lines: `560`-`565`
- Unsafe function lines: `1013`-`1015`
- Safety condition: Not recorded.
- Checked-side note: 未找到相关safe
- User visible: Not recorded.
- Flag: Not recorded.

Safe source:
```rust
    pub fn from_utf8(vec: Vec<u8>) -> Result<String, FromUtf8Error> {
        match str::from_utf8(&vec) {
            Ok(..) => Ok(String { vec }),
            Err(e) => Err(FromUtf8Error { bytes: vec, error: e }),
        }
    }
```

Unsafe source:
```rust
    pub unsafe fn from_utf8_unchecked(bytes: Vec<u8>) -> String {
        String { vec: bytes }
    }
```

### complex #47: `get` -> `get_unchecked`

- Source file: `rust/library/core/src/ptr/const_ptr.rs`
- Dependent library / type: `slice/string`
- Counterpart status: `unsafe_entry_only_or_unclear_pair`
- Safe function lines: `?`-`?`
- Unsafe function lines: `1537`-`1543`
- Safety condition: Not recorded.
- Checked-side note: 未找到相关safe
- User visible: Not recorded.
- Flag: Not recorded.

Safe source:
```rust
未找到相关safe
```

Unsafe source:
```rust
    pub const unsafe fn get_unchecked<I>(self, index: I) -> *const I::Output
    where
        I: [const] SliceIndex<[T]>,
    {
        // SAFETY: the caller ensures that `self` is dereferenceable and `index` in-bounds.
        unsafe { index.get_unchecked(self) }
    }
```

### complex #48: `get_disjoint_mut` -> `get_disjoint_unchecked_mut`

- Source file: `rust/library/core/src/slice/mod.rs`
- Dependent library / type: `slice/string`
- Counterpart status: `confirmed_pair`
- Safe function lines: `5209`-`5220`
- Unsafe function lines: `5142`-`5166`
- Safety condition: // SAFETY: We expect `indices` to contain disjunct values that are
        // in bounds of `self`.
- Checked-side note: Not recorded.
- User visible: Not recorded.
- Flag: Not recorded.

Safe source:
```rust
    pub fn get_disjoint_mut<I, const N: usize>(
        &mut self,
        indices: [I; N],
    ) -> Result<[&mut I::Output; N], GetDisjointMutError>
    where
        I: GetDisjointMutIndex + SliceIndex<Self>,
    {
        get_disjoint_check_valid(&indices, self.len())?;
        // SAFETY: The `get_disjoint_check_valid()` call checked that all indices
        // are disjunct and in bounds.
        unsafe { Ok(self.get_disjoint_unchecked_mut(indices)) }
    }
```

Unsafe source:
```rust
    pub unsafe fn get_disjoint_unchecked_mut<I, const N: usize>(
        &mut self,
        indices: [I; N],
    ) -> [&mut I::Output; N]
    where
        I: GetDisjointMutIndex + SliceIndex<Self>,
    {
        // NB: This implementation is written as it is because any variation of
        // `indices.map(|i| self.get_unchecked_mut(i))` would make miri unhappy,
        // or generate worse code otherwise. This is also why we need to go
        // through a raw pointer here.
        let slice: *mut [T] = self;
        let mut arr: MaybeUninit<[&mut I::Output; N]> = MaybeUninit::uninit();
        let arr_ptr = arr.as_mut_ptr();

        // SAFETY: We expect `indices` to contain disjunct values that are
        // in bounds of `self`.
        unsafe {
            for i in 0..N {
                let idx = indices.get_unchecked(i).clone();
                arr_ptr.cast::<&mut I::Output>().add(i).write(&mut *slice.get_unchecked_mut(idx));
            }
            arr.assume_init()
        }
    }
```

### complex #49: `slice` -> `slice_unchecked`

- Source file: `rust/library/core/src/str/mod.rs`
- Dependent library / type: `slice/string`
- Counterpart status: `unsafe_entry_only_or_unclear_pair`
- Safe function lines: `?`-`?`
- Unsafe function lines: `769`-`774`
- Safety condition: Not recorded.
- Checked-side note: 未找到相关safe
- User visible: Not recorded.
- Flag: Not recorded.

Safe source:
```rust
未找到相关safe
```

Unsafe source:
```rust
    pub unsafe fn slice_unchecked(&self, begin: usize, end: usize) -> &str {
        // SAFETY: the caller must uphold the safety contract for `get_unchecked`;
        // the slice is dereferenceable because `self` is a safe reference.
        // The returned pointer is safe because impls of `SliceIndex` have to guarantee that it is.
        unsafe { &*(begin..end).get_unchecked(self) }
    }
```

### complex #50: `slice_mut` -> `slice_mut_unchecked`

- Source file: `rust/library/core/src/str/mod.rs`
- Dependent library / type: `slice/string`
- Counterpart status: `unsafe_entry_only_or_unclear_pair`
- Safe function lines: `?`-`?`
- Unsafe function lines: `803`-`808`
- Safety condition: Not recorded.
- Checked-side note: 未找到相关safe
- User visible: Not recorded.
- Flag: Not recorded.

Safe source:
```rust
未找到相关safe
```

Unsafe source:
```rust
    pub unsafe fn slice_mut_unchecked(&mut self, begin: usize, end: usize) -> &mut str {
        // SAFETY: the caller must uphold the safety contract for `get_unchecked_mut`;
        // the slice is dereferenceable because `self` is a safe reference.
        // The returned pointer is safe because impls of `SliceIndex` have to guarantee that it is.
        unsafe { &mut *(begin..end).get_unchecked_mut(self) }
    }
```

### complex #51: `from_bytes` -> `from_bytes_unchecked`

- Source file: `../../refs/rust/library/std/src/sys_common/wtf8.rs`
- Dependent library / type: `unicode/char`
- Counterpart status: `unsafe_entry_only_or_unclear_pair`
- Safe function lines: `?`-`?`
- Unsafe function lines: `626`-`629`
- Safety condition: null
- Checked-side note: 未找到对应的safe
- User visible: Not recorded.
- Flag: Not recorded.

Safe source:
```rust
未找到对应的safe
```

Unsafe source:
```rust
pub unsafe fn from_bytes_unchecked(value: &[u8]) -> &Wtf8 {
        // SAFETY: start with &[u8], end with fancy &[u8]
        unsafe { &*(value as *const [u8] as *const Wtf8) }
    }
```

### complex #52: `slice` -> `slice_unchecked`

- Source file: `../../refs/rust/library/std/src/sys_common/wtf8.rs`
- Dependent library / type: `unicode/char`
- Counterpart status: `unsafe_entry_only_or_unclear_pair`
- Safe function lines: `?`-`?`
- Unsafe function lines: `967`-`974`
- Safety condition: null
- Checked-side note: 未找到对应的safe
- User visible: Not recorded.
- Flag: Not recorded.

Safe source:
```rust
未找到对应的safe
```

Unsafe source:
```rust
pub unsafe fn slice_unchecked(s: &Wtf8, begin: usize, end: usize) -> &Wtf8 {
    // SAFETY: memory layout of a &[u8] and &Wtf8 are the same
    unsafe {
        let len = end - begin;
        let start = s.as_bytes().as_ptr().add(begin);
        Wtf8::from_bytes_unchecked(slice::from_raw_parts(start, len))
    }
}
```

### complex #53: `from_bytes` -> `from_bytes_unchecked`

- Source file: `../../refs/rust/library/std/src/sys_common/wtf8.rs`
- Dependent library / type: `uniode/char`
- Counterpart status: `unsafe_entry_only_or_unclear_pair`
- Safe function lines: `?`-`?`
- Unsafe function lines: `202`-`204`
- Safety condition: null
- Checked-side note: 未找到对应的safe
- User visible: Not recorded.
- Flag: Not recorded.

Safe source:
```rust
未找到对应的safe
```

Unsafe source:
```rust
pub unsafe fn from_bytes_unchecked(value: Vec<u8>) -> Wtf8Buf {
        Wtf8Buf { bytes: value, is_known_utf8: false }
    }
```

## Category: `CString`

### CString #1: `from_bytes_with_nul` -> `from_bytes_with_nul_unchecked`

- Source file: `rust/library/core/src/ffi/c_str.rs`
- Dependent library / type: `Cstr`
- Counterpart status: `confirmed_pair`
- Safe function lines: `351`-`362`
- Unsafe function lines: `388`-`418`
- Safety condition: 保证字符串以空字符结尾,并且没有其他的空字节
- Checked-side note: Not recorded.
- User visible: Not recorded.
- Flag: Not recorded.

Safe source:
```rust
    pub const fn from_bytes_with_nul(bytes: &[u8]) -> Result<&Self, FromBytesWithNulError> {
        let nul_pos = memchr::memchr(0, bytes);
        match nul_pos {
            Some(nul_pos) if nul_pos + 1 == bytes.len() => {
                // SAFETY: We know there is only one nul byte, at the end
                // of the byte slice.
                Ok(unsafe { Self::from_bytes_with_nul_unchecked(bytes) })
            }
            Some(position) => Err(FromBytesWithNulError::InteriorNul { position }),
            None => Err(FromBytesWithNulError::NotNulTerminated),
        }
    }
```

Unsafe source:
```rust
    pub const unsafe fn from_bytes_with_nul_unchecked(bytes: &[u8]) -> &CStr {
        const_eval_select!(
            @capture { bytes: &[u8] } -> &CStr:
            if const {
                // Saturating so that an empty slice panics in the assert with a good
                // message, not here due to underflow.
                let mut i = bytes.len().saturating_sub(1);
                assert!(!bytes.is_empty() && bytes[i] == 0, "input was not nul-terminated");

                // Ending nul byte exists, skip to the rest.
                while i != 0 {
                    i -= 1;
                    let byte = bytes[i];
                    assert!(byte != 0, "input contained interior nul");
                }

                // SAFETY: See runtime cast comment below.
                unsafe { &*(bytes as *const [u8] as *const CStr) }
            } else {
                // Chance at catching some UB at runtime with debug builds.
                debug_assert!(!bytes.is_empty() && bytes[bytes.len() - 1] == 0);

                // SAFETY: Casting to CStr is safe because its internal representation
                // is a [u8] too (safe only inside std).
                // Dereferencing the obtained pointer is safe because it comes from a
                // reference. Making a reference is then safe because its lifetime
                // is bound by the lifetime of the given `bytes`.
                unsafe { &*(bytes as *const [u8] as *const CStr) }
            }
        )
    }
```

## Category: `index`

### index #1: `as_ascii` -> `as_ascii_unchecked`

- Source file: `rust/library/core/src/array/ascii.rs`
- Dependent library / type: `array`
- Counterpart status: `confirmed_pair`
- Safe function lines: `21`-`28`
- Unsafe function lines: `39`-`44`
- Safety condition: 保证所有的字符都是ascii
- Checked-side note: Not recorded.
- User visible: Not recorded.
- Flag: Not recorded.

Safe source:
```rust
    pub const fn as_ascii(&self) -> Option<&[ascii::Char; N]> {
        if self.is_ascii() {
            // SAFETY: Just checked that it's ASCII
            Some(unsafe { self.as_ascii_unchecked() })
        } else {
            None
        }
    }
```

Unsafe source:
```rust
    pub const unsafe fn as_ascii_unchecked(&self) -> &[ascii::Char; N] {
        let byte_ptr: *const [u8; N] = self;
        let ascii_ptr = byte_ptr as *const [ascii::Char; N];
        // SAFETY: The caller promised all the bytes are ASCII
        unsafe { &*ascii_ptr }
    }
```

### index #2: `digit` -> `digit_unchecked`

- Source file: `rust/library/core/src/ascii/ascii_char.rs`
- Dependent library / type: `ascii`
- Counterpart status: `confirmed_pair`
- Safe function lines: `489`-`496`
- Unsafe function lines: `516`-`530`
- Safety condition: 保证参数对应ascii码中0-9的范围
- Checked-side note: Not recorded.
- User visible: Not recorded.
- Flag: Not recorded.

Safe source:
```rust
    pub const fn digit(d: u8) -> Option<Self> {
        if d < 10 {
            // SAFETY: Just checked it's in-range.
            Some(unsafe { Self::digit_unchecked(d) })
        } else {
            None
        }
    }
```

Unsafe source:
```rust
    pub const unsafe fn digit_unchecked(d: u8) -> Self {
        assert_unsafe_precondition!(
            check_library_ub,
            "`ascii::Char::digit_unchecked` input cannot exceed 9.",
            (d: u8 = d) => d < 10
        );

        // SAFETY: `'0'` through `'9'` are U+00030 through U+0039,
        // so because `d` must be 64 or less the addition can return at most
        // 112 (0x70), which doesn't overflow and is within the ASCII range.
        unsafe {
            let byte = b'0'.unchecked_add(d);
            Self::from_u8_unchecked(byte)
        }
    }
```

### index #3: `from_u8` -> `from_u8_unchecked`

- Source file: `rust/library/core/src/ascii/ascii_char.rs`
- Dependent library / type: `ascii`
- Counterpart status: `confirmed_pair`
- Safe function lines: `461`-`468`
- Unsafe function lines: `478`-`481`
- Safety condition: 必须保证参数属于ascii码范围
- Checked-side note: Not recorded.
- User visible: Not recorded.
- Flag: Not recorded.

Safe source:
```rust
    pub const fn from_u8(b: u8) -> Option<Self> {
        if b <= 127 {
            // SAFETY: Just checked that `b` is in-range
            Some(unsafe { Self::from_u8_unchecked(b) })
        } else {
            None
        }
    }
```

Unsafe source:
```rust
    pub const unsafe fn from_u8_unchecked(b: u8) -> Self {
        // SAFETY: Our safety precondition is that `b` is in-range.
        unsafe { transmute(b) }
    }
```

### index #4: `from_u32` -> `from_u32_unchecked`

- Source file: `rust/library/core/src/char/methods.rs`
- Dependent library / type: `char`
- Counterpart status: `confirmed_pair`
- Safe function lines: `196`-`198`
- Unsafe function lines: `237`-`240`
- Safety condition: u32类型参数在所在unicode码表中
- Checked-side note: Not recorded.
- User visible: Not recorded.
- Flag: Not recorded.

Safe source:
```rust
    pub const fn from_u32(i: u32) -> Option<char> {
        super::convert::from_u32(i)
    }
```

Unsafe source:
```rust
    pub const unsafe fn from_u32_unchecked(i: u32) -> char {
        // SAFETY: the safety contract must be upheld by the caller.
        unsafe { super::convert::from_u32_unchecked(i) }
    }
```

### index #5: `from_u32` -> `from_u32_unchecked`

- Source file: `rust/library/core/src/char/mod.rs`
- Dependent library / type: `char`
- Counterpart status: `confirmed_pair`
- Safe function lines: `131`-`133`
- Unsafe function lines: `141`-`144`
- Safety condition: u32类型参数在所在unicode码表中
- Checked-side note: Not recorded.
- User visible: Not recorded.
- Flag: Not recorded.

Safe source:
```rust
pub const fn from_u32(i: u32) -> Option<char> {
    self::convert::from_u32(i)
}
```

Unsafe source:
```rust
pub const unsafe fn from_u32_unchecked(i: u32) -> char {
    // SAFETY: the safety contract must be upheld by the caller.
    unsafe { self::convert::from_u32_unchecked(i) }
}
```

### index #6: `new` -> `new_unchecked`

- Source file: `../../refs/rust/library/core/src/ptr/alignment.rs`
- Dependent library / type: `ptr`
- Counterpart status: `unsafe_entry_only_or_unclear_pair`
- Safe function lines: `?`-`?`
- Unsafe function lines: `76`-`86`
- Safety condition: 对齐参数必须为2的幂并且不能为0
- Checked-side note: Not recorded.
- User visible: Not recorded.
- Flag: Not recorded.

Safe source:
```rust
pub const fn new(align: usize) -> Option<Self> {
        if align.is_power_of_two() {
            // SAFETY: Just checked it only has one bit set
            Some(unsafe { Self::new_unchecked(align) })
        } else {
            None
        }
    }
```

Unsafe source:
```rust
pub const unsafe fn new_unchecked(align: usize) -> Self {
        assert_unsafe_precondition!(
            check_language_ub,
            "Alignment::new_unchecked requires a power of two",
            (align: usize = align) => align.is_power_of_two()
        );

        // SAFETY: By precondition, this must be a power of two, and
        // our variants encompass all possible powers of two.
        unsafe { mem::transmute::<usize, Alignment>(align) }
    }
```

### index #7: `as_ascii` -> `as_ascii_unchecked`

- Source file: `rust/library/core/src/slice/ascii.rs`
- Dependent library / type: `slice/string`
- Counterpart status: `confirmed_pair`
- Safe function lines: `27`-`34`
- Unsafe function lines: `45`-`50`
- Safety condition: 用户必须保证参数都符合ascii范围内
- Checked-side note: Not recorded.
- User visible: Not recorded.
- Flag: Not recorded.

Safe source:
```rust
    pub const fn as_ascii(&self) -> Option<&[ascii::Char]> {
        if self.is_ascii() {
            // SAFETY: Just checked that it's ASCII
            Some(unsafe { self.as_ascii_unchecked() })
        } else {
            None
        }
    }
```

Unsafe source:
```rust
    pub const unsafe fn as_ascii_unchecked(&self) -> &[ascii::Char] {
        let byte_ptr: *const [u8] = self;
        let ascii_ptr = byte_ptr as *const [ascii::Char];
        // SAFETY: The caller promised all the bytes are ASCII
        unsafe { &*ascii_ptr }
    }
```

### index #8: `get` -> `get_unchecked`

- Source file: `rust/library/core/src/slice/mod.rs`
- Dependent library / type: `slice/string`
- Counterpart status: `confirmed_pair`
- Safe function lines: `572`-`577`
- Unsafe function lines: `639`-`647`
- Safety condition: 访问不越界
- Checked-side note: Not recorded.
- User visible: Not recorded.
- Flag: Not recorded.

Safe source:
```rust
    pub const fn get<I>(&self, index: I) -> Option<&I::Output>
    where
        I: [const] SliceIndex<Self>,
    {
        index.get(self)
    }
```

Unsafe source:
```rust
    pub const unsafe fn get_unchecked<I>(&self, index: I) -> &I::Output
    where
        I: [const] SliceIndex<Self>,
    {
        // SAFETY: the caller must uphold most of the safety requirements for `get_unchecked`;
        // the slice is dereferenceable because `self` is a safe reference.
        // The returned pointer is safe because impls of `SliceIndex` have to guarantee that it is.
        unsafe { &*index.get_unchecked(self) }
    }
```

### index #9: `get_mut` -> `get_unchecked_mut`

- Source file: `rust/library/core/src/slice/mod.rs`
- Dependent library / type: `slice/string`
- Counterpart status: `confirmed_pair`
- Safe function lines: `599`-`604`
- Unsafe function lines: `684`-`692`
- Safety condition: 数组访问不越界
- Checked-side note: Not recorded.
- User visible: Not recorded.
- Flag: Not recorded.

Safe source:
```rust
    pub const fn get_mut<I>(&mut self, index: I) -> Option<&mut I::Output>
    where
        I: [const] SliceIndex<Self>,
    {
        index.get_mut(self)
    }
```

Unsafe source:
```rust
    pub const unsafe fn get_unchecked_mut<I>(&mut self, index: I) -> &mut I::Output
    where
        I: [const] SliceIndex<Self>,
    {
        // SAFETY: the caller must uphold the safety requirements for `get_unchecked_mut`;
        // the slice is dereferenceable because `self` is a safe reference.
        // The returned pointer is safe because impls of `SliceIndex` have to guarantee that it is.
        unsafe { &mut *index.get_unchecked_mut(self) }
    }
```

### index #10: `split_at_checked` -> `split_at_unchecked`

- Source file: `rust/library/core/src/slice/mod.rs`
- Dependent library / type: `slice/String`
- Counterpart status: `confirmed_pair`
- Safe function lines: `2153`-`2161`
- Unsafe function lines: `2038`-`2054`
- Safety condition: 用户必须保证参数mid在0到self.len之间
- Checked-side note: Not recorded.
- User visible: Not recorded.
- Flag: Not recorded.

Safe source:
```rust
    pub const fn split_at_checked(&self, mid: usize) -> Option<(&[T], &[T])> {
        if mid <= self.len() {
            // SAFETY: `[ptr; mid]` and `[mid; len]` are inside `self`, which
            // fulfills the requirements of `split_at_unchecked`.
            Some(unsafe { self.split_at_unchecked(mid) })
        } else {
            None
        }
    }
```

Unsafe source:
```rust
    pub const unsafe fn split_at_unchecked(&self, mid: usize) -> (&[T], &[T]) {
        // FIXME(const-hack): the const function `from_raw_parts` is used to make this
        // function const; previously the implementation used
        // `(self.get_unchecked(..mid), self.get_unchecked(mid..))`

        let len = self.len();
        let ptr = self.as_ptr();

        assert_unsafe_precondition!(
            check_library_ub,
            "slice::split_at_unchecked requires the index to be within the slice",
            (mid: usize = mid, len: usize = len) => mid <= len,
        );

        // SAFETY: Caller has to check that `0 <= mid <= self.len()`
        unsafe { (from_raw_parts(ptr, mid), from_raw_parts(ptr.add(mid), unchecked_sub(len, mid))) }
    }
```

### index #11: `split_at_mut_checked` -> `split_at_mut_unchecked`

- Source file: `rust/library/core/src/slice/mod.rs`
- Dependent library / type: `slice/string`
- Counterpart status: `confirmed_pair`
- Safe function lines: `2192`-`2200`
- Unsafe function lines: `2092`-`2112`
- Safety condition: 用户必须保证参数mid在0到self.len之间
- Checked-side note: Not recorded.
- User visible: Not recorded.
- Flag: Not recorded.

Safe source:
```rust
    pub const fn split_at_mut_checked(&mut self, mid: usize) -> Option<(&mut [T], &mut [T])> {
        if mid <= self.len() {
            // SAFETY: `[ptr; mid]` and `[mid; len]` are inside `self`, which
            // fulfills the requirements of `split_at_unchecked`.
            Some(unsafe { self.split_at_mut_unchecked(mid) })
        } else {
            None
        }
    }
```

Unsafe source:
```rust
    pub const unsafe fn split_at_mut_unchecked(&mut self, mid: usize) -> (&mut [T], &mut [T]) {
        let len = self.len();
        let ptr = self.as_mut_ptr();

        assert_unsafe_precondition!(
            check_library_ub,
            "slice::split_at_mut_unchecked requires the index to be within the slice",
            (mid: usize = mid, len: usize = len) => mid <= len,
        );

        // SAFETY: Caller has to check that `0 <= mid <= self.len()`.
        //
        // `[ptr; mid]` and `[mid; len]` are not overlapping, so returning a mutable reference
        // is fine.
        unsafe {
            (
                from_raw_parts_mut(ptr, mid),
                from_raw_parts_mut(ptr.add(mid), unchecked_sub(len, mid)),
            )
        }
    }
```

### index #12: `swap` -> `swap_unchecked`

- Source file: `rust/library/core/src/slice/mod.rs`
- Dependent library / type: `slice/string`
- Counterpart status: `confirmed_pair`
- Safe function lines: `905`-`917`
- Unsafe function lines: `948`-`964`
- Safety condition: 需要保证参数对应两个切片的索引大于切片的长度
- Checked-side note: Not recorded.
- User visible: Not recorded.
- Flag: Not recorded.

Safe source:
```rust
    pub const fn swap(&mut self, a: usize, b: usize) {
        // FIXME: use swap_unchecked here (https://github.com/rust-lang/rust/pull/88540#issuecomment-944344343)
        // Can't take two mutable loans from one vector, so instead use raw pointers.
        let pa = &raw mut self[a];
        let pb = &raw mut self[b];
        // SAFETY: `pa` and `pb` have been created from safe mutable references and refer
        // to elements in the slice and therefore are guaranteed to be valid and aligned.
        // Note that accessing the elements behind `a` and `b` is checked and will
        // panic when out of bounds.
        unsafe {
            ptr::swap(pa, pb);
        }
    }
```

Unsafe source:
```rust
    pub const unsafe fn swap_unchecked(&mut self, a: usize, b: usize) {
        assert_unsafe_precondition!(
            check_library_ub,
            "slice::swap_unchecked requires that the indices are within the slice",
            (
                len: usize = self.len(),
                a: usize = a,
                b: usize = b,
            ) => a < len && b < len,
        );

        let ptr = self.as_mut_ptr();
        // SAFETY: caller has to guarantee that `a < self.len()` and `b < self.len()`
        unsafe {
            ptr::swap(ptr.add(a), ptr.add(b));
        }
    }
```

### index #13: `get` -> `get_unchecked`

- Source file: `rust/library/core/src/str/mod.rs`
- Dependent library / type: `slice/string`
- Counterpart status: `confirmed_pair`
- Safe function lines: `618`-`620`
- Unsafe function lines: `683`-`688`
- Safety condition: 数组不能越界
/// * The starting index must not exceed the ending index;
    /// * Indexes must be within bounds of the original slice;
    /// * Indexes must lie on UTF-8 sequence boundaries.
- Checked-side note: Not recorded.
- User visible: Not recorded.
- Flag: Not recorded.

Safe source:
```rust
    pub const fn get<I: [const] SliceIndex<str>>(&self, i: I) -> Option<&I::Output> {
        i.get(self)
    }
```

Unsafe source:
```rust
    pub unsafe fn get_unchecked<I: SliceIndex<str>>(&self, i: I) -> &I::Output {
        // SAFETY: the caller must uphold the safety contract for `get_unchecked`;
        // the slice is dereferenceable because `self` is a safe reference.
        // The returned pointer is safe because impls of `SliceIndex` have to guarantee that it is.
        unsafe { &*i.get_unchecked(self) }
    }
```

### index #14: `get_mut` -> `get_unchecked_mut`

- Source file: `rust/library/core/src/str/mod.rs`
- Dependent library / type: `slice/string`
- Counterpart status: `confirmed_pair`
- Safe function lines: `651`-`653`
- Unsafe function lines: `718`-`723`
- Safety condition: 数组不能越界
/// * The starting index must not exceed the ending index;
    /// * Indexes must be within bounds of the original slice;
    /// * Indexes must lie on UTF-8 sequence boundaries.
- Checked-side note: Not recorded.
- User visible: Not recorded.
- Flag: Not recorded.

Safe source:
```rust
    pub const fn get_mut<I: [const] SliceIndex<str>>(&mut self, i: I) -> Option<&mut I::Output> {
        i.get_mut(self)
    }
```

Unsafe source:
```rust
    pub unsafe fn get_unchecked_mut<I: SliceIndex<str>>(&mut self, i: I) -> &mut I::Output {
        // SAFETY: the caller must uphold the safety contract for `get_unchecked_mut`;
        // the slice is dereferenceable because `self` is a safe reference.
        // The returned pointer is safe because impls of `SliceIndex` have to guarantee that it is.
        unsafe { &mut *i.get_unchecked_mut(self) }
    }
```

### index #15: `from_u32` -> `from_u32_unchecked`

- Source file: `../../refs/rust/library/std/src/sys_common/wtf8.rs`
- Dependent library / type: `Unicode./char`
- Counterpart status: `unsafe_entry_only_or_unclear_pair`
- Safe function lines: `?`-`?`
- Unsafe function lines: `60`-`62`
- Safety condition: 安全条件是value范围在特定范围内
- Checked-side note: Not recorded.
- User visible: Not recorded.
- Flag: Not recorded.

Safe source:
```rust
pub fn from_u32(value: u32) -> Option<CodePoint> {
        match value {
            0..=0x10FFFF => Some(CodePoint { value }),
            _ => None,
        }
    }
```

Unsafe source:
```rust
pub unsafe fn from_u32_unchecked(value: u32) -> CodePoint {
        CodePoint { value }
    }
```

### index #16: `set` -> `set_unchecked`

- Source file: `rust/library/portable-simd/crates/core_simd/src/masks.rs`
- Dependent library / type: `vector`
- Counterpart status: `confirmed_pair`
- Safe function lines: `274`-`276`
- Unsafe function lines: `261`-`266`
- Safety condition: 安全条件是index小于等于向量长度
- Checked-side note: Not recorded.
- User visible: Not recorded.
- Flag: Not recorded.

Safe source:
```rust
    pub fn set(&mut self, index: usize, value: bool) {
        self.0[index] = if value { T::TRUE } else { T::FALSE }
    }
```

Unsafe source:
```rust
    pub unsafe fn set_unchecked(&mut self, index: usize, value: bool) {
        // Safety: the caller must confirm this invariant
        unsafe {
            *self.0.as_mut_array().get_unchecked_mut(index) = if value { T::TRUE } else { T::FALSE }
        }
    }
```

### index #17: `test` -> `test_unchecked`

- Source file: `rust/library/portable-simd/crates/core_simd/src/masks.rs`
- Dependent library / type: `vector`
- Counterpart status: `confirmed_pair`
- Safe function lines: `252`-`254`
- Unsafe function lines: `240`-`243`
- Safety condition: 安全条件是index要小于等于数组边界
- Checked-side note: Not recorded.
- User visible: Not recorded.
- Flag: Not recorded.

Safe source:
```rust
    pub fn test(&self, index: usize) -> bool {
        T::eq(self.0[index], T::TRUE)
    }
```

Unsafe source:
```rust
    pub unsafe fn test_unchecked(&self, index: usize) -> bool {
        // Safety: the caller must confirm this invariant
        unsafe { T::eq(*self.0.as_array().get_unchecked(index), T::TRUE) }
    }
```

## Category: `index?`

### index? #1: `from_size_align` -> `from_size_align_unchecked`

- Source file: `rust/library/core/src/alloc/layout.rs`
- Dependent library / type: `alloc`
- Counterpart status: `confirmed_pair`
- Safe function lines: `59`-`66`
- Unsafe function lines: `132`-`144`
- Safety condition: 一个用于分配内存的方法, 安全条件是用户需要检查
内存对齐参数是2的幂并且不能超过整数最大值
- Checked-side note: Not recorded.
- User visible: Not recorded.
- Flag: Not recorded.

Safe source:
```rust
    pub const fn from_size_align(size: usize, align: usize) -> Result<Self, LayoutError> {
        if Layout::is_size_align_valid(size, align) {
            // SAFETY: Layout::is_size_align_valid checks the preconditions for this call.
            unsafe { Ok(Layout { size, align: mem::transmute(align) }) }
        } else {
            Err(LayoutError)
        }
    }
```

Unsafe source:
```rust
    pub const unsafe fn from_size_align_unchecked(size: usize, align: usize) -> Self {
        assert_unsafe_precondition!(
            check_library_ub,
            "Layout::from_size_align_unchecked requires that align is a power of 2 \
            and the rounded-up allocation size does not exceed isize::MAX",
            (
                size: usize = size,
                align: usize = align,
            ) => Layout::is_size_align_valid(size, align)
        );
        // SAFETY: the caller is required to uphold the preconditions.
        unsafe { Layout { size, align: mem::transmute(align) } }
    }
```

## Category: `intrinsic`

### intrinsic #1: `assume` -> `assert_unchecked`

- Source file: `rust/library/core/src/hint.rs`
- Dependent library / type: `core::hint`
- Counterpart status: `unsafe_entry_only_or_unclear_pair`
- Safe function lines: `?`-`?`
- Unsafe function lines: `202`-`212`
- Safety condition: Rarely used, it is equal to the cond is always true. 
It invokes unreachable() inside and leads to UB if reached.
- Checked-side note: Not recorded.
- User visible: handled
- Flag: Not recorded.

Safe source:
```rust
pub const unsafe fn assume(b: bool) {
    if !b {
        // SAFETY: the caller must guarantee the argument is never `false`
        unsafe { unreachable() }
    }
}
or
assert!()
```

Unsafe source:
```rust
pub const unsafe fn assert_unchecked(cond: bool) {
    // SAFETY: The caller promised `cond` is true.
    unsafe {
        ub_checks::assert_unsafe_precondition!(
            check_language_ub,
            "hint::assert_unchecked must never be called when the condition is false",
            (cond: bool = cond) => cond,
        );
        crate::intrinsics::assume(cond);
    }
}
```

### intrinsic #2: `unreachable` -> `unreachable_unchecked`

- Source file: `rust/library/core/src/hint.rs`
- Dependent library / type: `core::hint`
- Counterpart status: `unsafe_entry_only_or_unclear_pair`
- Safe function lines: `?`-`?`
- Unsafe function lines: `103`-`112`
- Safety condition: Rarely used, it's safe only when this code branch will never be reached. Otherwise, it will lead to UB.
- Checked-side note: Not recorded.
- User visible: handled
- Flag: Not recorded.

Safe source:
```rust
pub const unsafe fn unreachable() -> !; ---> will UB
 or
unreachable!() ---> will panic
```

Unsafe source:
```rust
pub const unsafe fn unreachable_unchecked() -> ! {
    ub_checks::assert_unsafe_precondition!(
        check_language_ub,
        "hint::unreachable_unchecked must never be reached",
        () => false
    );
    // SAFETY: the safety contract for `intrinsics::unreachable` must
    // be upheld by the caller.
    unsafe { intrinsics::unreachable() }
}
```

### intrinsic #3: `checked_add` -> `unchecked_add`

- Source file: `rust/library/core/src/intrinsics/mod.rs`
- Dependent library / type: `core::intrinsic`
- Counterpart status: `unsafe_entry_only_or_unclear_pair`
- Safe function lines: `?`-`?`
- Unsafe function lines: `1988`-`2030`
- Safety condition: Not used directly, see library/core/src/num for details
- Checked-side note: No directly corresponding safe function
- User visible: handled
- Flag: Not recorded.

Safe source:
```rust
No directly corresponding safe function
```

Unsafe source:
```rust
pub const unsafe fn unchecked_add<T: Copy>(x: T, y: T) -> T;

/// Returns the result of an unchecked subtraction, resulting in
/// undefined behavior when `x - y > T::MAX` or `x - y < T::MIN`.
///
/// The stable counterpart of this intrinsic is `unchecked_sub` on the various
/// integer types, such as [`u16::unchecked_sub`] and [`i64::unchecked_sub`].
#[rustc_intrinsic_const_stable_indirect]
#[rustc_nounwind]
#[rustc_intrinsic]
pub const unsafe fn unchecked_sub<T: Copy>(x: T, y: T) -> T;

/// Returns the result of an unchecked multiplication, resulting in
/// undefined behavior when `x * y > T::MAX` or `x * y < T::MIN`.
///
/// The stable counterpart of this intrinsic is `unchecked_mul` on the various
/// integer types, such as [`u16::unchecked_mul`] and [`i64::unchecked_mul`].
#[rustc_intrinsic_const_stable_indirect]
#[rustc_nounwind]
#[rustc_intrinsic]
pub const unsafe fn unchecked_mul<T: Copy>(x: T, y: T) -> T;

/// Performs rotate left.
///
/// Note that, unlike most intrinsics, this is safe to call;
/// it does not require an `unsafe` block.
/// Therefore, implementations must not require the user to uphold
/// any safety invariants.
///
/// The stabilized versions of this intrinsic are available on the integer
/// primitives via the `rotate_left` method. For example,
/// [`u32::rotate_left`]
#[rustc_intrinsic_const_stable_indirect]
#[rustc_nounwind]
#[rustc_intrinsic]
#[rustc_allow_const_fn_unstable(const_trait_impl, funnel_shifts)]
#[miri::intrinsic_fallback_is_spec]
pub const fn rotate_left<T: [const] fallback::FunnelShift>(x: T, shift: u32) -> T {
    // Make sure to call the intrinsic for `funnel_shl`, not the fallback impl.
    // SAFETY: we modulo `shift` so that the result is definitely less than the size of
    // `T` in bits.
    unsafe { unchecked_funnel_shl(x, x, shift % (mem::size_of::<T>() as u32 * 8)) }
}
```

### intrinsic #4: `checked_div` -> `unchecked_div`

- Source file: `rust/library/core/src/intrinsics/mod.rs`
- Dependent library / type: `core::intrinsic`
- Counterpart status: `unsafe_entry_only_or_unclear_pair`
- Safe function lines: `?`-`?`
- Unsafe function lines: `1947`-`2030`
- Safety condition: Not used directly, see library/core/src/num for details
- Checked-side note: No directly corresponding safe function
- User visible: handled
- Flag: Not recorded.

Safe source:
```rust
No directly corresponding safe function
```

Unsafe source:
```rust
pub const unsafe fn unchecked_div<T: Copy>(x: T, y: T) -> T;
/// Returns the remainder of an unchecked division, resulting in
/// undefined behavior when `y == 0` or `x == T::MIN && y == -1`
///
/// Safe wrappers for this intrinsic are available on the integer
/// primitives via the `checked_rem` method. For example,
/// [`u32::checked_rem`]
#[rustc_intrinsic_const_stable_indirect]
#[rustc_nounwind]
#[rustc_intrinsic]
pub const unsafe fn unchecked_rem<T: Copy>(x: T, y: T) -> T;

/// Performs an unchecked left shift, resulting in undefined behavior when
/// `y < 0` or `y >= N`, where N is the width of T in bits.
///
/// Safe wrappers for this intrinsic are available on the integer
/// primitives via the `checked_shl` method. For example,
/// [`u32::checked_shl`]
#[rustc_intrinsic_const_stable_indirect]
#[rustc_nounwind]
#[rustc_intrinsic]
pub const unsafe fn unchecked_shl<T: Copy, U: Copy>(x: T, y: U) -> T;
/// Performs an unchecked right shift, resulting in undefined behavior when
/// `y < 0` or `y >= N`, where N is the width of T in bits.
///
/// Safe wrappers for this intrinsic are available on the integer
/// primitives via the `checked_shr` method. For example,
/// [`u32::checked_shr`]
#[rustc_intrinsic_const_stable_indirect]
#[rustc_nounwind]
#[rustc_intrinsic]
pub const unsafe fn unchecked_shr<T: Copy, U: Copy>(x: T, y: U) -> T;

/// Returns the result of an unchecked addition, resulting in
/// undefined behavior when `x + y > T::MAX` or `x + y < T::MIN`.
///
/// The stable counterpart of this intrinsic is `unchecked_add` on the various
/// integer types, such as [`u16::unchecked_add`] and [`i64::unchecked_add`].
#[rustc_intrinsic_const_stable_indirect]
#[rustc_nounwind]
#[rustc_intrinsic]
pub const unsafe fn unchecked_add<T: Copy>(x: T, y: T) -> T;

/// Returns the result of an unchecked subtraction, resulting in
/// undefined behavior when `x - y > T::MAX` or `x - y < T::MIN`.
///
/// The stable counterpart of this intrinsic is `unchecked_sub` on the various
/// integer types, such as [`u16::unchecked_sub`] and [`i64::unchecked_sub`].
#[rustc_intrinsic_const_stable_indirect]
#[rustc_nounwind]
#[rustc_intrinsic]
pub const unsafe fn unchecked_sub<T: Copy>(x: T, y: T) -> T;

/// Returns the result of an unchecked multiplication, resulting in
/// undefined behavior when `x * y > T::MAX` or `x * y < T::MIN`.
///
/// The stable counterpart of this intrinsic is `unchecked_mul` on the various
/// integer types, such as [`u16::unchecked_mul`] and [`i64::unchecked_mul`].
#[rustc_intrinsic_const_stable_indirect]
#[rustc_nounwind]
#[rustc_intrinsic]
pub const unsafe fn unchecked_mul<T: Copy>(x: T, y: T) -> T;

/// Performs rotate left.
///
/// Note that, unlike most intrinsics, this is safe to call;
/// it does not require an `unsafe` block.
/// Therefore, implementations must not require the user to uphold
/// any safety invariants.
///
/// The stabilized versions of this intrinsic are available on the integer
/// primitives via the `rotate_left` method. For example,
/// [`u32::rotate_left`]
#[rustc_intrinsic_const_stable_indirect]
#[rustc_nounwind]
#[rustc_intrinsic]
#[rustc_allow_const_fn_unstable(const_trait_impl, funnel_shifts)]
#[miri::intrinsic_fallback_is_spec]
pub const fn rotate_left<T: [const] fallback::FunnelShift>(x: T, shift: u32) -> T {
    // Make sure to call the intrinsic for `funnel_shl`, not the fallback impl.
    // SAFETY: we modulo `shift` so that the result is definitely less than the size of
    // `T` in bits.
    unsafe { unchecked_funnel_shl(x, x, shift % (mem::size_of::<T>() as u32 * 8)) }
}
```

### intrinsic #5: `checked_mul` -> `unchecked_mul`

- Source file: `rust/library/core/src/intrinsics/mod.rs`
- Dependent library / type: `core::intrinsic`
- Counterpart status: `unsafe_entry_only_or_unclear_pair`
- Safe function lines: `?`-`?`
- Unsafe function lines: `2008`-`2030`
- Safety condition: Not used directly, see library/core/src/num for details
- Checked-side note: No directly corresponding safe function
- User visible: handled
- Flag: Not recorded.

Safe source:
```rust
No directly corresponding safe function
```

Unsafe source:
```rust
pub const unsafe fn unchecked_mul<T: Copy>(x: T, y: T) -> T;

/// Performs rotate left.
///
/// Note that, unlike most intrinsics, this is safe to call;
/// it does not require an `unsafe` block.
/// Therefore, implementations must not require the user to uphold
/// any safety invariants.
///
/// The stabilized versions of this intrinsic are available on the integer
/// primitives via the `rotate_left` method. For example,
/// [`u32::rotate_left`]
#[rustc_intrinsic_const_stable_indirect]
#[rustc_nounwind]
#[rustc_intrinsic]
#[rustc_allow_const_fn_unstable(const_trait_impl, funnel_shifts)]
#[miri::intrinsic_fallback_is_spec]
pub const fn rotate_left<T: [const] fallback::FunnelShift>(x: T, shift: u32) -> T {
    // Make sure to call the intrinsic for `funnel_shl`, not the fallback impl.
    // SAFETY: we modulo `shift` so that the result is definitely less than the size of
    // `T` in bits.
    unsafe { unchecked_funnel_shl(x, x, shift % (mem::size_of::<T>() as u32 * 8)) }
}
```

### intrinsic #6: `checked_rem` -> `unchecked_rem`

- Source file: `rust/library/core/src/intrinsics/mod.rs`
- Dependent library / type: `core::intrinsic`
- Counterpart status: `unsafe_entry_only_or_unclear_pair`
- Safe function lines: `?`-`?`
- Unsafe function lines: `1957`-`2030`
- Safety condition: Not used directly, see library/core/src/num for details
- Checked-side note: No directly corresponding safe function
- User visible: handled
- Flag: Not recorded.

Safe source:
```rust
No directly corresponding safe function
```

Unsafe source:
```rust
pub const unsafe fn unchecked_rem<T: Copy>(x: T, y: T) -> T;

/// Performs an unchecked left shift, resulting in undefined behavior when
/// `y < 0` or `y >= N`, where N is the width of T in bits.
///
/// Safe wrappers for this intrinsic are available on the integer
/// primitives via the `checked_shl` method. For example,
/// [`u32::checked_shl`]
#[rustc_intrinsic_const_stable_indirect]
#[rustc_nounwind]
#[rustc_intrinsic]
pub const unsafe fn unchecked_shl<T: Copy, U: Copy>(x: T, y: U) -> T;
/// Performs an unchecked right shift, resulting in undefined behavior when
/// `y < 0` or `y >= N`, where N is the width of T in bits.
///
/// Safe wrappers for this intrinsic are available on the integer
/// primitives via the `checked_shr` method. For example,
/// [`u32::checked_shr`]
#[rustc_intrinsic_const_stable_indirect]
#[rustc_nounwind]
#[rustc_intrinsic]
pub const unsafe fn unchecked_shr<T: Copy, U: Copy>(x: T, y: U) -> T;

/// Returns the result of an unchecked addition, resulting in
/// undefined behavior when `x + y > T::MAX` or `x + y < T::MIN`.
///
/// The stable counterpart of this intrinsic is `unchecked_add` on the various
/// integer types, such as [`u16::unchecked_add`] and [`i64::unchecked_add`].
#[rustc_intrinsic_const_stable_indirect]
#[rustc_nounwind]
#[rustc_intrinsic]
pub const unsafe fn unchecked_add<T: Copy>(x: T, y: T) -> T;

/// Returns the result of an unchecked subtraction, resulting in
/// undefined behavior when `x - y > T::MAX` or `x - y < T::MIN`.
///
/// The stable counterpart of this intrinsic is `unchecked_sub` on the various
/// integer types, such as [`u16::unchecked_sub`] and [`i64::unchecked_sub`].
#[rustc_intrinsic_const_stable_indirect]
#[rustc_nounwind]
#[rustc_intrinsic]
pub const unsafe fn unchecked_sub<T: Copy>(x: T, y: T) -> T;

/// Returns the result of an unchecked multiplication, resulting in
/// undefined behavior when `x * y > T::MAX` or `x * y < T::MIN`.
///
/// The stable counterpart of this intrinsic is `unchecked_mul` on the various
/// integer types, such as [`u16::unchecked_mul`] and [`i64::unchecked_mul`].
#[rustc_intrinsic_const_stable_indirect]
#[rustc_nounwind]
#[rustc_intrinsic]
pub const unsafe fn unchecked_mul<T: Copy>(x: T, y: T) -> T;

/// Performs rotate left.
///
/// Note that, unlike most intrinsics, this is safe to call;
/// it does not require an `unsafe` block.
/// Therefore, implementations must not require the user to uphold
/// any safety invariants.
///
/// The stabilized versions of this intrinsic are available on the integer
/// primitives via the `rotate_left` method. For example,
/// [`u32::rotate_left`]
#[rustc_intrinsic_const_stable_indirect]
#[rustc_nounwind]
#[rustc_intrinsic]
#[rustc_allow_const_fn_unstable(const_trait_impl, funnel_shifts)]
#[miri::intrinsic_fallback_is_spec]
pub const fn rotate_left<T: [const] fallback::FunnelShift>(x: T, shift: u32) -> T {
    // Make sure to call the intrinsic for `funnel_shl`, not the fallback impl.
    // SAFETY: we modulo `shift` so that the result is definitely less than the size of
    // `T` in bits.
    unsafe { unchecked_funnel_shl(x, x, shift % (mem::size_of::<T>() as u32 * 8)) }
}
```

### intrinsic #7: `checked_shl` -> `unchecked_shl`

- Source file: `rust/library/core/src/intrinsics/mod.rs`
- Dependent library / type: `core::intrinsic`
- Counterpart status: `unsafe_entry_only_or_unclear_pair`
- Safe function lines: `?`-`?`
- Unsafe function lines: `1968`-`2030`
- Safety condition: Not used directly, see library/core/src/num for details
- Checked-side note: No directly corresponding safe function
- User visible: handled
- Flag: Not recorded.

Safe source:
```rust
No directly corresponding safe function
```

Unsafe source:
```rust
pub const unsafe fn unchecked_shl<T: Copy, U: Copy>(x: T, y: U) -> T;
/// Performs an unchecked right shift, resulting in undefined behavior when
/// `y < 0` or `y >= N`, where N is the width of T in bits.
///
/// Safe wrappers for this intrinsic are available on the integer
/// primitives via the `checked_shr` method. For example,
/// [`u32::checked_shr`]
#[rustc_intrinsic_const_stable_indirect]
#[rustc_nounwind]
#[rustc_intrinsic]
pub const unsafe fn unchecked_shr<T: Copy, U: Copy>(x: T, y: U) -> T;

/// Returns the result of an unchecked addition, resulting in
/// undefined behavior when `x + y > T::MAX` or `x + y < T::MIN`.
///
/// The stable counterpart of this intrinsic is `unchecked_add` on the various
/// integer types, such as [`u16::unchecked_add`] and [`i64::unchecked_add`].
#[rustc_intrinsic_const_stable_indirect]
#[rustc_nounwind]
#[rustc_intrinsic]
pub const unsafe fn unchecked_add<T: Copy>(x: T, y: T) -> T;

/// Returns the result of an unchecked subtraction, resulting in
/// undefined behavior when `x - y > T::MAX` or `x - y < T::MIN`.
///
/// The stable counterpart of this intrinsic is `unchecked_sub` on the various
/// integer types, such as [`u16::unchecked_sub`] and [`i64::unchecked_sub`].
#[rustc_intrinsic_const_stable_indirect]
#[rustc_nounwind]
#[rustc_intrinsic]
pub const unsafe fn unchecked_sub<T: Copy>(x: T, y: T) -> T;

/// Returns the result of an unchecked multiplication, resulting in
/// undefined behavior when `x * y > T::MAX` or `x * y < T::MIN`.
///
/// The stable counterpart of this intrinsic is `unchecked_mul` on the various
/// integer types, such as [`u16::unchecked_mul`] and [`i64::unchecked_mul`].
#[rustc_intrinsic_const_stable_indirect]
#[rustc_nounwind]
#[rustc_intrinsic]
pub const unsafe fn unchecked_mul<T: Copy>(x: T, y: T) -> T;

/// Performs rotate left.
///
/// Note that, unlike most intrinsics, this is safe to call;
/// it does not require an `unsafe` block.
/// Therefore, implementations must not require the user to uphold
/// any safety invariants.
///
/// The stabilized versions of this intrinsic are available on the integer
/// primitives via the `rotate_left` method. For example,
/// [`u32::rotate_left`]
#[rustc_intrinsic_const_stable_indirect]
#[rustc_nounwind]
#[rustc_intrinsic]
#[rustc_allow_const_fn_unstable(const_trait_impl, funnel_shifts)]
#[miri::intrinsic_fallback_is_spec]
pub const fn rotate_left<T: [const] fallback::FunnelShift>(x: T, shift: u32) -> T {
    // Make sure to call the intrinsic for `funnel_shl`, not the fallback impl.
    // SAFETY: we modulo `shift` so that the result is definitely less than the size of
    // `T` in bits.
    unsafe { unchecked_funnel_shl(x, x, shift % (mem::size_of::<T>() as u32 * 8)) }
}
```

### intrinsic #8: `checked_shr` -> `unchecked_shr`

- Source file: `rust/library/core/src/intrinsics/mod.rs`
- Dependent library / type: `core::intrinsic`
- Counterpart status: `unsafe_entry_only_or_unclear_pair`
- Safe function lines: `?`-`?`
- Unsafe function lines: `1978`-`2030`
- Safety condition: Not used directly, see library/core/src/num for details
- Checked-side note: No directly corresponding safe function
- User visible: handled
- Flag: Not recorded.

Safe source:
```rust
No directly corresponding safe function
```

Unsafe source:
```rust
pub const unsafe fn unchecked_shr<T: Copy, U: Copy>(x: T, y: U) -> T;

/// Returns the result of an unchecked addition, resulting in
/// undefined behavior when `x + y > T::MAX` or `x + y < T::MIN`.
///
/// The stable counterpart of this intrinsic is `unchecked_add` on the various
/// integer types, such as [`u16::unchecked_add`] and [`i64::unchecked_add`].
#[rustc_intrinsic_const_stable_indirect]
#[rustc_nounwind]
#[rustc_intrinsic]
pub const unsafe fn unchecked_add<T: Copy>(x: T, y: T) -> T;

/// Returns the result of an unchecked subtraction, resulting in
/// undefined behavior when `x - y > T::MAX` or `x - y < T::MIN`.
///
/// The stable counterpart of this intrinsic is `unchecked_sub` on the various
/// integer types, such as [`u16::unchecked_sub`] and [`i64::unchecked_sub`].
#[rustc_intrinsic_const_stable_indirect]
#[rustc_nounwind]
#[rustc_intrinsic]
pub const unsafe fn unchecked_sub<T: Copy>(x: T, y: T) -> T;

/// Returns the result of an unchecked multiplication, resulting in
/// undefined behavior when `x * y > T::MAX` or `x * y < T::MIN`.
///
/// The stable counterpart of this intrinsic is `unchecked_mul` on the various
/// integer types, such as [`u16::unchecked_mul`] and [`i64::unchecked_mul`].
#[rustc_intrinsic_const_stable_indirect]
#[rustc_nounwind]
#[rustc_intrinsic]
pub const unsafe fn unchecked_mul<T: Copy>(x: T, y: T) -> T;

/// Performs rotate left.
///
/// Note that, unlike most intrinsics, this is safe to call;
/// it does not require an `unsafe` block.
/// Therefore, implementations must not require the user to uphold
/// any safety invariants.
///
/// The stabilized versions of this intrinsic are available on the integer
/// primitives via the `rotate_left` method. For example,
/// [`u32::rotate_left`]
#[rustc_intrinsic_const_stable_indirect]
#[rustc_nounwind]
#[rustc_intrinsic]
#[rustc_allow_const_fn_unstable(const_trait_impl, funnel_shifts)]
#[miri::intrinsic_fallback_is_spec]
pub const fn rotate_left<T: [const] fallback::FunnelShift>(x: T, shift: u32) -> T {
    // Make sure to call the intrinsic for `funnel_shl`, not the fallback impl.
    // SAFETY: we modulo `shift` so that the result is definitely less than the size of
    // `T` in bits.
    unsafe { unchecked_funnel_shl(x, x, shift % (mem::size_of::<T>() as u32 * 8)) }
}
```

### intrinsic #9: `checked_sub` -> `unchecked_sub`

- Source file: `rust/library/core/src/intrinsics/mod.rs`
- Dependent library / type: `core::intrinsic`
- Counterpart status: `unsafe_entry_only_or_unclear_pair`
- Safe function lines: `?`-`?`
- Unsafe function lines: `1998`-`2030`
- Safety condition: Not used directly, see library/core/src/num for details
- Checked-side note: No directly corresponding safe function
- User visible: handled
- Flag: Not recorded.

Safe source:
```rust
No directly corresponding safe function
```

Unsafe source:
```rust
pub const unsafe fn unchecked_sub<T: Copy>(x: T, y: T) -> T;

/// Returns the result of an unchecked multiplication, resulting in
/// undefined behavior when `x * y > T::MAX` or `x * y < T::MIN`.
///
/// The stable counterpart of this intrinsic is `unchecked_mul` on the various
/// integer types, such as [`u16::unchecked_mul`] and [`i64::unchecked_mul`].
#[rustc_intrinsic_const_stable_indirect]
#[rustc_nounwind]
#[rustc_intrinsic]
pub const unsafe fn unchecked_mul<T: Copy>(x: T, y: T) -> T;

/// Performs rotate left.
///
/// Note that, unlike most intrinsics, this is safe to call;
/// it does not require an `unsafe` block.
/// Therefore, implementations must not require the user to uphold
/// any safety invariants.
///
/// The stabilized versions of this intrinsic are available on the integer
/// primitives via the `rotate_left` method. For example,
/// [`u32::rotate_left`]
#[rustc_intrinsic_const_stable_indirect]
#[rustc_nounwind]
#[rustc_intrinsic]
#[rustc_allow_const_fn_unstable(const_trait_impl, funnel_shifts)]
#[miri::intrinsic_fallback_is_spec]
pub const fn rotate_left<T: [const] fallback::FunnelShift>(x: T, shift: u32) -> T {
    // Make sure to call the intrinsic for `funnel_shl`, not the fallback impl.
    // SAFETY: we modulo `shift` so that the result is definitely less than the size of
    // `T` in bits.
    unsafe { unchecked_funnel_shl(x, x, shift % (mem::size_of::<T>() as u32 * 8)) }
}
```

### intrinsic #10: `transmute` -> `transmute_unchecked`

- Source file: `rust/library/core/src/intrinsics/mod.rs`
- Dependent library / type: `core::intrinsic`
- Counterpart status: `unsafe_entry_only_or_unclear_pair`
- Safe function lines: `?`-`?`
- Unsafe function lines: `855`-`923`
- Safety condition: Fundamentally unsafe, handle separately.
- Checked-side note: Not recorded.
- User visible: handled
- Flag: Not recorded.

Safe source:
```rust
pub const unsafe fn transmute<Src, Dst>(src: Src) -> Dst;
```

Unsafe source:
```rust
pub const unsafe fn transmute_unchecked<Src, Dst>(src: Src) -> Dst;

/// Returns `true` if the actual type given as `T` requires drop
/// glue; returns `false` if the actual type provided for `T`
/// implements `Copy`.
///
/// If the actual type neither requires drop glue nor implements
/// `Copy`, then the return value of this function is unspecified.
///
/// Note that, unlike most intrinsics, this can only be called at compile-time
/// as backends do not have an implementation for it. The only caller (its
/// stable counterpart) wraps this intrinsic call in a `const` block so that
/// backends only see an evaluated constant.
///
/// The stabilized version of this intrinsic is [`mem::needs_drop`](crate::mem::needs_drop).
#[rustc_intrinsic_const_stable_indirect]
#[rustc_nounwind]
#[rustc_intrinsic]
pub const fn needs_drop<T: ?Sized>() -> bool;

/// Calculates the offset from a pointer.
///
/// This is implemented as an intrinsic to avoid converting to and from an
/// integer, since the conversion would throw away aliasing information.
///
/// This can only be used with `Ptr` as a raw pointer type (`*mut` or `*const`)
/// to a `Sized` pointee and with `Delta` as `usize` or `isize`.  Any other
/// instantiations may arbitrarily misbehave, and that's *not* a compiler bug.
///
/// # Safety
///
/// If the computed offset is non-zero, then both the starting and resulting pointer must be
/// either in bounds or at the end of an allocation. If either pointer is out
/// of bounds or arithmetic overflow occurs then this operation is undefined behavior.
///
/// The stabilized version of this intrinsic is [`pointer::offset`].
#[must_use = "returns a new pointer rather than modifying its argument"]
#[rustc_intrinsic_const_stable_indirect]
#[rustc_nounwind]
#[rustc_intrinsic]
pub const unsafe fn offset<Ptr: bounds::BuiltinDeref, Delta>(dst: Ptr, offset: Delta) -> Ptr;

/// Calculates the offset from a pointer, potentially wrapping.
///
/// This is implemented as an intrinsic to avoid converting to and from an
/// integer, since the conversion inhibits certain optimizations.
///
/// # Safety
///
/// Unlike the `offset` intrinsic, this intrinsic does not restrict the
/// resulting pointer to point into or at the end of an allocated
/// object, and it wraps with two's complement arithmetic. The resulting
/// value is not necessarily valid to be used to actually access memory.
///
/// The stabilized version of this intrinsic is [`pointer::wrapping_offset`].
#[must_use = "returns a new pointer rather than modifying its argument"]
#[rustc_intrinsic_const_stable_indirect]
#[rustc_nounwind]
#[rustc_intrinsic]
pub const unsafe fn arith_offset<T>(dst: *const T, offset: isize) -> *const T;

/// Projects to the `index`-th element of `slice_ptr`, as the same kind of pointer
/// as the slice was provided -- so `&mut [T] → &mut T`, `&[T] → &T`,
/// `*mut [T] → *mut T`, or `*const [T] → *const T` -- without a bounds check.
///
/// This is exposed via `<usize as SliceIndex>::get(_unchecked)(_mut)`,
/// and isn't intended to be used elsewhere.
///
/// Expands in MIR to `{&, &mut, &raw const, &raw mut} (*slice_ptr)[index]`,
```

## Category: `itron`

### itron #1: `get` -> `get_unchecked`

- Source file: `rust/library/std/src/sys/pal/itron/spin.rs`
- Dependent library / type: `itron`
- Counterpart status: `confirmed_pair`
- Safe function lines: `71`-`76`
- Unsafe function lines: `87`-`91`
- Safety condition: RTOS的自旋锁相关, 暂不了解
- Checked-side note: Not recorded.
- User visible: Not recorded.
- Flag: Not recorded.

Safe source:
```rust
    pub fn get(&self) -> Option<(abi::ID, &T)> {
        match self.id.load(Ordering::Acquire) {
            ID_UNINIT => None,
            id => Some((id as abi::ID, unsafe { (&*self.extra.get()).assume_init_ref() })),
        }
    }
```

Unsafe source:
```rust
    pub unsafe fn get_unchecked(&self) -> (abi::ID, &T) {
        (self.id.load(Ordering::Acquire) as abi::ID, unsafe {
            (&*self.extra.get()).assume_init_ref()
        })
    }
```

### itron #2: `set` -> `set_unchecked`

- Source file: `rust/library/std/src/sys/pal/itron/spin.rs`
- Dependent library / type: `itron`
- Counterpart status: `unsafe_entry_only_or_unclear_pair`
- Safe function lines: `?`-`?`
- Unsafe function lines: `95`-`105`
- Safety condition: Not recorded.
- Checked-side note: 未找到相关safe
- User visible: Not recorded.
- Flag: Not recorded.

Safe source:
```rust
未找到相关safe
```

Unsafe source:
```rust
    pub unsafe fn set_unchecked(&self, (id, extra): (abi::ID, T)) {
        debug_assert!(self.get().is_none());

        // Assumption: A positive `abi::ID` fits in `usize`.
        debug_assert!(id >= 0);
        debug_assert!(usize::try_from(id).is_ok());
        let id = id as usize;

        unsafe { *self.extra.get() = MaybeUninit::new(extra) };
        self.id.store(id, Ordering::Release);
    }
```

## Category: `nonzero`

### nonzero #1: `new` -> `new_unchecked`

- Source file: `rust/library/core/src/num/niche_types.rs`
- Dependent library / type: `algorithm`
- Counterpart status: `confirmed_pair`
- Safe function lines: `23`-`31`
- Unsafe function lines: `40`-`43`
- Safety condition: 确保参数nonzero
- Checked-side note: Not recorded.
- User visible: Not recorded.
- Flag: Not recorded.

Safe source:
```rust
            pub const fn new(val: $int) -> Option<Self> {
                #[allow(non_contiguous_range_endpoints)]
                if let $pat = val {
                    // SAFETY: just checked that the value matches the pattern
                    Some(unsafe { $name(crate::mem::transmute(val)) })
                } else {
                    None
                }
            }
```

Unsafe source:
```rust
            pub const unsafe fn new_unchecked(val: $int) -> Self {
                // SAFETY: Caller promised that `val` is within the valid range.
                unsafe { crate::mem::transmute(val) }
            }
```

## Category: `nullptr`

### nullptr #1: `as_ref` -> `as_ref_unchecked`

- Source file: `rust/library/core/src/ptr/const_ptr.rs`
- Dependent library / type: `ptr`
- Counterpart status: `unsafe_entry_only_or_unclear_pair`
- Safe function lines: `?`-`?`
- Unsafe function lines: `297`-`300`
- Safety condition: 必须保证self是合法的, 不是空指针
- Checked-side note: Not recorded.
- User visible: Not recorded.
- Flag: Not recorded.

Safe source:
```rust
pub const unsafe fn as_ref<'a>(self) -> Option<&'a T> {
        // SAFETY: the caller must guarantee that `self` is valid
        // for a reference if it isn't null.
        if self.is_null() { None } else { unsafe { Some(&*self) } }
    }
```

Unsafe source:
```rust
    pub const unsafe fn as_ref_unchecked<'a>(self) -> &'a T {
        // SAFETY: the caller must guarantee that `self` is valid for a reference
        unsafe { &*self }
    }
```

### nullptr #2: `as_ref` -> `as_mut_unchecked`

- Source file: `rust/library/core/src/ptr/mut_ptr.rs`
- Dependent library / type: `ptr`
- Counterpart status: `unsafe_entry_only_or_unclear_pair`
- Safe function lines: `?`-`?`
- Unsafe function lines: `618`-`621`
- Safety condition: 必须保证参数ptr是非空的
- Checked-side note: Not recorded.
- User visible: Not recorded.
- Flag: Not recorded.

Safe source:
```rust
pub const unsafe fn as_ref<'a>(self) -> Option<&'a T> {
        // SAFETY: the caller must guarantee that `self` is valid for a
        // reference if it isn't null.
        if self.is_null() { None } else { unsafe { Some(&*self) } }
    }
```

Unsafe source:
```rust
    pub const unsafe fn as_mut_unchecked<'a>(self) -> &'a mut T {
        // SAFETY: the caller must guarantee that `self` is valid for a reference
        unsafe { &mut *self }
    }
```

### nullptr #3: `as_ref` -> `as_ref_unchecked`

- Source file: `rust/library/core/src/ptr/mut_ptr.rs`
- Dependent library / type: `ptr`
- Counterpart status: `unsafe_entry_only_or_unclear_pair`
- Safe function lines: `?`-`?`
- Unsafe function lines: `295`-`298`
- Safety condition: 必须保证参数ptr是非空的
- Checked-side note: Not recorded.
- User visible: Not recorded.
- Flag: Not recorded.

Safe source:
```rust
pub const unsafe fn as_ref<'a>(self) -> Option<&'a T> {
        // SAFETY: the caller must guarantee that `self` is valid for a
        // reference if it isn't null.
        if self.is_null() { None } else { unsafe { Some(&*self) } }
    }
```

Unsafe source:
```rust
    pub const unsafe fn as_ref_unchecked<'a>(self) -> &'a T {
        // SAFETY: the caller must guarantee that `self` is valid for a reference
        unsafe { &*self }
    }
```

### nullptr #4: `new` -> `new_unchecked`

- Source file: `rust/library/core/src/ptr/non_null.rs`
- Dependent library / type: `ptr`
- Counterpart status: `confirmed_pair`
- Safe function lines: `269`-`276`
- Unsafe function lines: `233`-`243`
- Safety condition: 用户需要检查是否为空指针
- Checked-side note: Not recorded.
- User visible: Not recorded.
- Flag: Not recorded.

Safe source:
```rust
    pub const fn new(ptr: *mut T) -> Option<Self> {
        if !ptr.is_null() {
            // SAFETY: The pointer is already checked and is not null
            Some(unsafe { Self::new_unchecked(ptr) })
        } else {
            None
        }
    }
```

Unsafe source:
```rust
    pub const unsafe fn new_unchecked(ptr: *mut T) -> Self {
        // SAFETY: the caller must guarantee that `ptr` is non-null.
        unsafe {
            assert_unsafe_precondition!(
                check_language_ub,
                "NonNull::new_unchecked requires that the pointer is non-null",
                (ptr: *mut () = ptr as *mut ()) => !ptr.is_null()
            );
            transmute(ptr)
        }
    }
```

### nullptr #5: `new` -> `new_unchecked`

- Source file: `rust/library/core/src/ptr/unique.rs`
- Dependent library / type: `ptr`
- Counterpart status: `confirmed_pair`
- Safe function lines: `93`-`99`
- Unsafe function lines: `86`-`89`
- Safety condition: 必须保证参数ptr是非空的
- Checked-side note: Not recorded.
- User visible: Not recorded.
- Flag: Not recorded.

Safe source:
```rust
    pub const fn new(ptr: *mut T) -> Option<Self> {
        if let Some(pointer) = NonNull::new(ptr) {
            Some(Unique { pointer, _marker: PhantomData })
        } else {
            None
        }
    }
```

Unsafe source:
```rust
    pub const unsafe fn new_unchecked(ptr: *mut T) -> Self {
        // SAFETY: the caller must guarantee that `ptr` is non-null.
        unsafe { Unique { pointer: NonNull::new_unchecked(ptr), _marker: PhantomData } }
    }
```

## Category: `overflow`

### overflow #1: `checked_add` -> `unchecked_add`

- Source file: `rust/library/core/src/num/int_macros.rs`
- Dependent library / type: `algorithm`
- Counterpart status: `confirmed_pair`
- Safe function lines: `523`-`526`
- Unsafe function lines: `583`-`597`
- Safety condition: 必须保证不会溢出, 参考intrinsic::unchecked_add
- Checked-side note: Not recorded.
- User visible: Not recorded.
- Flag: Not recorded.

Safe source:
```rust
        pub const fn checked_add(self, rhs: Self) -> Option<Self> {
            let (a, b) = self.overflowing_add(rhs);
            if intrinsics::unlikely(b) { None } else { Some(a) }
        }
```

Unsafe source:
```rust
        pub const unsafe fn unchecked_add(self, rhs: Self) -> Self {
            assert_unsafe_precondition!(
                check_language_ub,
                concat!(stringify!($SelfT), "::unchecked_add cannot overflow"),
                (
                    lhs: $SelfT = self,
                    rhs: $SelfT = rhs,
                ) => !lhs.overflowing_add(rhs).1,
            );

            // SAFETY: this is guaranteed to be safe by the caller.
            unsafe {
                intrinsics::unchecked_add(self, rhs)
            }
        }
```

## Category: `overflow/nonzero`

### overflow/nonzero #1: `checked_add` -> `unchecked_add`

- Source file: `rust/library/core/src/num/nonzero.rs`
- Dependent library / type: `num/nonzero`
- Counterpart status: `confirmed_pair`
- Safe function lines: `1646`-`1659`
- Unsafe function lines: `1723`-`1726`
- Safety condition: 必须保证参数参数非零并且不会溢出
- Checked-side note: Not recorded.
- User visible: Not recorded.
- Flag: Not recorded.

Safe source:
```rust
        pub const fn checked_add(self, other: $Int) -> Option<Self> {
            if let Some(result) = self.get().checked_add(other) {
                // SAFETY:
                // - `checked_add` returns `None` on overflow
                // - `self` is non-zero
                // - the only way to get zero from an addition without overflow is for both
                //   sides to be zero
                //
                // So the result cannot be zero.
                Some(unsafe { Self::new_unchecked(result) })
            } else {
                None
            }
        }
```

Unsafe source:
```rust
        pub const unsafe fn unchecked_add(self, other: $Int) -> Self {
            // SAFETY: The caller ensures there is no overflow.
            unsafe { Self::new_unchecked(self.get().unchecked_add(other)) }
        }
```

### overflow/nonzero #2: `checked_mul` -> `unchecked_mul`

- Source file: `rust/library/core/src/num/nonzero.rs`
- Dependent library / type: `num/nonzero`
- Counterpart status: `confirmed_pair`
- Safe function lines: `1068`-`1081`
- Unsafe function lines: `1155`-`1158`
- Safety condition: 必须保证参数非零并且不会溢出
- Checked-side note: Not recorded.
- User visible: Not recorded.
- Flag: Not recorded.

Safe source:
```rust
            pub const fn checked_mul(self, other: Self) -> Option<Self> {
                if let Some(result) = self.get().checked_mul(other.get()) {
                    // SAFETY:
                    // - `checked_mul` returns `None` on overflow
                    // - `self` and `other` are non-zero
                    // - the only way to get zero from a multiplication without overflow is for one
                    //   of the sides to be zero
                    //
                    // So the result cannot be zero.
                    Some(unsafe { Self::new_unchecked(result) })
                } else {
                    None
                }
            }
```

Unsafe source:
```rust
            pub const unsafe fn unchecked_mul(self, other: Self) -> Self {
                // SAFETY: The caller ensures there is no overflow.
                unsafe { Self::new_unchecked(self.get().unchecked_mul(other.get())) }
            }
```

## Category: `pin`

### pin #1: `get_mut` -> `get_unchecked_mut`

- Source file: `rust/library/core/src/pin.rs`
- Dependent library / type: `pin`
- Counterpart status: `confirmed_pair`
- Safe function lines: `1596`-`1601`
- Unsafe function lines: `1617`-`1619`
- Safety condition: 需要用户保证不会将数据移动出这个位置?
/// This function is unsafe. You must guarantee that you will never move
/// the data out of the mutable reference you receive when you call this
/// function, so that the invariants on the `Pin` type can be upheld.
- Checked-side note: Not recorded.
- User visible: Not recorded.
- Flag: Not recorded.

Safe source:
```rust
    pub const fn get_mut(self) -> &'a mut T
    where
        T: Unpin,
    {
        self.pointer
    }
```

Unsafe source:
```rust
    pub const unsafe fn get_unchecked_mut(self) -> &'a mut T {
        self.pointer
    }
```

### pin #2: `into_inner` -> `into_inner_unchecked`

- Source file: `rust/library/core/src/pin.rs`
- Dependent library / type: `pin`
- Counterpart status: `confirmed_pair`
- Safe function lines: `1210`-`1212`
- Unsafe function lines: `1512`-`1514`
- Safety condition: 疑似没有核心的问题
- Checked-side note: Not recorded.
- User visible: Not recorded.
- Flag: Not recorded.

Safe source:
```rust
    pub const fn into_inner(pin: Pin<Ptr>) -> Ptr {
        pin.pointer
    }
```

Unsafe source:
```rust
    pub const unsafe fn into_inner_unchecked(pin: Pin<Ptr>) -> Ptr {
        pin.pointer
    }
```

### pin #3: `new` -> `new_unchecked`

- Source file: `rust/library/core/src/pin.rs`
- Dependent library / type: `pin`
- Counterpart status: `confirmed_pair`
- Safe function lines: `430`-`455`
- Unsafe function lines: `1347`-`1349`
- Safety condition: 疑似没有核心的问题
- Checked-side note: Not recorded.
- User visible: Not recorded.
- Flag: Not recorded.

Safe source:
```rust
//!     fn new() -> Pin<Box<Self>> {
//!         let res = Unmovable {
//!             data: [0; 64],
//!             // We only create the pointer once the data is in place
//!             // otherwise it will have already moved before we even started.
//!             slice: NonNull::from(&[]),
//!             _pin: PhantomPinned,
//!         };
//!         // First we put the data in a box, which will be its final resting place
//!         let mut boxed = Box::new(res);
//!
//!         // Then we make the slice field point to the proper part of that boxed data.
//!         // From now on we need to make sure we don't move the boxed data.
//!         boxed.slice = NonNull::from(&boxed.data);
//!
//!         // To do that, we pin the data in place by pointing to it with a pinning
//!         // (`Pin`-wrapped) pointer.
//!         //
//!         // `Box::into_pin` makes existing `Box` pin the data in-place without moving it,
//!         // so we can safely do this now *after* inserting the slice pointer above, but we have
//!         // to take care that we haven't performed any other semantic moves of `res` in between.
//!         let pin = Box::into_pin(boxed);
//!
//!         // Now we can return the pinned (through a pinning Box) data
//!         pin
//!     }
```

Unsafe source:
```rust
    pub const unsafe fn new_unchecked(pointer: Ptr) -> Pin<Ptr> {
        Pin { pointer }
    }
```

## Category: `Result/Option`

### Result/Option #1: `unwrap` -> `unwrap_unchecked`

- Source file: `rust/library/core/src/result.rs`
- Dependent library / type: `Option/Result`
- Counterpart status: `confirmed_pair`
- Safe function lines: `1227`-`1235`
- Unsafe function lines: `1650`-`1660`
- Safety condition: 枚举类型必须得是安全的
- Checked-side note: unwrap及其一系列
- User visible: Not recorded.
- Flag: Not recorded.

Safe source:
```rust
    pub fn unwrap(self) -> T
    where
        E: fmt::Debug,
    {
        match self {
            Ok(t) => t,
            Err(e) => unwrap_failed("called `Result::unwrap()` on an `Err` value", &e),
        }
    }
```

Unsafe source:
```rust
    pub const unsafe fn unwrap_unchecked(self) -> T {
        match self {
            Ok(t) => t,
            Err(e) => {
                // FIXME(const-hack): to avoid E: const Destruct bound
                super::mem::forget(e);
                // SAFETY: the safety contract must be upheld by the caller.
                unsafe { hint::unreachable_unchecked() }
            }
        }
    }
```

### Result/Option #2: `unwrap_err` -> `unwrap_err_unchecked`

- Source file: `rust/library/core/src/result.rs`
- Dependent library / type: `Option/Result`
- Counterpart status: `confirmed_pair`
- Safe function lines: `1324`-`1332`
- Unsafe function lines: `1685`-`1691`
- Safety condition: 枚举类型必须得是安全的
- Checked-side note: unwrap及其一系列
- User visible: Not recorded.
- Flag: Not recorded.

Safe source:
```rust
    pub fn unwrap_err(self) -> E
    where
        T: fmt::Debug,
    {
        match self {
            Ok(t) => unwrap_failed("called `Result::unwrap_err()` on an `Ok` value", &t),
            Err(e) => e,
        }
    }
```

Unsafe source:
```rust
    pub unsafe fn unwrap_err_unchecked(self) -> E {
        match self {
            // SAFETY: the safety contract must be upheld by the caller.
            Ok(_) => unsafe { hint::unreachable_unchecked() },
            Err(e) => e,
        }
    }
```

## Category: `Reuslt/Option`

### Reuslt/Option #1: `unwrap` -> `unwrap_unchecked`

- Source file: `rust/library/core/src/option.rs`
- Dependent library / type: `Option/Result`
- Counterpart status: `confirmed_pair`
- Safe function lines: `1013`-`1018`
- Unsafe function lines: `1128`-`1134`
- Safety condition: Option类型为Some而不是None
- Checked-side note: Not recorded.
- User visible: Not recorded.
- Flag: Not recorded.

Safe source:
```rust
    pub const fn unwrap(self) -> T {
        match self {
            Some(val) => val,
            None => unwrap_failed(),
        }
    }
```

Unsafe source:
```rust
    pub const unsafe fn unwrap_unchecked(self) -> T {
        match self {
            Some(val) => val,
            // SAFETY: the safety contract must be upheld by the caller.
            None => unsafe { hint::unreachable_unchecked() },
        }
    }
```

## Category: `slice/buf`

### slice/buf #1: `from_encoded_bytes` -> `from_encoded_bytes_unchecked`

- Source file: `rust/library/std/src/sys/os_str/bytes.rs`
- Dependent library / type: `os_str/slice`
- Counterpart status: `unsafe_entry_only_or_unclear_pair`
- Safe function lines: `?`-`?`
- Unsafe function lines: `90`-`92`
- Safety condition: Not recorded.
- Checked-side note: 未找到相关safe
- User visible: Not recorded.
- Flag: Not recorded.

Safe source:
```rust
未找到相关safe
```

Unsafe source:
```rust
    pub unsafe fn from_encoded_bytes_unchecked(s: Vec<u8>) -> Self {
        Self { inner: s }
    }
```

## Category: `String`

### String #1: `from_utf8` -> `from_utf8_unchecked`

- Source file: `rust/library/core/src/str/mod.rs`
- Dependent library / type: `null`
- Counterpart status: `confirmed_pair`
- Safe function lines: `251`-`253`
- Unsafe function lines: `316`-`319`
- Safety condition: 同14行
- Checked-side note: Not recorded.
- User visible: Not recorded.
- Flag: Not recorded.

Safe source:
```rust
    pub const fn from_utf8(v: &[u8]) -> Result<&str, Utf8Error> {
        converts::from_utf8(v)
    }
```

Unsafe source:
```rust
    pub const unsafe fn from_utf8_unchecked(v: &[u8]) -> &str {
        // SAFETY: converts::from_utf8_unchecked has the same safety requirements as this function.
        unsafe { converts::from_utf8_unchecked(v) }
    }
```

### String #2: `from_utf8_mut` -> `from_utf8_unchecked_mut`

- Source file: `rust/library/core/src/str/mod.rs`
- Dependent library / type: `null`
- Counterpart status: `confirmed_pair`
- Safe function lines: `284`-`286`
- Unsafe function lines: `341`-`344`
- Safety condition: 同15行
- Checked-side note: Not recorded.
- User visible: Not recorded.
- Flag: Not recorded.

Safe source:
```rust
    pub const fn from_utf8_mut(v: &mut [u8]) -> Result<&mut str, Utf8Error> {
        converts::from_utf8_mut(v)
    }
```

Unsafe source:
```rust
    pub const unsafe fn from_utf8_unchecked_mut(v: &mut [u8]) -> &mut str {
        // SAFETY: converts::from_utf8_unchecked_mut has the same safety requirements as this function.
        unsafe { converts::from_utf8_unchecked_mut(v) }
    }
```

### String #3: `from_utf8` -> `from_utf8_unchecked`

- Source file: `rust/library/core/src/str/converts.rs`
- Dependent library / type: `str`
- Counterpart status: `confirmed_pair`
- Safe function lines: `89`-`98`
- Unsafe function lines: `178`-`182`
- Safety condition: 必须保证str数组中的每一个字节都是UTF-8类型
- Checked-side note: Not recorded.
- User visible: Not recorded.
- Flag: Not recorded.

Safe source:
```rust
pub const fn from_utf8(v: &[u8]) -> Result<&str, Utf8Error> {
    // FIXME(const-hack): This should use `?` again, once it's `const`
    match run_utf8_validation(v) {
        Ok(_) => {
            // SAFETY: validation succeeded.
            Ok(unsafe { from_utf8_unchecked(v) })
        }
        Err(err) => Err(err),
    }
}
```

Unsafe source:
```rust
pub const unsafe fn from_utf8_unchecked(v: &[u8]) -> &str {
    // SAFETY: the caller must guarantee that the bytes `v` are valid UTF-8.
    // Also relies on `&str` and `&[u8]` having the same layout.
    unsafe { mem::transmute(v) }
}
```

### String #4: `from_utf8_mut` -> `from_utf8_unchecked_mut`

- Source file: `rust/library/core/src/str/converts.rs`
- Dependent library / type: `str`
- Counterpart status: `confirmed_pair`
- Safe function lines: `135`-`144`
- Unsafe function lines: `208`-`214`
- Safety condition: 必须保证str数组中的每一个字节都是UTF-8类型
- Checked-side note: Not recorded.
- User visible: Not recorded.
- Flag: Not recorded.

Safe source:
```rust
pub const fn from_utf8_mut(v: &mut [u8]) -> Result<&mut str, Utf8Error> {
    // FIXME(const-hack): This should use `?` again, once it's `const`
    match run_utf8_validation(v) {
        Ok(_) => {
            // SAFETY: validation succeeded.
            Ok(unsafe { from_utf8_unchecked_mut(v) })
        }
        Err(err) => Err(err),
    }
}
```

Unsafe source:
```rust
pub const unsafe fn from_utf8_unchecked_mut(v: &mut [u8]) -> &mut str {
    // SAFETY: the caller must guarantee that the bytes `v`
    // are valid UTF-8, thus the cast to `*mut str` is safe.
    // Also, the pointer dereference is safe because that pointer
    // comes from a reference which is guaranteed to be valid for writes.
    unsafe { &mut *(v as *mut [u8] as *mut str) }
}
```

## Category: `sync`

### sync #1: `get_or_init` -> `get_unchecked`

- Source file: `rust/library/std/src/sys/sync/once_box.rs`
- Dependent library / type: `sync`
- Counterpart status: `confirmed_pair`
- Safe function lines: `47`-`53`
- Unsafe function lines: `42`-`44`
- Safety condition: Not recorded.
- Checked-side note: Not recorded.
- User visible: Not recorded.
- Flag: Not recorded.

Safe source:
```rust
    pub fn get_or_init(&self, f: impl FnOnce() -> Pin<Box<T>>) -> Pin<&T> {
        let ptr = self.ptr.load(Acquire);
        match unsafe { ptr.as_ref() } {
            Some(val) => unsafe { Pin::new_unchecked(val) },
            None => self.initialize(f),
        }
    }
```

Unsafe source:
```rust
    pub unsafe fn get_unchecked(&self) -> Pin<&T> {
        unsafe { Pin::new_unchecked(&*self.ptr.load(Relaxed)) }
    }
```

## Category: `thread`

### thread #1: `spawn` -> `spawn_unchecked`

- Source file: `rust/library/std/src/thread/mod.rs`
- Dependent library / type: `thread`
- Counterpart status: `unsafe_entry_only_or_unclear_pair`
- Safe function lines: `?`-`?`
- Unsafe function lines: `461`-`0`
- Safety condition: thread
- Checked-side note: Not recorded.
- User visible: Not recorded.
- Flag: Not recorded.

Safe source:
```rust
pub fn spawn<F, T>(f: F) -> JoinHandle<T>
where
    F: FnOnce() -> T,
    F: Send + 'static,
    T: Send + 'static,
{
    Builder::new().spawn(f).expect("failed to spawn thread")
}
```

Unsafe source:
```rust
pub unsafe fn spawn_unchecked<F, T>(self, f: F) -> io::Result<JoinHandle<T>>
    where
        F: FnOnce() -> T,
        F: Send,
        T: Send,
    {
        Ok(JoinHandle(unsafe { self.spawn_unchecked_(f, None) }?))
    }
```

## Category: `type`

### type #1: `float_to_int` -> `float_to_int_unchecked`

- Source file: `rust/library/core/src/intrinsics/mod.rs`
- Dependent library / type: `algorithm`
- Counterpart status: `unsafe_entry_only_or_unclear_pair`
- Safe function lines: `?`-`?`
- Unsafe function lines: `1616`-`1723`
- Safety condition: 类型转换
- Checked-side note: 安全函数被对应实例化
- User visible: Not recorded.
- Flag: Not recorded.

Safe source:
```rust
安全函数被对应实例化
```

Unsafe source:
```rust
pub unsafe fn float_to_int_unchecked<Float: Copy, Int: Copy>(value: Float) -> Int;

/// Float addition that allows optimizations based on algebraic rules.
///
/// Stabilized as [`f16::algebraic_add`], [`f32::algebraic_add`], [`f64::algebraic_add`] and [`f128::algebraic_add`].
#[rustc_nounwind]
#[rustc_intrinsic]
pub const fn fadd_algebraic<T: Copy>(a: T, b: T) -> T;

/// Float subtraction that allows optimizations based on algebraic rules.
///
/// Stabilized as [`f16::algebraic_sub`], [`f32::algebraic_sub`], [`f64::algebraic_sub`] and [`f128::algebraic_sub`].
#[rustc_nounwind]
#[rustc_intrinsic]
pub const fn fsub_algebraic<T: Copy>(a: T, b: T) -> T;

/// Float multiplication that allows optimizations based on algebraic rules.
///
/// Stabilized as [`f16::algebraic_mul`], [`f32::algebraic_mul`], [`f64::algebraic_mul`] and [`f128::algebraic_mul`].
#[rustc_nounwind]
#[rustc_intrinsic]
pub const fn fmul_algebraic<T: Copy>(a: T, b: T) -> T;

/// Float division that allows optimizations based on algebraic rules.
///
/// Stabilized as [`f16::algebraic_div`], [`f32::algebraic_div`], [`f64::algebraic_div`] and [`f128::algebraic_div`].
#[rustc_nounwind]
#[rustc_intrinsic]
pub const fn fdiv_algebraic<T: Copy>(a: T, b: T) -> T;

/// Float remainder that allows optimizations based on algebraic rules.
///
/// Stabilized as [`f16::algebraic_rem`], [`f32::algebraic_rem`], [`f64::algebraic_rem`] and [`f128::algebraic_rem`].
#[rustc_nounwind]
#[rustc_intrinsic]
pub const fn frem_algebraic<T: Copy>(a: T, b: T) -> T;

/// Returns the number of bits set in an integer type `T`
///
/// Note that, unlike most intrinsics, this is safe to call;
/// it does not require an `unsafe` block.
/// Therefore, implementations must not require the user to uphold
/// any safety invariants.
///
/// The stabilized versions of this intrinsic are available on the integer
/// primitives via the `count_ones` method. For example,
/// [`u32::count_ones`]
#[rustc_intrinsic_const_stable_indirect]
#[rustc_nounwind]
#[rustc_intrinsic]
pub const fn ctpop<T: Copy>(x: T) -> u32;

/// Returns the number of leading unset bits (zeroes) in an integer type `T`.
///
/// Note that, unlike most intrinsics, this is safe to call;
/// it does not require an `unsafe` block.
/// Therefore, implementations must not require the user to uphold
/// any safety invariants.
///
/// The stabilized versions of this intrinsic are available on the integer
/// primitives via the `leading_zeros` method. For example,
/// [`u32::leading_zeros`]
///
/// # Examples
///
/// ```
/// #![feature(core_intrinsics)]
/// # #![allow(internal_features)]
///
/// use std::intrinsics::ctlz;
///
/// let x = 0b0001_1100_u8;
/// let num_leading = ctlz(x);
/// assert_eq!(num_leading, 3);
/// ```
///
/// An `x` with value `0` will return the bit width of `T`.
///
/// ```
/// #![feature(core_intrinsics)]
/// # #![allow(internal_features)]
///
/// use std::intrinsics::ctlz;
///
/// let x = 0u16;
/// let num_leading = ctlz(x);
/// assert_eq!(num_leading, 16);
/// ```
#[rustc_intrinsic_const_stable_indirect]
#[rustc_nounwind]
#[rustc_intrinsic]
pub const fn ctlz<T: Copy>(x: T) -> u32;

/// Like `ctlz`, but extra-unsafe as it returns `undef` when
/// given an `x` with value `0`.
///
/// This intrinsic does not have a stable counterpart.
///
/// # Examples
///
/// ```
/// #![feature(core_intrinsics)]
/// # #![allow(internal_features)]
///
/// use std::intrinsics::ctlz_nonzero;
///
/// let x = 0b0001_1100_u8;
/// let num_leading = unsafe { ctlz_nonzero(x) };
```

### type #2: `downcast_mut` -> `downcast_mut_unchecked`

- Source file: `rust/library/core/src/any.rs`
- Dependent library / type: `any`
- Counterpart status: `confirmed_pair`
- Safe function lines: `264`-`273`
- Unsafe function lines: `324`-`328`
- Safety condition: 必须包含类型T
- Checked-side note: Not recorded.
- User visible: Not recorded.
- Flag: Not recorded.

Safe source:
```rust
    pub fn downcast_mut<T: Any>(&mut self) -> Option<&mut T> {
        if self.is::<T>() {
            // SAFETY: just checked whether we are pointing to the correct type, and we can rely on
            // that check for memory safety because we have implemented Any for all types; no other
            // impls can exist as they would conflict with our impl.
            unsafe { Some(self.downcast_unchecked_mut()) }
        } else {
            None
        }
    }
```

Unsafe source:
```rust
pub unsafe fn downcast_mut_unchecked<T: Any>(&mut self) -> &mut T {
        debug_assert!(self.is::<T>());
        // SAFETY: caller guarantees that T is the correct type
        unsafe { &mut *(self as *mut dyn Any as *mut T) }
    }
```

### type #3: `downcast_mut` -> `downcast_mut_unchecked`

- Source file: `rust/library/core/src/any.rs`
- Dependent library / type: `any`
- Counterpart status: `confirmed_pair`
- Safe function lines: `264`-`273`
- Unsafe function lines: `459`-`462`
- Safety condition: Not recorded.
- Checked-side note: Not recorded.
- User visible: Not recorded.
- Flag: Not recorded.

Safe source:
```rust
    pub fn downcast_mut<T: Any>(&mut self) -> Option<&mut T> {
        if self.is::<T>() {
            // SAFETY: just checked whether we are pointing to the correct type, and we can rely on
            // that check for memory safety because we have implemented Any for all types; no other
            // impls can exist as they would conflict with our impl.
            unsafe { Some(self.downcast_unchecked_mut()) }
        } else {
            None
        }
    }
```

Unsafe source:
```rust
pub unsafe fn downcast_mut_unchecked<T: Any>(&mut self) -> &mut T {
        // SAFETY: guaranteed by caller
        unsafe { <dyn Any>::downcast_mut_unchecked::<T>(self) }
    }
```

### type #4: `downcast_mut` -> `downcast_mut_unchecked`

- Source file: `rust/library/core/src/any.rs`
- Dependent library / type: `any`
- Counterpart status: `confirmed_pair`
- Safe function lines: `264`-`273`
- Unsafe function lines: `591`-`594`
- Safety condition: Not recorded.
- Checked-side note: Not recorded.
- User visible: Not recorded.
- Flag: Not recorded.

Safe source:
```rust
    pub fn downcast_mut<T: Any>(&mut self) -> Option<&mut T> {
        if self.is::<T>() {
            // SAFETY: just checked whether we are pointing to the correct type, and we can rely on
            // that check for memory safety because we have implemented Any for all types; no other
            // impls can exist as they would conflict with our impl.
            unsafe { Some(self.downcast_unchecked_mut()) }
        } else {
            None
        }
    }
```

Unsafe source:
```rust
pub unsafe fn downcast_mut_unchecked<T: Any>(&mut self) -> &mut T {
        // SAFETY: guaranteed by caller
        unsafe { <dyn Any>::downcast_mut_unchecked::<T>(self) }
    }
```

### type #5: `downcast_ref` -> `downcast_ref_unchecked`

- Source file: `rust/library/core/src/any.rs`
- Dependent library / type: `any`
- Counterpart status: `confirmed_pair`
- Safe function lines: `228`-`237`
- Unsafe function lines: `294`-`298`
- Safety condition: 必须包含类型T
- Checked-side note: Not recorded.
- User visible: Not recorded.
- Flag: Not recorded.

Safe source:
```rust
    pub fn downcast_ref<T: Any>(&self) -> Option<&T> {
        if self.is::<T>() {
            // SAFETY: just checked whether we are pointing to the correct type, and we can rely on
            // that check for memory safety because we have implemented Any for all types; no other
            // impls can exist as they would conflict with our impl.
            unsafe { Some(self.downcast_unchecked_ref()) }
        } else {
            None
        }
    }
```

Unsafe source:
```rust
pub unsafe fn downcast_ref_unchecked<T: Any>(&self) -> &T {
        debug_assert!(self.is::<T>());
        // SAFETY: caller guarantees that T is the correct type
        unsafe { &*(self as *const dyn Any as *const T) }
    }
```

### type #6: `downcast_ref` -> `downcast_ref_unchecked`

- Source file: `rust/library/core/src/any.rs`
- Dependent library / type: `any`
- Counterpart status: `confirmed_pair`
- Safe function lines: `228`-`237`
- Unsafe function lines: `563`-`566`
- Safety condition: Not recorded.
- Checked-side note: Not recorded.
- User visible: Not recorded.
- Flag: Not recorded.

Safe source:
```rust
    pub fn downcast_ref<T: Any>(&self) -> Option<&T> {
        if self.is::<T>() {
            // SAFETY: just checked whether we are pointing to the correct type, and we can rely on
            // that check for memory safety because we have implemented Any for all types; no other
            // impls can exist as they would conflict with our impl.
            unsafe { Some(self.downcast_unchecked_ref()) }
        } else {
            None
        }
    }
```

Unsafe source:
```rust
pub unsafe fn downcast_ref_unchecked<T: Any>(&self) -> &T {
        // SAFETY: guaranteed by caller
        unsafe { <dyn Any>::downcast_ref_unchecked::<T>(self) }
    }
```

### type #7: `downcast_ref` -> `downcast_ref_unchecked`

- Source file: `rust/library/core/src/any.rs`
- Dependent library / type: `any+send`
- Counterpart status: `confirmed_pair`
- Safe function lines: `228`-`237`
- Unsafe function lines: `430`-`433`
- Safety condition: Not recorded.
- Checked-side note: 同上
- User visible: Not recorded.
- Flag: Not recorded.

Safe source:
```rust
    pub fn downcast_ref<T: Any>(&self) -> Option<&T> {
        if self.is::<T>() {
            // SAFETY: just checked whether we are pointing to the correct type, and we can rely on
            // that check for memory safety because we have implemented Any for all types; no other
            // impls can exist as they would conflict with our impl.
            unsafe { Some(self.downcast_unchecked_ref()) }
        } else {
            None
        }
    }
```

Unsafe source:
```rust
pub unsafe fn downcast_ref_unchecked<T: Any>(&self) -> &T {
        // SAFETY: guaranteed by caller
        unsafe { <dyn Any>::downcast_ref_unchecked::<T>(self) }
    }
```

### type #8: `downcast` -> `downcast_unchecked`

- Source file: `rust/library/alloc/src/sync.rs`
- Dependent library / type: `Arc/sync`
- Counterpart status: `confirmed_pair`
- Safe function lines: `2877`-`2889`
- Unsafe function lines: `2919`-`2927`
- Safety condition: 包含类型必须得是type T,否则会导致未定义行为
- Checked-side note: Not recorded.
- User visible: Not recorded.
- Flag: Not recorded.

Safe source:
```rust
    pub fn downcast<T>(self) -> Result<Arc<T, A>, Self>
    where
        T: Any + Send + Sync,
    {
        if (*self).is::<T>() {
            unsafe {
                let (ptr, alloc) = Arc::into_inner_with_allocator(self);
                Ok(Arc::from_inner_in(ptr.cast(), alloc))
            }
        } else {
            Err(self)
        }
    }
```

Unsafe source:
```rust
    pub unsafe fn downcast_unchecked<T>(self) -> Arc<T, A>
    where
        T: Any + Send + Sync,
    {
        unsafe {
            let (ptr, alloc) = Arc::into_inner_with_allocator(self);
            Arc::from_inner_in(ptr.cast(), alloc)
        }
    }
```

### type #9: `downcast` -> `downcast_unchecked`

- Source file: `rust/library/alloc/src/boxed/convert.rs`
- Dependent library / type: `Box`
- Counterpart status: `confirmed_pair`
- Safe function lines: `333`-`335`
- Unsafe function lines: `363`-`369`
- Safety condition: 必须得是类型T
- Checked-side note: Not recorded.
- User visible: Not recorded.
- Flag: Not recorded.

Safe source:
```rust
    pub fn downcast<T: Any>(self) -> Result<Box<T, A>, Self> {
        if self.is::<T>() { unsafe { Ok(self.downcast_unchecked::<T>()) } } else { Err(self) }
    }
```

Unsafe source:
```rust
    pub unsafe fn downcast_unchecked<T: Any>(self) -> Box<T, A> {
        debug_assert!(self.is::<T>());
        unsafe {
            let (raw, alloc): (*mut dyn Any, _) = Box::into_raw_with_allocator(self);
            Box::from_raw_in(raw as *mut T, alloc)
        }
    }
```

### type #10: `downcast` -> `downcast_unchecked`

- Source file: `rust/library/alloc/src/boxed/convert.rs`
- Dependent library / type: `Box+Send`
- Counterpart status: `confirmed_pair`
- Safe function lines: `333`-`335`
- Unsafe function lines: `363`-`369`
- Safety condition: 必须得是类型T
- Checked-side note: 同上, 区别在于send
- User visible: Not recorded.
- Flag: Not recorded.

Safe source:
```rust
    pub fn downcast<T: Any>(self) -> Result<Box<T, A>, Self> {
        if self.is::<T>() { unsafe { Ok(self.downcast_unchecked::<T>()) } } else { Err(self) }
    }
```

Unsafe source:
```rust
    pub unsafe fn downcast_unchecked<T: Any>(self) -> Box<T, A> {
        debug_assert!(self.is::<T>());
        unsafe {
            let (raw, alloc): (*mut dyn Any, _) = Box::into_raw_with_allocator(self);
            Box::from_raw_in(raw as *mut T, alloc)
        }
    }
```

### type #11: `downcast` -> `downcast_unchecked`

- Source file: `rust/library/alloc/src/boxed/convert.rs`
- Dependent library / type: `Box+send+Sync`
- Counterpart status: `confirmed_pair`
- Safe function lines: `333`-`335`
- Unsafe function lines: `363`-`369`
- Safety condition: 必须得是类型T
- Checked-side note: 同上, 区别在于send + sync
- User visible: Not recorded.
- Flag: Not recorded.

Safe source:
```rust
    pub fn downcast<T: Any>(self) -> Result<Box<T, A>, Self> {
        if self.is::<T>() { unsafe { Ok(self.downcast_unchecked::<T>()) } } else { Err(self) }
    }
```

Unsafe source:
```rust
    pub unsafe fn downcast_unchecked<T: Any>(self) -> Box<T, A> {
        debug_assert!(self.is::<T>());
        unsafe {
            let (raw, alloc): (*mut dyn Any, _) = Box::into_raw_with_allocator(self);
            Box::from_raw_in(raw as *mut T, alloc)
        }
    }
```

### type #12: `downcast` -> `downcast_unchecked`

- Source file: `rust/library/alloc/src/rc.rs`
- Dependent library / type: `RC`
- Counterpart status: `confirmed_pair`
- Safe function lines: `2162`-`2171`
- Unsafe function lines: `2201`-`2206`
- Safety condition: 用户必须保证类型是T
- Checked-side note: Not recorded.
- User visible: Not recorded.
- Flag: Not recorded.

Safe source:
```rust
    pub fn downcast<T: Any>(self) -> Result<Rc<T, A>, Self> {
        if (*self).is::<T>() {
            unsafe {
                let (ptr, alloc) = Rc::into_inner_with_allocator(self);
                Ok(Rc::from_inner_in(ptr.cast(), alloc))
            }
        } else {
            Err(self)
        }
    }
```

Unsafe source:
```rust
    pub unsafe fn downcast_unchecked<T: Any>(self) -> Rc<T, A> {
        unsafe {
            let (ptr, alloc) = Rc::into_inner_with_allocator(self);
            Rc::from_inner_in(ptr.cast(), alloc)
        }
    }
```

## Category: `type/nonzero`

### type/nonzero #1: `new` -> `new_unchecked`

- Source file: `rust/library/core/src/num/nonzero.rs`
- Dependent library / type: `num/nonzero`
- Counterpart status: `confirmed_pair`
- Safe function lines: `402`-`406`
- Unsafe function lines: `419`-`434`
- Safety condition: 必须保证参数非零并且类型符合
- Checked-side note: Not recorded.
- User visible: Not recorded.
- Flag: Not recorded.

Safe source:
```rust
    pub const fn new(n: T) -> Option<Self> {
        // SAFETY: Memory layout optimization guarantees that `Option<NonZero<T>>` has
        //         the same layout and size as `T`, with `0` representing `None`.
        unsafe { intrinsics::transmute_unchecked(n) }
    }
```

Unsafe source:
```rust
    pub const unsafe fn new_unchecked(n: T) -> Self {
        match Self::new(n) {
            Some(n) => n,
            None => {
                // SAFETY: The caller guarantees that `n` is non-zero, so this is unreachable.
                unsafe {
                    ub_checks::assert_unsafe_precondition!(
                        check_language_ub,
                        "NonZero::new_unchecked requires the argument to be non-zero",
                        () => false,
                    );
                    intrinsics::unreachable()
                }
            }
        }
    }
```

## Category: `vector`

### vector #1: `from_int` -> `from_int_unchecked`

- Source file: `rust/library/portable-simd/crates/core_simd/src/masks.rs`
- Dependent library / type: `vector`
- Counterpart status: `unsafe_entry_only_or_unclear_pair`
- Safe function lines: `?`-`?`
- Unsafe function lines: `190`-`196`
- Safety condition: 安全条件是所有的值都是0或者-1
- Checked-side note: Not recorded.
- User visible: Not recorded.
- Flag: Not recorded.

Safe source:
```rust
pub fn from_int(value: Simd<T, N>) -> Self {
        assert!(T::valid(value), "all values must be either 0 or -1",);
        // Safety: the validity has been checked
        unsafe { Self::from_int_unchecked(value) }
    }
```

Unsafe source:
```rust
pub unsafe fn from_int_unchecked(value: Simd<T, N>) -> Self {
        // Safety: the caller must confirm this invariant
        unsafe {
            core::intrinsics::assume(<T as Sealed>::valid(value));
            Self(mask_impl::Mask::from_int_unchecked(value))
        }
    }
```
