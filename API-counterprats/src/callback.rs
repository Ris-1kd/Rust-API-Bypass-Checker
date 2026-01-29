use log::{info, debug};
use rustc_middle::ty::TyCtxt;
use rustc_middle::ty;
use crate::options::*;
use rustc_interface::interface;
use rustc_hir::def::DefKind;
use rustc_hir::def_id::{DefId, LocalDefId};
use rustc_middle::mir::TerminatorKind;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug)]
pub struct AnalyzerCallbacks{
    pub source_name: String,    // the name of the source file to be analyzed
}
impl rustc_driver::Callbacks for AnalyzerCallbacks{
    fn after_analysis<'tcx, 'compiler>(
        &mut self,
        compiler: &'compiler interface::Compiler,
        tcx: TyCtxt<'tcx>
    ) ->rustc_driver::Compilation 
    {
        self.run_analysis(compiler,tcx);
        rustc_driver::Compilation::Stop
    }


    fn config(&mut self, _config: &mut interface::Config){
        self.source_name = _config.input.source_name().prefer_local().to_string();
        info!("The source file: {:?}",self.source_name);
    }
}

impl AnalyzerCallbacks {
    pub fn new() -> Self{
        // the source_name will be filled by config() of Callback trait.
        AnalyzerCallbacks { source_name: String::new() }
    }

    pub fn run_analysis<'tcx,'compiler>(
        &self, 
        compiler:&'compiler interface::Compiler,
        tcx: TyCtxt<'tcx>
    ) 
    {
        // === helper: try to resolve callee DefId from the MIR Call "func" operand ===
    fn callee_def_id_from_operand<'tcx>(
        tcx: TyCtxt<'tcx>,
        body: &rustc_middle::mir::Body<'tcx>,
        func: &rustc_middle::mir::Operand<'tcx>,
    ) -> Option<DefId> {
        // Case 1: directly a constant of FnDef
        if let rustc_middle::mir::Operand::Constant(c) = func {
            let fnty = c.const_.ty();
            if let ty::FnDef(def_id, _) = fnty.kind() {
                return Some(*def_id);
            }
            return None;
        }

        // Case 2: func is stored in a local temp place
        if let rustc_middle::mir::Operand::Copy(place) | rustc_middle::mir::Operand::Move(place) = func {
            let place_ty = place.ty(body, tcx).ty;
            if let ty::FnDef(def_id, _) = place_ty.kind() {
                return Some(*def_id);
            }
        }

        None
    }

    // === collect all direct callees ===
    let mut all_callees: BTreeSet<String> = BTreeSet::new();

    // (optional) also group by caller, nicer for demo output
    let mut by_caller: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();

    // Traverse all items that have a body (functions, impl fns, etc.)
    for local_def_id in tcx.hir().body_owners() {
        let caller_def_id = local_def_id.to_def_id();

        // Method A: filter out non-function body owners (const/static/etc.)
        match tcx.def_kind(caller_def_id) {
            DefKind::Fn | DefKind::AssocFn => {}
            _ => continue,
        }

        let caller_name = tcx.def_path_str(caller_def_id);

        // Optimized MIR is fine for “what calls what” demo (functions only)
        let body = tcx.optimized_mir(local_def_id);

        for bb in body.basic_blocks.iter() {
            let term = bb.terminator();

            if let TerminatorKind::Call { func, .. } = &term.kind {
                if let Some(callee_def_id) = callee_def_id_from_operand(tcx, body, func) {
                    let callee_name = tcx.def_path_str(callee_def_id);

                    all_callees.insert(callee_name.clone());
                    by_caller
                        .entry(caller_name.clone())
                        .or_default()
                        .insert(callee_name);
                }
            }
        }
    }

    // === print results ===
    println!("==== callee list (unique, direct FnDef calls) ====");
    for name in &all_callees {
        println!("{name}");
    }

    println!("==== call graph view (caller -> callees) ====");
    for (caller, callees) in by_caller {
        println!("-- {caller}");
        for callee in callees {
            println!("   -> {callee}");
        }
    }
    println!("==== done ====");
    }
}