# `base.csv` 与生成文档的差异说明

对比对象：

- 原始人工整理表：[base.csv](/home/yunlong/workspace/Bypassing/Rust-API-Bypass/unchecked_method_base/base.csv)
- 生成后的源码交叉校对文档：[stdlib-safe-unsafe-counterparts.md](/home/yunlong/workspace/Bypassing/Rust-API-Bypass/unchecked_method_base/stdlib-safe-unsafe-counterparts.md)

## 1. 完全对齐的部分

- 条目总数完全一致：`125`
- 分类计数完全一致
- 文档的每一条都对应 `base.csv` 中的一条记录，没有增删条目

也就是说，这份生成文档不是重新发明了一套清单，而是在你原始 `base.csv` 的基础上补了：

- 当前本地 Rust 源码中的实际函数体
- 当前本地 Rust 源码中的实际行号
- 对 checked/unchecked 是否构成干净 counterpart 的二次标注

## 2. 主要差异来源

生成文档和 `base.csv` 的不同，主要来自三类原因。

### 2.1 `Checked Func` 在原表中缺失或不是函数体

`base.csv` 中有 `58` 条记录没有直接给出 checked 侧函数实现，而是：

- 留空
- 写成“同上”
- 写成“未找到相关safe”
- 写成说明性文字，而不是函数代码

我在生成文档时对这类条目做了两步处理：

1. 按命名规则推导 checked 名字，例如：
   - `downcast_unchecked` -> `downcast`
   - `from_utf8_unchecked` -> `from_utf8`
   - `unwrap_unchecked` -> `unwrap`
   - `unchecked_add` -> `checked_add`
2. 再去当前本地 Rust 源码中查找这个 checked 实现是否真实存在

结果是：

- `58` 条缺失 checked 实现的记录里，有 `25` 条能在源码里确认到 checked 侧实现
- 还有 `33` 条虽然可以机械推导出一个 checked 名字，但在源码中不能确认是一对一 counterpart

典型“推导后确认成功”的例子：

- `downcast_unchecked` -> `downcast`
- `downcast_ref_unchecked` -> `downcast_ref`
- `unwrap_unchecked` -> `unwrap`
- `unwrap_err_unchecked` -> `unwrap_err`
- `from_utf8_unchecked` -> `from_utf8`

典型“能推导名字，但仍不能确认 clean counterpart”的例子：

- `from_vec_unchecked` -> `from_vec`
- `map_unchecked` -> `map`
- `map_unchecked_mut` -> `map_mut`
- `get_unchecked` -> `get`（raw pointer / pointer-like API 上不一定成立）
- `to_int_unchecked` -> `to_int`

## 3. 行号和源码片段为什么会变

生成文档中有 `107` 条记录的 unsafe 侧行号，与 `base.csv` 中记录的 `Start_line/End_line` 不一致。

拆开看：

- `71` 条属于最终确认的 counterpart
- `36` 条属于 unclear / unsafe-only 条目
- 只有 `18` 条 unsafe 行号与原表完全一致

这不是因为我改了条目，而是因为我按你当前目录下 clone 的 Rust 源码重新定位了函数定义。  
因此，行号变化主要反映的是：

- 你原来整理时用的 Rust 源码版本，与当前本地 clone 版本不同
- 原表里部分 `Start_line/End_line` 只记了函数体附近位置，不一定是当前源码的完整函数边界

几个代表性的行号变化例子：

- `alloc/src/boxed/convert.rs`
  - `downcast_unchecked`
  - 原表：`394-400`
  - 当前源码定位：`363-369`

- `alloc/src/collections/btree/map.rs`
  - `insert_after_unchecked`
  - 原表：`3191-3218`
  - 当前源码定位：`3418-3445`

- `core/src/cell.rs`
  - `as_ref_unchecked`
  - 原表：`2284-2287`
  - 当前源码定位：`2533-2536`

## 4. 我新增的二次判定：`confirmed_pair` 与 `unclear_pair`

这个是生成文档相对 `base.csv` 最大的新增信息。

我把每条记录分成两种状态：

- `confirmed_pair`
  说明 checked/unchecked 两端都能在当前 Rust 源码中定位到实现
- `unsafe_entry_only_or_unclear_pair`
  说明 `base.csv` 确实记录了这个 unchecked 项，但 checked 侧不能自动确认是同文件同语义的一对一 counterpart

最终结果：

- `81` 条：`confirmed_pair`
- `44` 条：`unsafe_entry_only_or_unclear_pair`

这些 `44` 条并不代表你整理错了，而是代表：

- 它们更像“相关 API”而不是干净 counterpart
- 或者原表本来就已经备注了“未找到相关safe”
- 或者它们是 intrinsic / raw pointer / internal helper，一对一 safe 映射本来就不稳定

## 5. `unclear` 条目的主要来源

这 `44` 条里，按类别分布如下：

- `complex`: `21`
- `intrinsic`: `10`
- `nullptr`: `3`
- `buf`: `3`
- `index`: `2`
- `type`: `1`
- `vector`: `1`
- `slice/buf`: `1`
- `itron`: `1`
- `thread`: `1`

其中最典型的几类来源是：

### 5.1 原表已经明确写了“未找到相关safe”

例如：

- `from_vec_unchecked`
- `from_boxed_utf8_unchecked`
- `to_int_unchecked`
- `Pin::map_unchecked`
- `Pin::map_unchecked_mut`

### 5.2 intrinsic / primitive 级 API，本来就不是标准库层面的 clean pair

例如：

- `unchecked_div`
- `unchecked_rem`
- `unchecked_shl`
- `unchecked_shr`
- `unchecked_add`
- `unchecked_sub`
- `unchecked_mul`
- `transmute_unchecked`
- `unreachable_unchecked`
- `assert_unchecked`

这些条目在 `base.csv` 中有研究价值，但在“标准库 safe/unsafe counterpart”意义上不一定应该算作干净 pair。

### 5.3 名字能机械对应，但语义不一定是一对一

例如：

- raw pointer 上的 `get_unchecked` / `get_unchecked_mut`
- pointer-like `as_ref_unchecked`
- 一些 `buf`、`thread`、`itron` 相关 API

这类 API 在命名上容易让人误以为有 counterpart，但源码里往往不是同一层抽象的 safe wrapper。

## 6. 你可以怎样理解这份差异

最简洁的结论是：

- 如果你关心“我原始人工表格的条目和分类有没有被改动”，答案是：没有，数量和分类都对齐
- 如果你关心“生成文档是否对原表做了源码层面的再筛选”，答案是：做了
- 如果你关心“这 125 条是不是都能当作论文里严格的一对一 counterpart”，答案是：不能，当前按源码交叉校验后只有 `81` 条比较稳

## 7. 后续建议

如果后面这份材料要直接用于论文，我建议把 `125` 条再拆成三层：

1. `confirmed clean counterparts`
2. `related but unclear counterparts`
3. `unsafe-only / intrinsic / excluded entries`

这样会比直接把 `125` 条平铺成“全部 counterpart”更严谨。
