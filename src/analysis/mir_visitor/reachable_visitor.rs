// Filename: reachable_visitor.rs
// Date:    2025/5/21
// Description: The ReachCallVisitor traveral all the functions from the entry function to detect whether there exist APIs we concern.
//              At the same it cached all the functions' wto and function infomation, which we can use to build the call graph, and other additional information.

use rustc_middle::mir::{Body, Operand, TerminatorKind};
use log::{info, debug};
use crate::analysis::wto::WtoVisitor;
use crate::analysis::global_context::GlobalContext;
use rustc_middle::ty;
use rustc_hir::def_id::DefId;
use std::collections::VecDeque;
use crate::analysis::wto::*;
use std::rc::Rc;

pub struct PreCallVisitor<'a,'tcx,'compiler>{

    /// Follow the FixPointIterator, using the global_context.
    pub context:&'a mut GlobalContext<'tcx,'compiler>,

    /// Current function location
    pub current_function_def_id:DefId,

    /// The Body to the current DefId
    pub body:Body<'tcx>,

    /// The queue to traverse the functions
    pub queue: VecDeque<DefId>,

    /// The entry function id of the entire 
    pub entry_def_id:DefId,
}

impl<'a,'tcx,'compiler,'b> PreCallVisitor<'a,'tcx,'compiler>{
    pub fn new(context:&'a mut GlobalContext<'tcx,'compiler>, entry_def_id:DefId)->Self{
        let body = context.tcx.optimized_mir(entry_def_id).clone();
        Self{
            context,
            current_function_def_id:entry_def_id,
            body:body,
            queue: VecDeque::new(),
            entry_def_id:entry_def_id,
        }
    }    

    // First check whether the DefId is new. If new then cache its wto and name, or do nothing
    fn cache_wto_and_name_by_def_id(&mut self, def_id:DefId){
        if !self.context.function_name_cache.contains_key(&def_id) && !self.context.wto_cache.contains_key(&def_id){
            let function_name = self.context.tcx.def_path_str(def_id);
            self.context.function_name_cache.insert(def_id,Rc::new(function_name.clone()));
            let wto = self.context.get_wto(def_id);
            self.context.wto_cache.insert(def_id, wto.clone());

            debug!("The function name of {:?} has been cached: {:?}",def_id,function_name);
            debug!("The wto result of {:?} has been cached: {:?}",def_id,wto);
            debug!("The function :{:?} has been enqueued.",def_id);
        }
    }

    // Using queue instead of recursive to avoid the arguments pass problems.
    pub fn visit_terminator_calls(&mut self, entry_def_id:DefId){
        self.queue.push_back(entry_def_id);

        while !self.queue.is_empty(){
            let head_def_id = self.queue.front().unwrap().clone();
            self.cache_wto_and_name_by_def_id(head_def_id);
            let wto = self.context.get_wto(head_def_id);
            self.body = self.context.tcx.optimized_mir(head_def_id).clone();
            // visit the current function's basicblocks
            for comp in wto.components(){
                self.visit_component(&comp);
            }
            let _ = self.queue.pop_front();
        }
    }

    // Use info to show the collected APi
    pub fn show(&self){
        info!("Function name cache: {:?}",self.context.function_name_cache);
    }

    fn match_base_function(&mut self,def_id:DefId,caller_def_id:DefId){
        todo!()
    }

}


impl<'tcx,'a,'comipler> WtoVisitor for PreCallVisitor<'tcx,'a,'comipler>{
    fn visit_circle(&mut self, circle: &WtoCircle) {
        for comp in circle{
            self.visit_component(comp);
        }  
    }

    fn visit_vertex(&mut self, vertex: &WtoVertex) {
        let bb = vertex.node();
        // in PreCallVisitor, we don't care the Statement, and the state is also not concerned. Only care Terminator
        let terminator = &self.body.basic_blocks[bb].terminator.as_ref().unwrap();
        match &terminator.kind{
            TerminatorKind::Call{ func,.. } => {
                if let Operand::Constant(c) = func{
                    if let ty::FnDef(def_id,_) = c.const_.ty().kind(){
                        if def_id.is_local(){
                            debug!("In {:?} {:?} found a local function call in : [{:?}], its DefId is: {:?}.",bb ,self.current_function_def_id,func,def_id);
                            self.queue.push_back(*def_id);
                        }
                        // else, we have to match the function name to decide whether it is in our basement
                        else{
                            debug!("In {:?} {:?} found a lib function call:: [{:?}], its DefId is: {:?}.",bb,self.current_function_def_id,func,def_id);
                            self.match_base_function(def_id.clone(),self.current_function_def_id.clone()); 
                        }
                    }
                }  
            },
            // The TailCall is a little different from normal Call, we just set it aside temporarily.
            // TerminatorKind::TailCall{func,..}=>{ },
            _=> {
                // do nothing
                // debug!("In {:?} {:?} found a non-concern terminator and its type:{:?}",bb, self.current_function_def_id,others) // Just for debug
            }
        }
    }

    
}
