use crate::analysis::diagnostics::DiagnosticsForDefId;
use crate::analysis::option::AnalysisOption;
use crate::analysis::wto::Wto;
use crate::analysis::mir_visitor::func_handler::FunctionBase;
use libc::FALLOC_FL_KEEP_SIZE;
use log::{debug, info, error};
use rustc_hir::def::DefKind;
use rustc_hir::def_id::{DefId, LocalDefId};
use rustc_middle::mir::{Operand, TerminatorKind}; // [新增] 为 call 扫描
use rustc_middle::ty::TyCtxt;
use rustc_session::Session;
use std::collections::{HashMap, HashSet, VecDeque};
use std::fmt;
use std::rc::Rc;

/// Cache the wto so we do not need to recompute them when analyzing a function multiple times
pub struct WtoCache<'tcx> {
    value: HashMap<DefId, Wto<'tcx>>,
}

impl<'tcx> WtoCache<'tcx> {
    pub fn get(&self, def_id: DefId) -> Option<&Wto<'tcx>> {
        self.value.get(&def_id)
    }

    pub fn insert(&mut self, def_id: DefId, wto: Wto<'tcx>) {
        self.value.insert(def_id, wto);
    }

    pub fn contains_key(&self, def_id: &DefId) -> bool {
        self.value.contains_key(def_id)
    }
}

impl<'tcx> Default for WtoCache<'tcx> {
    fn default() -> Self {
        Self {
            value: HashMap::new(),
        }
    }
}

// 用一个非pub的struct存储一下遍历函数列表的时候需要记录的内容
// [修改] 加 Clone，方便把筛选后的 entry 直接塞到 reachable_entries
#[derive(Debug, Clone)]
pub struct FuncInfo {
    pub def_id: DefId,
    pub def_id_index: u32,
    pub def_kind: DefKind,
    pub func_name: String,
    pub is_builtin_derive: bool,

    // 下面两个成员项是在遍历的过程中用来标记函数调用图的边以及统计包含API数量的.
    // 该函数所调用的函数 (仅存一层浅信息，避免递归结构)
    pub callees: Vec<FuncInfo>,
    // 该函数所涉及的API个数 (direct-hit 计数)
    pub api_count: u32,
    // 所涉及的API的列表 (direct-hit 列表；你可按需存 paired API 或 callee 自身)
    pub api_vec: Vec<DefId>,
}

/// Stores the global information of the analysis
pub struct GlobalContext<'tcx, 'compilation> {
    /// The central data structure of the compiler
    pub tcx: TyCtxt<'tcx>,

    /// Represents the data associated with a compilation session for a single crate
    pub session: &'compilation Session,

    /// The entry function of the analysis (bin crate: main; lib crate: 仅用于占位也可)
    pub entry_point: DefId,

    /// All the reachable entry points in auto mode.
    /// 这里存的是：调用链路中（直接或间接）至少一次命中 stdAPI 的 entry
    pub reachable_entries: Vec<FuncInfo>,

    /// Stores the DefIds that have been already checked, to avoid redundant check
    pub checked_def_ids: HashSet<DefId>,

    /// Cache for the Weak Topological Ordering
    pub wto_cache: WtoCache<'tcx>,

    /// Cache for the name of each DefId
    pub function_name_cache: HashMap<DefId, Rc<String>>,

    /// Customized options that may change the behavior of the analysis
    pub analysis_options: AnalysisOption,

    /// Generated diagnostic messages for each DefId
    pub diagnostics_for: DiagnosticsForDefId<'compilation>,
}

impl<'tcx, 'compilation> fmt::Debug for GlobalContext<'tcx, 'compilation> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "GlobalContext")
    }
}

impl<'tcx, 'compilation> GlobalContext<'tcx, 'compilation> {
    pub fn new(
        session: &'compilation Session,
        tcx: TyCtxt<'tcx>,
        analysis_options: AnalysisOption,
    ) -> Option<Self> {
        // ------------------------------------------------------------
        // Phase 0: 枚举所有本地 MIR 函数（Fn/AssocFn）
        // ------------------------------------------------------------

        let mut all_entries: HashMap<LocalDefId, FuncInfo> = HashMap::new();

        for &local_def_id in tcx.mir_keys(()).iter() {
            let def_id = local_def_id.to_def_id();
            let def_kind = tcx.def_kind(def_id);

            if def_kind == DefKind::Fn || def_kind == DefKind::AssocFn {
                let def_id_index = def_id.index.as_u32();
                let def_path = tcx.def_path_str(def_id);

                let is_builtin_derive: bool = match def_kind {
                    DefKind::Fn => false,
                    DefKind::AssocFn => {
                        if let Some(impl_id) = tcx.impl_of_method(def_id) {
                            tcx.is_builtin_derived(impl_id)
                        } else {
                            false
                        }
                    }
                    _ => false,
                };

                let info = FuncInfo {
                    def_id,
                    def_id_index,
                    def_kind,
                    func_name: def_path,
                    is_builtin_derive,
                    callees: Vec::new(),
                    api_count: 0,
                    api_vec: Vec::new(),
                };
                all_entries.insert(local_def_id, info);
            }
        }

        // show_all_entries: 展示然后结束, 或者无分析项
        if analysis_options.show_all_entries || all_entries.is_empty() {
            for (_, info) in &all_entries {
                info!("{:?}", info);
            }
            info!("The total function number: {}", &all_entries.len());
            return None;
        }


        // entry_point：bin 有 main 就用 main；否则随便选一个占位（lib crate 常见）
        // let entry_point = if let Some((entry_local, _entry_type)) = tcx.entry_fn(()) {
        //     entry_local
        // } else {
        //     all_entries.values().next().unwrap().def_id
        // };

        if analysis_options.auto_analysis {
            return None;
            // 先处理完单独的分析在考虑全部自动化分析的事
            todo!();
        }

        let mut entry_point: Option<DefId> = None;
        if analysis_options.entry_def_id_index != None {
            let index = analysis_options.entry_def_id_index.unwrap();
            for local in &all_entries{
                if local.1.def_id_index == index {
                    entry_point = Some(local.1.def_id.clone());
                    break;
                }
            }
            if entry_point == None {
                // 没有指定的DefId
                error!("{} is not a valid entry point index! ", index);
                return None
            }
        }

        
        
        let func_base = FunctionBase::new();
        
        // ------------------------------------------------------------
        // Phase 1: 对每个函数只做“一层 call 捕获”
        //          产出：
        //            - 每个 FuncInfo 的 callees（仅本地函数）
        //            - 每个 FuncInfo 的 direct api_count/api_vec（仅外部调用命中）
        // ------------------------------------------------------------
        // 为了能在 HashMap 上做可变遍历，先收集 key 列表
        let keys: Vec<LocalDefId> = all_entries.keys().copied().collect();

        for k in keys {
            // 这里拿到可变引用，填充 callees/api_count/api_vec
            let Some(func) = all_entries.get_mut(&k) else { continue };

            // 你之前提到想过滤 builtin derive：这里按你的需求，直接不把它当 entry 候选，
            // 但“是否扫描它以建图”你可以自由选择：
            // - 如果你完全不关心 derive 产生的代码：continue（更轻量）
            // - 如果你希望图更完整：不跳过（更保守）
            if func.is_builtin_derive {
                continue;
            }

            Self::basic_block_call_search(tcx, &analysis_options, func, &func_base);
        }

        // ------------------------------------------------------------
        // Phase 2: 从 all_entries 构建本地调用图 succ/pred（DefId 级）
        // ------------------------------------------------------------
        let mut succ: HashMap<DefId, HashSet<DefId>> = HashMap::new();
        let mut pred: HashMap<DefId, HashSet<DefId>> = HashMap::new();

        for (_, caller_info) in &all_entries {
            let caller = caller_info.def_id;
            for callee_info in &caller_info.callees {
                let callee = callee_info.def_id;
                succ.entry(caller).or_default().insert(callee);
                pred.entry(callee).or_default().insert(caller);
            }
        }

        // ------------------------------------------------------------
        // Phase 3: 反向传播：找到所有“调用链路中包含至少一次 stdAPI”的函数集合
        //  1) HitFns：直接命中 stdAPI 的函数（api_count > 0）
        //  2) 从 HitFns 出发沿 pred 反向 BFS/DFS
        // ------------------------------------------------------------
        let mut hit_fns: HashSet<DefId> = HashSet::new();
        for (_, info) in &all_entries {
            if info.api_count > 0 {
                hit_fns.insert(info.def_id);
            }
        }

        let mut reaches_hit: HashSet<DefId> = hit_fns.clone();
        let mut q: VecDeque<DefId> = hit_fns.into_iter().collect();

        while let Some(x) = q.pop_front() {
            if let Some(ps) = pred.get(&x) {
                for &p in ps {
                    if reaches_hit.insert(p) {
                        q.push_back(p);
                    }
                }
            }
        }

        // ------------------------------------------------------------
        // Phase 4: reachable_entries = entry_candidates ∩ reaches_hit
        // 这里“entry_candidates”按你当前习惯：Fn 一定算；AssocFn 排除 builtin derive
        // ------------------------------------------------------------
        let mut reachable_entries: Vec<FuncInfo> = Vec::new();
        for (_, info) in &all_entries {
            let is_entry_candidate = match info.def_kind {
                DefKind::Fn => true,
                DefKind::AssocFn => !info.is_builtin_derive,
                _ => false,
            };
            if is_entry_candidate && reaches_hit.contains(&info.def_id) {
                reachable_entries.push(info.clone());
            }
        }

        if analysis_options.show_reachable_entries{
            for entry in &reachable_entries{
                info!("{:?}, {}, {:?}",entry.def_id,entry.func_name, entry.callees);
            } 
            info!("The number of reachable functions: {}, the total number of all entries: {}", reachable_entries.len(), all_entries.len());
            return None
        }

        let entry_point = entry_point.unwrap();

        Some(Self {
            tcx,
            session,
            entry_point,
            reachable_entries,
            checked_def_ids: HashSet::new(),
            wto_cache: WtoCache::default(),
            function_name_cache: HashMap::new(),
            analysis_options,
            diagnostics_for: DiagnosticsForDefId::default(),
        })
    }

    pub fn get_wto(&mut self, def_id: DefId) -> Wto<'tcx> {
        let mir = self.tcx.optimized_mir(def_id);
        let wto;
        // First see whether the wto has been already computed
        if let Some(cached_wto) = self.wto_cache.get(def_id) {
            debug!("Using cached w.t.o for {}", self.tcx.item_name(def_id));
            wto = cached_wto.clone();
        } else {
            // If not, compute the wto
            wto = Wto::new(mir);
            debug!(
                "Compute the new w.t.o for {}: {:?}",
                self.tcx.item_name(def_id),
                wto
            );
            // Cache the wto
            self.wto_cache.insert(def_id, wto.clone());
        }
        wto
    }

    /// 扫描一个函数：只做“一层”Call 捕获
    /// - 本地 callee：加入 func.callees（浅层信息）
    /// - 外部 callee：由你的匹配逻辑判断是否命中 std 对偶 API；命中则更新 api_count/api_vec
    pub fn basic_block_call_search(
        tcx: TyCtxt<'tcx>,
        analysis_options: &AnalysisOption,
        func: &mut FuncInfo,
        func_base: &FunctionBase
    ) {
        let body = tcx.optimized_mir(func.def_id);

        // 避免重复插入相同 callee / 相同 API
        let mut seen_local_callees: HashSet<DefId> = HashSet::new();
        let mut seen_api: HashSet<DefId> = HashSet::new();

        for bb_data in body.basic_blocks.iter() {
            let Some(term) = &bb_data.terminator else { continue };

            match &term.kind {
                TerminatorKind::Call { func: op, .. } => {
                    // 目前只处理最常见的“直接 FnDef 调用”
                    let Some(callee_def_id) = Self::resolve_direct_callee_def_id(op) else {
                        // 解析不到：函数指针/dyn dispatch 等
                        // 你的保守策略：可在这里记录 unknown callsite，或直接忽略
                        continue;
                    };

                    if callee_def_id.is_local() {
                        // 本地边：caller -> callee
                        if seen_local_callees.insert(callee_def_id) {
                            func.callees.push(Self::mk_shallow_func_info(tcx, callee_def_id));
                        }
                    } else {
                        // 外部调用：交给你的 std 对偶匹配逻辑
                        if let Some(api_id) =
                            Self::match_interesting_std_api(tcx, analysis_options, callee_def_id, func_base)
                        {
                            if seen_api.insert(api_id) {
                                func.api_vec.push(api_id);
                            }
                            func.api_count = func.api_count.saturating_add(1);
                        }
                    }
                }
                _ => {}
            }
        }
    }

    /// 解析最常见的 direct call：Operand::Constant + ty::FnDef(def_id, ..)
    fn resolve_direct_callee_def_id(op: &Operand<'tcx>) -> Option<DefId> {
        if let Operand::Constant(c) = op {
            // 注意：不同 rustc 版本这里的 API 细节可能略有差异
            if let rustc_middle::ty::TyKind::FnDef(def_id, _) = *c.const_.ty().kind() {
                return Some(def_id);
            }
        }
        None
    }

    /// 构造一个“浅层”的 FuncInfo（避免递归嵌套导致环）
    fn mk_shallow_func_info(tcx: TyCtxt<'tcx>, def_id: DefId) -> FuncInfo {
        let def_kind = tcx.def_kind(def_id);
        let def_id_index = def_id.index.as_u32();
        let func_name = tcx.def_path_str(def_id);

        let is_builtin_derive: bool = match def_kind {
            DefKind::Fn => false,
            DefKind::AssocFn => {
                if let Some(impl_id) = tcx.impl_of_method(def_id) {
                    tcx.is_builtin_derived(impl_id)
                } else {
                    false
                }
            }
            _ => false,
        };

        FuncInfo {
            def_id,
            def_id_index,
            def_kind,
            func_name,
            is_builtin_derive,
            callees: Vec::new(),
            api_count: 0,
            api_vec: Vec::new(),
        }
    }

    /// 你自己的 std 对偶匹配逻辑挂钩点：
    /// - 返回 Some(api_id) 表示命中；api_id 由你决定存什么（callee 自身 DefId、或“对偶 ID”）
    /// - 返回 None 表示不关注
    ///
    /// 这里我给的是“默认不命中”，你把内部替换成你的 known_name/完整路径匹配即可。
    fn match_interesting_std_api(
        _tcx: TyCtxt<'tcx>,
        _analysis_options: &AnalysisOption,
        _callee: DefId,
        _func_base: &FunctionBase
    ) -> Option<DefId> {
        // TODO(your logic):
        // 例：用 def_path_str(callee) 做完整路径匹配；
        // 命中时返回 Some(callee) 或 Some(paired_api_defid)（按你需要）
        
        // _func_base.contains_and_get_kind(func_name)
        let func_name: String = _tcx.def_path_str(_callee);
        
        if let Some(kind) = _func_base.contains_and_get_kind(&func_name){
            return Some(_callee.clone())
        }
        None
    }
}
