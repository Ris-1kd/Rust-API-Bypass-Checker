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

背景：

这个候选来自 `ring` 的大整数模幂运算实现。它不是 HKDF 路径，而是
`ring/src/arithmetic/bigint/exp.rs` 中围绕工作缓冲区进行分区和复用的
buffer-processing 逻辑。该逻辑的核心不是某个完整密码学算法的吞吐量，而是一个
真实库内部反复执行的底层数据布局操作：预先分配一段 working buffer，然后按照
模数长度等布局参数将其切分成多个互不重叠的可变区域，用于保存中间状态和缓存值。

路径：

```text
ring bigint exponentiation implementation
  -> allocate / receive a working buffer
  -> split the buffer into table region and state region
  -> repeatedly partition the state region into disjoint mutable subregions
  -> split_at_mut(m_len)
```

更具体地说，源代码中存在类似如下的 buffer 分区模式：

```text
working buffer
  -> lookup table region + mutable state region
  -> acc / base_cached / m_cached style state regions
```

这里列出 `acc`、`base_cached`、`m_cached` 的目的只是帮助理解 workload 的数据流，
不建议在论文正文中展开过多变量名。正文更适合抽象描述为：

```text
repeatedly partitioning a pre-allocated working buffer into disjoint mutable
regions with split_at_mut under statically known layout parameters
```

替换点：

- safe：`split_at_mut(m_len)`
- unsafe：等价于 `split_at_mut_unchecked` 的 unchecked mutable slice construction

安全条件：

- `m_len <= current_region.len()`。
- 两个返回的 mutable slice 必须不重叠。
- 在该 workload 中，这些条件来自固定的 buffer layout 和 record size 约束，而不是运行时猜测。
- 因为每次分区都是从同一个已知布局的 working buffer 中按固定长度切出区域，所以它非常适合展示
  “安全条件由上层结构性不变量保证”的情况。

工作负载含义：

- benchmark 不测完整 RSA 或完整 BigInt exponentiation。
- 它保留的是该实现中的关键 buffer-layout 操作：对大量记录反复执行同样的分区和轻量状态更新。
- 这样做的目的不是声称 `ring` 整体密码学操作加速了 3-4%，而是观察当替换点位于一个重复执行的
  数据布局组件中时，局部安全检查移除是否还能在更高层 component 中被观察到。
- 这和 Table 5 的 function-level micro-benchmark 不同：Table 5 更接近“围绕目标函数直接构造输入”，
  而这里保留了真实库中一段上层 buffer 组织逻辑和输入结构。

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

和 HKDF / Table 5 的关系：

- `ring::hkdf::fill_okm()` 是当前 Table 5 中和 `ring` 对应的目标函数。
- BigInt buffer split 不是 `fill_okm()`，因此不能在正文中暗示它是 Table 5 那一行的直接延伸。
- 如果论文坚持 Section 6.3 必须从 Table 5 直接延伸，那么应使用 HKDF，但 HKDF 的性能结果很弱。
- 如果论文允许 Section 6.3 是“额外 application-derived case study”，则 BigInt 更适合作为正向例子。
- 最安全的措辞是避免写 “end-to-end ring speedup”，而写 “a workload derived from ring's buffer-processing logic”。

适合正文的一句话版本：

```text
The extracted workload preserves the component's key buffer-layout operation:
repeatedly partitioning a pre-allocated working buffer into disjoint mutable
regions with split_at_mut under statically known layout parameters.
```

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

讨论背景：

审稿意见要求讨论优化等级敏感性。我们之前明确区分了两个层面：

- 分析器层面：分析器读取 Rust 编译产生的 MIR，代码中主要使用 `tcx.optimized_mir`。优化等级可能影响
  MIR 的形态，例如内联、简化控制流、去除某些中间变量等，但不会改变程序语义，也不会把分析器要处理的
  MIR statement/terminator 变成完全不同的语言。因此优化等级对分析过程本身是一个 reproducibility
  细节，而不是本文性能结果的主变量。
- 性能测量层面：safe/unsafe replacement 的收益来自去掉运行时检查或更轻量的 unsafe primitive。
  这个问题必须在优化后的 release 配置下讨论，因为 debug/O0 下 unsafe 标准库实现中可能仍然保留
  debug assertion 或 unsafe precondition check，不能代表实际 release 性能。

关于 Rust/Cargo 优化等级：

- Cargo 默认 `dev` profile 基本对应 `opt-level=0`，即 debug 构建。
- Cargo 默认 `release` profile 对应 `opt-level=3`，这一点之前确认过；因此不能把 “release mode”
  写成 O2。
- `opt-level=1`、`opt-level=2`、`opt-level=3` 都是性能导向的优化等级，但它们不是简单地保证
  safe/unsafe ratio 单调变化。
- `opt-level=s` 和 `opt-level=z` 是 size-oriented 配置，目标偏向减小二进制体积，而不是最大化运行时性能。

为什么不使用 debug/O0 作为性能主实验：

- 一些标准库 unsafe API 内部包含 `assert_unsafe_precondition!` 之类的机制。
- 这些检查在 debug 或特定 UB-check 配置下可能仍然执行。
- 因此在 O0/debug 下比较 safe API 和 unsafe API，测到的并不一定是“去掉安全检查后的 release 性能差异”。
- O0 更适合作为功能调试设置，而不是本文讨论的 performance replacement 场景。

为什么主实验使用 Cargo release/O3：

- 这是 Cargo 默认 release profile，是 Rust 用户最常见的性能导向构建配置。
- 我们的目标不是比较编译器优化等级本身，而是评估在常见 release 构建下，已验证的 replacement 是否能带来可观察差异。
- O1/O2/Os/Oz 可以作为 sensitivity checks，但不应取代主实验配置。

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

逐项观察：

- `arrayvec` 在 O1/O3 下为负，在 O2 下略正，说明该 case 的替换收益非常容易被周围代码生成差异覆盖。
- `bit-vec` 从 O1 到 O3 逐渐变好，O3 下约 `+4.65%`，这是少数和直觉一致的结果。
- `itertools` 只有 O3 略正，O1/O2 反而为负，说明 heap/iterator 相关路径受优化策略影响较大。
- `rand` 在 O2 下出现较高正收益，但 O3 近似持平，这说明不能简单把 O2/O3 看成线性增强关系。
- `ring` 在 O1/O2 略正，O3 略负，符合我们对 HKDF 路径的判断：局部替换点被更大的上下文成本和噪声稀释。

论文中更适合强调的结论：

- 优化等级会影响具体数值，但不会改变本文的保守结论：replacement 的实际收益取决于 API 语义、调用频率、
  以及替换点是否在热路径上。
- 因为 target-function 结果不单调，所以不应该用这些数据声称“某个优化等级下我们的技术更强”。
- 更合理的说法是：我们选择标准 release/O3 作为主配置，并用 O1/O2/O3 作为敏感性检查，结果表明测量值存在
  workload-dependent variation。

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
- 当前只对 `rand` 做了一个小规模检查，因此不能把它扩展成全局结论。
- `Oz` 下 `rand` 出现 `+12.27%`，但这更适合说明 size-oriented 配置下代码布局和优化策略会改变测量结果，
  不适合作为本文主性能结论。

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

和 target-function 结果的区别：

- API-level benchmark 直接测标准库 API pair 本身，因此更能反映某个安全检查的局部成本。
- target-function benchmark 把 API replacement 放回真实函数或 crate 内部，结果会受到外围逻辑影响。
- application-derived case study 又进一步加入真实数据结构和路径频率因素，因此收益更容易被稀释。
- 因此，API-level 数据可以支撑“哪些 API 类型理论上有较大局部差异”，但不能直接推出应用层收益。

适合 response letter 的简化说法：

```text
We additionally measured selected API-level and target-function-level cases
under multiple optimization levels. The results show that optimization levels
affect the absolute measurements, but the relative benefit remains highly
dependent on the API semantics and the surrounding workload. Therefore, we use
Cargo's default release configuration as the main setting and treat other
optimization levels as sensitivity checks.
```

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

中文理解版：

```text
我们以 Cargo 默认 release profile 作为主实验配置，因为它是 Rust 中最常见的性能导向发布配置。
为了回应优化等级敏感性问题，我们额外在 API-level 和 target-function-level 上测量了 O1、O2、O3，
并对 size-oriented 的 Os/Oz 做了小规模补充检查。实验结果并不呈现单调趋势：safe/unsafe 替换的
相对收益取决于 API 的语义、周围代码的复杂度、以及替换点是否位于频繁执行路径上。因此，我们将
release/O3 作为主结果配置，而把其他优化等级作为敏感性分析，而不是把优化等级本身作为一个新的
性能主张。
```
