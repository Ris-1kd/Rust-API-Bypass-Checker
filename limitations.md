# `/src/analysis` 缺陷总结

这份文档基于对 `/src/analysis` 目录及其调用链的静态阅读，重点记录当前会影响分析结果可信度的缺陷，而不是功能缺口或普通 TODO。

## 1. 函数调用返回后，caller 状态会被 callee 状态直接覆盖

严重程度：高

相关位置：
- `src/analysis/mir_visitor/call_visitor.rs:1956`
- `src/analysis/mir_visitor/call_visitor.rs:1982`

问题：
- `transfer_and_refine_normal_return_state` 一开始就执行 `self.block_visitor.body_visitor.state = function_post_state.clone();`
- 这会把 caller 原先的局部状态整体覆盖掉
- 之后真正回传的只有 `return_value_path`
- 参数 side effect 和 heap side effect 的传播逻辑目前大段被注释掉

影响：
- interprocedural analysis 的语义不成立
- 对 `&mut` 参数、堆对象、别名对象的修改很容易丢失
- caller 在 call 前积累的约束也可能被错误冲掉

## 2. 固定点收敛判断忽略 symbolic domain

严重程度：高

相关位置：
- `src/analysis/abstract_domain.rs:59`
- `src/analysis/mir_visitor/body_visitor.rs:1024`

问题：
- `AbstractDomain::leq` 只比较 `numerical_domain`
- WTO 循环收敛、narrowing 停止条件都建立在这个 `leq` 上

影响：
- 即使 symbolic state 还在变化，循环也可能被错误视为“已收敛”
- 得到的 post state 可能不是实际 fixed point
- 后续基于 symbolic 条件的 API 判定会建立在错误状态上

## 3. symbolic lattice 语义不完整，`meet` / `narrowing` 近似不安全

严重程度：高

相关位置：
- `src/analysis/abstract_domain.rs:321`
- `src/analysis/abstract_domain.rs:331`
- `src/analysis/abstract_domain.rs:353`
- `src/analysis/memory/symbolic_domain.rs:126`

问题：
- `join` 走 symbolic map 的 `lub`
- `meet` 直接返回 `other.symbolic_domain.clone()`
- `narrowing_with` 也直接返回 `other.symbolic_domain.clone()`
- `SymbolicDomain::lub` 是“并集式”保留，不是真正的路径敏感合并

影响：
- 分支 join 后可能保留不该同时成立的 symbolic facts
- refine/narrowing 时又可能无根据地覆盖旧状态
- symbolic 部分整体不满足一个稳定可信的格结构

## 4. 当前的轻量化状态存储直接丢弃了引用、堆对象和非 primitive 值

严重程度：高

相关位置：
- `src/analysis/abstract_domain.rs:153`
- `src/analysis/abstract_domain.rs:174`
- `src/analysis/mir_visitor/block_visitor.rs:2840`

问题：
- `update_value_at` 遇到 `Reference`、`HeapBlock`、`NonPrimitive` 直接 forget
- 但后续大量逻辑又依赖别名、引用和复合对象路径继续存在
- `visit_address_of` 明明构造了 `&place`，最后仍然调用 `update_value_at`

影响：
- 引用语义、别名关系、聚合对象内容很容易在状态中直接消失
- `copy_or_move_elements`、`transfer_and_refine`、`refine_paths` 这些逻辑失去依赖基础
- 很多跨语句、跨调用的对象关系无法稳定表达

## 5. `copy_or_move_elements` 是内存模型核心，但已知存在实际 bug，且弱更新未实现

严重程度：高

相关位置：
- `src/analysis/mir_visitor/block_visitor.rs:2970`
- `src/analysis/mir_visitor/block_visitor.rs:3032`

问题：
- 文件中已经明确标出两个具体 bug
- 对非 constant index 位置仍然会落到强更新
- 对 heap / NonPrimitive 复制的展开规则不稳

影响：
- 数组、slice、struct 的抽象内存更新会被污染
- 一旦出现索引写入或局部聚合对象复制，状态可能变错
- API bypass 场景恰恰大量依赖集合长度和元素路径，受影响很大

## 6. SMT 层对很多表达式没有降级策略，而是直接 `unimplemented!()`

严重程度：高

相关位置：
- `src/analysis/z3_solver.rs:214`
- `src/analysis/z3_solver.rs:276`

问题：
- `Offset`、`Cast`、`HeapBlock`、`Join`、`Reference`、`Widen` 都不能转成 Z3 AST
- 不是保守返回 unknown，而是直接 `unimplemented!()`

影响：
- 一旦 symbolic refinement 不够干净，求解阶段就可能直接失效
- 分析器对复杂表达式的健壮性不足

补充问题：
- `Top/Bottom` 被编码成 fresh uninterpreted const，而不是真实 lattice 语义
- 这会把“不可能值/任意值”和“某个未知对象”混在一起

相关位置：
- `src/analysis/z3_solver.rs:329`

## 7. checker 去重按 `DefId` 做，全局上下文敏感性丢失

严重程度：中高

相关位置：
- `src/analysis/mir_visitor/body_visitor.rs:172`

问题：
- `run_checker` 只要某个 `DefId` 检查过一次，后续上下文不再重复检查

影响：
- 同一个函数在不同调用点、不同数值约束下的结果会被混掉
- 很容易漏报或错失“仅在某个调用上下文成立”的 bypass 结论

## 8. 诊断链路是分裂的，结果可见性和分析能力不一致

严重程度：中高

相关位置：
- `src/analysis/analyzer/numerical_analysis.rs:93`
- `src/analysis/mir_visitor/call_visitor.rs:1330`

问题：
- 统一 `emit_diagnostics()` 被注释掉
- 一部分地方直接 `emit()`
- 一部分地方放入 buffer
- 一部分地方 `warning.cancel()`

影响：
- “没报结果”并不等于“证明安全”
- 很难区分是没分析到、被吞掉、还是被取消了
- 调试体验和可信度都受影响

## 9. interprocedural 初始状态设计过于粗暴

严重程度：中高

相关位置：
- `src/analysis/mir_visitor/call_visitor.rs:129`

问题：
- callee 初始化直接 clone 整个 caller state
- 然后再用 fresh offset 初始化参数

影响：
- 会把本不应进入 callee 语义空间的 caller 局部状态带进去
- 后面不得不靠清理逻辑补救
- 说明当前“函数摘要/调用约定”设计还没独立出来

## 10. call frame 清理是补丁式的，不能替代摘要语义

严重程度：中

相关位置：
- `src/analysis/abstract_domain.rs:83`
- `src/analysis/mir_visitor/call_visitor.rs:2068`

问题：
- `drop_call_frame_vars_from` 的目标是控制 fresh offset 膨胀
- 但它是在错误的状态传递设计之后做清理

影响：
- 能减轻状态膨胀，但不能保证语义正确
- 如果 side effect 映射本身有误，清理只会更早丢信息

## 11. `auto_analysis` 和多 domain 支持停留在接口层

严重程度：中

相关位置：
- `src/analysis/global_context.rs:162`
- `src/analysis/analyzer/numerical_analysis.rs:84`

问题：
- `auto_analysis` 直接提前返回
- CLI 虽然支持多 domain，但执行层实际上只跑 `Interval`

影响：
- 外部接口表达出的能力和内部真实能力不一致
- 会增加调试和后续演进成本

## 12. 当前 API handler 已经开始承担“业务规则 + 诊断输出 + 抽象语义更新”三种职责

严重程度：中

相关位置：
- `src/analysis/mir_visitor/call_visitor.rs:1330`
- `src/analysis/mir_visitor/call_visitor.rs:1373`

问题：
- 例如 `handle_checked_add`、`handle_swap` 同时负责：
- 构造条件
- 调求解器
- 直接发诊断
- 决定返回值语义

影响：
- handler 很难复用
- 也很难把“分析结果”和“报告策略”分离
- 后面一旦想做批量建议、JSON 输出、自动重写，改动会比较痛

## 总结

当前最关键的不是“支持更多 API”，而是下面三个根问题还没有稳定：

1. 函数调用前后状态怎么保存、摘要、回传
2. symbolic domain 的 lattice / fixed-point 语义怎么定义
3. 引用、heap、NonPrimitive 到底如何在状态里表示

这三个问题不稳定时，局部 API handler 即使写得再多，分析结果仍然很容易不可信。
