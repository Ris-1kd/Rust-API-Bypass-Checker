# 实验数据整理

日期：2026-05-18

本文档整理本轮 revision 中讨论过的实验数据，包括
application-derived / e2e 候选案例、优化等级敏感性实验、以及 API-level
补充数据。这里的重点不是给出最终论文表述，而是保留足够详细的数据和解释，
方便后续逐条讨论。

## 数据来源

- `E2E_CASE_STUDY_EXPERIMENTS.md`
- `conversations/E2E_CASE_STUDY_DISCUSSION_2026-05-18.md`
- `evaluation/results/opt_level_target_function_summary.md`
- `evaluation/results/rand_size_opt_summary.md`
- `API-counterprats/bench-results/selected_bench_overview_opt_sensitivity_all.csv`

## Application-Derived / E2E 候选数据

这些实验比单纯 API micro-benchmark 更接近真实程序上下文，但多数仍然是
application-derived component workload，而不是完整应用级别的 full end-to-end
吞吐量测试。

### RustScan Port Planning

替换点：

- safe：`slice.swap(i, j)`
- unsafe：基于指针的 swap，索引由 Fisher-Yates 逻辑生成并保证有效

验证方式：

- Proptest 差分测试通过。
- safe 和 unsafe 版本使用相同的 port vector 和 RNG seed。

| Workload | Safe Time | Unsafe Time | Safe/Unsafe | 解释 |
|---:|---:|---:|---:|---|
| 1,024 ports | 5.1854 us | 5.2583 us | 0.986x | unsafe 略慢 |
| 8,192 ports | 43.407 us | 45.454 us | 0.955x | unsafe 更慢 |
| 65,535 ports | 358.13 us | 361.67 us | 0.990x | 接近持平但略负 |

备注：

这是一个真实且调用链较短的 application component，但不能作为正向案例。可能原因是
Fisher-Yates 中的索引范围非常简单，编译器已经能够较好处理 bounds 信息，替换
`swap` 并没有消除明显的运行时开销。

### Arti Timeout Estimator

路径：

```text
circuit build-time history
  -> sparse histogram bins
  -> k_smallest(n_modes)
  -> itertools::k_smallest::sift_down
  -> heap.swap(...)
```

替换点：

- safe：`heap.swap(...)`
- unsafe：vendored `itertools::sift_down` 中使用 pointer-based swap

验证方式：

- Proptest 差分测试通过。
- 随机生成 build-time samples 和 mode count，比较 safe 和 unsafe estimator 得到的
  `Xm` 是否一致。

| Workload | Safe Time | Unsafe Time | Safe/Unsafe | 解释 |
|---|---:|---:|---:|---|
| 1,000 samples, 10 modes | 6.7796 us | 6.8428 us | 0.991x | 略负 |
| 8,000 samples, 10 modes | 51.792 us | 52.577 us | 0.985x | 略负 |
| 65,536 samples, 10 modes | 422.55 us | 420.99 us | 1.004x | 接近持平 |
| 65,536 samples, 64 modes | 420.75 us | 420.84 us | 1.000x | 持平 |
| Wide 65,536 samples, 64 modes | 483.70 us | 477.59 us | 1.013x | 最好约 1.3% |

备注：

Arti 是语义上最自然的真实调用链之一。它可以说明替换点确实出现在一个更高层的
真实场景里。但是性能收益很弱，因为 `sift_down` 和其中的 `swap` 只是 estimator
的一小部分，histogram 构建和 iterator 逻辑会稀释局部收益。

### ring BigInt Buffer Splitting

路径：

```text
ring bigint exponentiation buffer-processing component
  -> repeated fixed-layout buffer partitioning
  -> split_at_mut(m_len)
```

替换点：

- safe：`split_at_mut(m_len)`
- unsafe：等价于 `split_at_mut_unchecked` 的 unchecked mutable slice construction

验证方式：

- Proptest 差分测试通过。
- 测试生成不同 region size 和 record count，比较 checksum 和最终 buffer 状态。

| m_len | Safe Time | Unsafe Time | Safe/Unsafe | 解释 |
|---:|---:|---:|---:|---|
| 1 | 21.412 us | 21.205 us | 1.010x | 弱正向 |
| 2 | 41.270 us | 41.582 us | 0.993x | 略负 |
| 4 | 80.020 us | 79.156 us | 1.011x | 弱正向 |
| 8 | 144.59 us | 146.41 us | 0.988x | 略负 |
| 16 | 278.66 us | 274.86 us | 1.014x | 弱正向 |
| 32 | 255.38 us | 245.88 us | 1.039x | 最好约 3.9% |

备注：

这是当前真实候选中最好的正向案例。它仍然应该被保守描述为
application-derived component，而不是完整应用加速。另一个需要注意的问题是：
如果 Table 5 中 `ring` 只报告了 HKDF 的 `fill_okm()`，那么这个 BigInt case 和
Table 5 并不完全对应，除非正文中明确说明它是额外 case study，或者同步更新
Table 5。

### rand partial_shuffle

路径：

```text
rand::seq::SliceRandom::partial_shuffle
  -> slice.swap(...)
  -> split_at_mut(...)
```

替换点：

- safe：`swap` 和 `split_at_mut`
- unsafe：`swap_unchecked` 和 `split_at_mut_unchecked`

验证方式：

- Proptest 差分测试通过。
- safe 和 unsafe 使用相同输入和相同 RNG seed。

| Safe Time | Unsafe Time | Safe/Unsafe | 解释 |
|---:|---:|---:|---|
| 7.7491 us | 7.7445 us | 1.001x | 基本无收益 |

备注：

scope 是正确的，而且已经在 evaluation harness 中。但它不适合作为正向 case。

### arrayvec swap_pop

路径：

```text
arrayvec::ArrayVec::swap_pop
  -> slice.swap(index, len - 1)
```

替换点：

- safe：`slice::swap`
- unsafe：`slice::swap_unchecked`

验证方式：

- Proptest 差分测试通过。
- 使用 `Vec` model 比较最终语义。

| Safe Time | Unsafe Time | Safe/Unsafe | 解释 |
|---:|---:|---:|---|
| 1.5843 us | 1.5697 us | 1.009x | 太弱 |

备注：

scope 正确，但收益太小。

### bit-vec push

路径：

```text
bit_vec::BitVec::push
  -> nbits.checked_add(1).expect(...)
```

替换点：

- safe：`checked_add(1).expect(...)`
- unsafe：`unchecked_add(1)`

验证方式：

- Proptest 差分测试通过。
- 比较最终 `BitVec` 内容和 `Vec<bool>` model。

| Safe Time | Unsafe Time | Safe/Unsafe | 解释 |
|---:|---:|---:|---|
| 180.95 us | 180.55 us | 1.002x | 基本无收益 |

备注：

scope 正确，但收益太小。

### ripgrep Match::offset

路径：

```text
Searcher::find
  -> core.find(&slice[pos..])
  -> m.offset(self.core.pos())
  -> Match::offset
  -> checked_add(...).unwrap()
```

替换点：

- safe：`usize::checked_add(amount).unwrap()`
- unsafe：`usize::unchecked_add(amount)`

验证方式：

- Proptest 差分测试通过。
- 生成合法的 `Match { start, end }` 和 offset，比较 safe 与 unchecked 的 shifted match。

| Workload | Safe Time | Unsafe Time | Safe/Unsafe | 解释 |
|---|---:|---:|---:|---|
| 16K matches | 12.169 us | 10.274 us | 1.184x | 明显正向 |
| 65K matches | 49.388 us | 41.093 us | 1.202x | 明显正向 |
| 262K matches | 231.86 us | 172.61 us | 1.343x | 明显正向 |

备注：

这是目前最强的正向 extracted component。scope 是正确的，因为替换点是标准库整数
API pair。但它不是完整 ripgrep end-to-end 搜索，也不在当前 Table 5 的目标函数中，
除非单独补充或重新报告。

### Arrow ScalarBuffer checked_mul

路径：

```text
arrow_buffer::ScalarBuffer::new
  -> offset.checked_mul(size).expect(...)
  -> len.checked_mul(size).expect(...)
```

替换点：

- safe：`checked_mul(...).expect(...)`
- unsafe：`unchecked_mul(...)`

验证方式：

- Proptest 差分测试通过。
- 使用相同的 `Buffer`、`offset`、`len`，比较生成的 typed slice 内容。

| Workload | Safe Time | Unsafe Time | Safe/Unsafe | 解释 |
|---|---:|---:|---:|---|
| 16K slices | 250.03 us | 244.98 us | 1.021x | 弱正向 |
| 65K slices | 1.0309 ms | 1.0233 ms | 1.007x | 弱 |
| 262K slices | 4.2940 ms | 4.1849 ms | 1.026x | 弱正向 |

备注：

这个例子不是字符串场景，语义也比较干净，但收益只有 1-3%，不足以替代 ring 或
ripgrep 候选。

## Target-Function 优化等级敏感性

数据来源：

- `evaluation/results/opt_level_target_function_summary.md`

这些数据是在目标函数级别测量 safe/unsafe replacement 在 O1、O2、O3 下的表现。

| Crate | Opt | Safe Mean | Unsafe Mean | Safe/Unsafe | Speedup |
|---|---:|---:|---:|---:|---:|
| arrayvec | O1 | 1.8238 us | 2.1072 us | 0.8655x | -13.45% |
| arrayvec | O2 | 2.2430 us | 2.2064 us | 1.0166x | +1.66% |
| arrayvec | O3 | 2.1327 us | 2.2799 us | 0.9354x | -6.46% |
| bit-vec | O1 | 199.9500 us | 204.1800 us | 0.9793x | -2.07% |
| bit-vec | O2 | 184.9800 us | 180.1900 us | 1.0266x | +2.66% |
| bit-vec | O3 | 185.1400 us | 176.9200 us | 1.0465x | +4.65% |
| itertools | O1 | 1.2477 ms | 1.3393 ms | 0.9316x | -6.84% |
| itertools | O2 | 1.2796 ms | 1.3036 ms | 0.9816x | -1.84% |
| itertools | O3 | 1.3359 ms | 1.3105 ms | 1.0194x | +1.94% |
| rand | O1 | 32.0910 us | 31.8600 us | 1.0073x | +0.73% |
| rand | O2 | 11.4060 us | 9.8726 us | 1.1553x | +15.53% |
| rand | O3 | 9.7637 us | 9.8401 us | 0.9922x | -0.78% |
| ring | O1 | 18.2860 us | 17.9110 us | 1.0209x | +2.09% |
| ring | O2 | 12.8840 us | 12.4910 us | 1.0315x | +3.15% |
| ring | O3 | 12.6880 us | 12.8020 us | 0.9911x | -0.89% |

解释：

- target-function 层面的结果在 O1、O2、O3 之间不呈现单调变化。
- 局部 safe/unsafe replacement 的收益通常很小，容易受到周围代码生成、内联、
  benchmark 结构和测量噪声的影响。
- O3 仍然可以作为主要设置，因为它是 Cargo 默认 release profile，也是最常见的
  performance-oriented release 配置。
- 不能根据这些数据声称“优化等级越高收益越高”或“优化等级越高收益越低”。

## Size-Oriented 优化等级

数据来源：

- `evaluation/results/rand_size_opt_summary.md`

这里只测了 `rand` 在 `Os` 和 `Oz` 下的表现。

| Crate | Opt | Safe Mean | Unsafe Mean | Safe/Unsafe | Speedup |
|---|---:|---:|---:|---:|---:|
| rand | Os | 7.8598 us | 7.9128 us | 0.9933x | -0.67% |
| rand | Oz | 11.6100 us | 10.3410 us | 1.1227x | +12.27% |

解释：

- `Os` 和 `Oz` 是面向二进制体积的优化设置，不是主要的运行时性能配置。
- 它们的行为可能和 O1/O2/O3 不同，因此更适合作为 sensitivity check，而不是主实验设置。

## API-Level 优化等级敏感性

数据来源：

- `API-counterprats/bench-results/selected_bench_overview_opt_sensitivity_all.csv`

这些数据是在标准库 API pair 本身上测量 O1、O2、O3、Os、Oz 下的表现。它们不是
target-function 级别结果，也不是 application-derived 结果。

| API Pair | O1 Ratio | O2 Ratio | O3 Ratio | Os Ratio | Oz Ratio |
|---|---:|---:|---:|---:|---:|
| `str::from_utf8` | 9.413x | 10.900x | 9.586x | 9.979x | 10.921x |
| `CStr::from_bytes_with_nul` | 9.575x | 8.806x | 9.988x | 10.110x | 6.385x |
| `u32::shl_exact` | 1.723x | 1.954x | 1.922x | 1.894x | 1.751x |
| `i32::shl_exact` | 1.689x | 1.970x | 1.972x | 1.982x | 1.803x |
| `u32::shr_exact` | 1.239x | 1.289x | 1.359x | 1.731x | 1.659x |
| `i32::checked_neg` | 1.050x | 1.024x | 1.242x | 1.176x | 1.125x |
| `slice::as_ascii` | 2.847x | 2.834x | 2.243x | 2.211x | 3.102x |
| `char::from_u32` | 1.500x | 1.494x | 1.408x | 1.408x | 1.573x |
| `Alignment::new` | 1.668x | 1.663x | 1.571x | 1.604x | 1.418x |
| `Arc::get_mut` | 1.276x | 2.039x | 2.065x | 1.289x | 1.217x |
| `Rc::get_mut` | 1.114x | 1.087x | 1.107x | 1.098x | 1.053x |
| `Pin::get_unchecked_mut` | 1.018x | 1.174x | 0.979x | 0.989x | 0.975x |
| `slice::split_at` | 1.158x | 1.055x | 1.046x | 1.059x | 0.965x |
| `slice::swap` | 1.030x | 0.999x | 1.000x | 1.000x | 1.038x |
| `Option::unwrap` | 0.886x | 0.960x | 1.051x | 1.028x | 1.027x |
| `Result::unwrap_err` | 1.164x | 0.982x | 1.036x | 0.996x | 1.026x |
| `BTreeMap::insert_after` | 1.026x | 1.014x | 1.009x | 1.020x | 1.068x |
| `BTreeSet::insert_before` | 1.000x | 0.851x | 0.979x | 1.019x | 1.058x |
| `Layout::from_size_align` | 1.542x | 1.115x | 1.118x | 1.483x | 1.431x |
| `Layout::from_size_alignment` | 0.992x | 0.991x | 1.024x | 1.059x | 1.008x |

解释：

- API-level 的优化等级敏感性依然高度依赖 API 语义和安全检查类型。
- 一些验证成本较高的 API，例如 `str::from_utf8`，在多个优化等级下都有稳定高收益。
- 一些 API，例如 `slice::swap`，在所有优化等级下都接近持平。
- 这些数据适合用于说明“优化等级不会改变主要趋势”，但不能直接证明
  target-function 或 application-level 的收益。

## 保守论文口径

可以考虑的表述方向：

```text
We use Cargo's default release profile as the main configuration because it is
the standard performance-oriented setting for Rust release builds. To evaluate
sensitivity to optimization levels, we additionally measured selected API-level
and target-function-level benchmarks under O1, O2, and O3, with a small
size-optimization check under Os/Oz. The results do not show a monotonic trend:
the relative benefit depends on the API semantics, how much code surrounds the
replacement, and whether the replaced check lies on a hot execution path. This
supports our choice to report the main results under the standard release
configuration while treating optimization-level differences as a sensitivity
factor rather than a separate optimization claim.
```
