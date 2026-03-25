use crate::analysis::diagnostics::DiagnosticsForDefId;
use crate::analysis::memory::symbolic_value::SymbolicValue;
use crate::rustc_middle::ty;
use crate::analysis::option::AnalysisOption;
use crate::analysis::wto::Wto;
use log::{debug, info};
use rustc_hir::def::DefKind;
use rustc_hir::def_id::{DefId, LocalDefId};
use rustc_middle::ty::TyCtxt;
use rustc_session::Session;
use std::collections::{HashMap, HashSet};
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
}

impl<'tcx> Default for WtoCache<'tcx> {
    fn default() -> Self {
        Self {
            value: HashMap::new(),
        }
    }
}

// 用一个非pub的struct存储一下遍历函数列表的时候需要记录的内容
#[derive(Debug)]
struct FuncInfo {
    pub def_id: DefId,
    pub def_id_index: u32,
    pub def_kind: DefKind,
    pub func_name: String,
    pub is_builtin_derive: bool,
}

/// Stores the global information of the analysis
pub struct GlobalContext<'tcx, 'compilation> {
    /// The central data structure of the compiler
    pub tcx: TyCtxt<'tcx>,

    /// Represents the data associated with a compilation session for a single crate
    pub session: &'compilation Session,

    /// The entry function of the analysis
    pub entry_point: DefId,

    /// All the reachable entry points in auto mode.
    pub reachable_entries: Vec<DefId>,

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

        // 1. 首先遍历所有的函数, 如果是show_all_entries就输出然后结束 return None
        let mut all_entries: HashMap<LocalDefId,FuncInfo> = HashMap::new();
        for &local_def_id in tcx.mir_keys(()).iter(){
            let def_id = local_def_id.to_def_id();
            let def_kind = tcx.def_kind(def_id);
            if def_kind == DefKind::Fn || def_kind == DefKind::AssocFn {
                let def_id_index = def_id.index.as_u32();
                let def_path = tcx.def_path_str(def_id);
                let is_builtin_derive: bool  = match def_kind{
                    DefKind::Fn => false,
                    DefKind::AssocFn =>{
                        if let Some(impl_id) = tcx.impl_of_method(def_id){
                            tcx.is_builtin_derived(impl_id)
                        }
                        else{
                            false
                        }
                    }
                    _ => false
                };
                let info: FuncInfo = FuncInfo { def_id, def_id_index, def_kind, func_name: def_path, is_builtin_derive };
                all_entries.insert(local_def_id, info);
            }
        }
        // 如果只是show_entries就展示然后截断, 展示DefId+函数名
        if analysis_options.show_all_entries {
            for (_, info) in &all_entries{
                println!("{:?}", info);
            }
            println!("The total function number: {}", &all_entries.len());        
            return None 
        }

        
        // 2. 如果不是则对其进行遍历然后就对值进行遍历, 提取call语句并且存他们的wto.
        let mut reachable_entries: Vec<DefId> = Vec::new();
        for (local_def_id,func) in &all_entries {
            let def_id = local_def_id.to_def_id();
            match tcx.def_kind(def_id){
                DefKind::Fn => {
                    // 如果是Fn, 那么一定要分析, 就是本地的原生函数
                    todo!();
                },
                DefKind::AssocFn => {
                    // 如果是AssocFn, 用impl排除一些底层trait的派生, 然后也要执行可达性分析.

                },
                _ =>{}
            }
        }
        return None
    

        // if let Some(entry) = entry_func {
        //     Some(Self {
        //         tcx,
        //         session,
        //         function_name_cache: HashMap::new(),
        //         entry_point: entry.to_def_id(),
        //         reachable_entries: Vec::new(),
        //         checked_def_ids: HashSet::new(),
        //         // dropped_heaps: HashSet::new(),
        //         wto_cache: WtoCache::default(),
        //         analysis_options,
        //         diagnostics_for: DiagnosticsForDefId::default(),
        //     })
        // } else {
        //     error!("Entry point not found");
        //     None
        // }
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

    // pub fn get_reachable_analysis(){
    //     todo!();

    //     {
    //       if analysis_options.show_entries {

    //         // 完善版, 将函数的路径以及函数名+函数DefID的值一起输出
    //         let mut entries_names: HashMap<_,(String, u32)> = HashMap::new();
    //         for def_id in tcx.hir().body_owners() {
    //             if tcx.def_kind(def_id) == DefKind::Fn || tcx.def_kind(def_id) == DefKind::AssocFn {
    //                 let name = tcx.item_name(def_id.to_def_id());
    //                 let def_index = def_id.to_def_id().index.as_u32();
    //                 let def_path = tcx.def_path_str(def_id);
    //                 let tuple = (def_path.clone(),def_index);
    //                 if !entries_names.contains_key(&name){
    //                     entries_names.insert(name,tuple);
    //                     println!("{}, {}", &def_path, def_index);
    //                 }
                    
    //             }
    //         }

    //         // 原版show_entries, 仅提供函数名
    //         // let mut names = HashSet::new();
    //         // for def_id in tcx.hir().body_owners() {
    //         //     if tcx.def_kind(def_id) == DefKind::Fn || tcx.def_kind(def_id) == DefKind::AssocFn {
    //         //         let name = tcx.item_name(def_id.to_def_id());
    //         //         if !names.contains(&name) {
    //         //             names.insert(name);
    //         //             println!("{}", name);
    //         //         }
    //         //         // println!("{}", def_id.to_def_id().index.as_u32());
    //         //     }
    //         // }
    //         return None;
    //     }
    //     info!("Initializing GlobalContext");
    //     let mut entry_func = None;

    //     // 以下部分的原逻辑是通过 先罗列函数名+通过entry_name或者entry_index来锁定入口函数
    //     // 这部分函数目前要全部整合修改成我们预设的phase 0 分析!

    //     let mut entry_cadidates: Vec<DefId> = Vec::new();
    //     // List functions
    //     for def_id in tcx.hir().body_owners() {
    //         let def_kind = tcx.def_kind(def_id);
    //         // Find the DefId for the entry point, note that the entry point must be a function
    //         if def_kind == DefKind::Fn || def_kind == DefKind::AssocFn {
    //             // If `entry_def_id_index` flag is provided, find entry point according to the index
    //             if let Some(entry_def_id_index) = analysis_options.entry_def_id_index {
    //                 let item_name = tcx.item_name(def_id.to_def_id());
    //                 if def_id.to_def_id().index.as_u32() == entry_def_id_index {
    //                     entry_func = Some(def_id);
    //                     debug!("Entry Point: {:?}, DefId: {:?}", item_name, def_id);
    //                 } else {
    //                     debug!(
    //                         "Name: {:?}, DefId: {:?}, DefKind: {:?}",
    //                         tcx.item_name(def_id.to_def_id()),
    //                         def_id,
    //                         def_kind
    //                     );
    //                 }
    //             }
    //             // If not, find entry point according to the function name
    //             else {
    //                 let entry_point = analysis_options.entry_point.clone();
    //                 let item_name = tcx.item_name(def_id.to_def_id());
    //                 if item_name.to_string() == *entry_point {
    //                     entry_func = Some(def_id);
    //                     debug!("Entry Point: {:?}, DefId: {:?}", item_name, def_id);
    //                 } else {
    //                     debug!(
    //                         "Name: {:?}, DefId: {:?}, DefKind: {:?}",
    //                         tcx.item_name(def_id.to_def_id()),
    //                         def_id,
    //                         def_kind
    //                     );
    //                 }
    //             }
    //         }
    //     }
    // }
    // }
}
