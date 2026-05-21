use crate::analysis::abstract_domain::AbstractDomain;
use crate::analysis::crate_context::CrateContext;
use crate::analysis::diagnostics::{Diagnostic, DiagnosticCause};
use crate::analysis::global_context::GlobalContext;
use crate::analysis::memory::constant_value::ConstantValue;
use crate::analysis::memory::expression::{Expression, ExpressionType};
use crate::analysis::memory::path::{Path, PathEnum};
use crate::analysis::memory::symbolic_value::{self, SymbolicValue};
use crate::analysis::mir_visitor::block_visitor::BlockVisitor;
use crate::analysis::mir_visitor::call_visitor::CallVisitor;
use crate::analysis::mir_visitor::type_visitor::TypeVisitor;
use crate::analysis::numerical::interval_domain::{
    GetDomainType, IntervalAbstractDomain, NumericalDomainType,
};
use crate::analysis::numerical::linear_constraint::LinearConstraintSystem;
use crate::analysis::wto::{Wto, WtoCircle, WtoVertex, WtoVisitor};
use crate::analysis::z3_solver::Z3Solver;
use crate::checker::assertion_checker::AssertionChecker;
use crate::checker::checker_trait::CheckerTrait;
//use crate::checker::unsafe_func_checker::UnsafeFuncChecker;
use crate::analysis::mir_visitor::func_handler::FuncHandler;
use log::{debug, error, warn};
use rug::Integer;
use rustc_errors::Diag;
use rustc_hir::def_id::DefId;
use rustc_middle::mir;
use rustc_middle::ty::{Const, Ty};
use rustc_span::Span;
use std::collections::{HashMap, HashSet};
use std::convert::TryFrom;
use std::rc::Rc;

/// A wto visitor used to analyze a function
pub struct WtoFixPointIterator<'tcx, 'a, 'compilation, DomainType>
where
    DomainType: NumericalDomainType,
    IntervalAbstractDomain<DomainType>: GetDomainType,
{
    // Global context
    pub context: &'a mut GlobalContext<'tcx, 'compilation>,

    // The current function's DefId
    pub def_id: DefId,

    // The current function's w.t.o
    pub wto: Wto<'tcx>,

    // Current span
    pub current_span: Span,

    // Current location
    pub current_location: mir::Location,

    // The initial state for the fixed-point algorithm
    pub init_state: AbstractDomain<DomainType>,

    // Current abstract state
    pub state: AbstractDomain<DomainType>,

    // The post-condition for each basic block
    pub post: HashMap<mir::BasicBlock, AbstractDomain<DomainType>>,

    // There may be multiple return statements, record them so we can compute the union of the return values
    pub result_blocks: HashSet<mir::BasicBlock>,

    // Helper struct to get information in Rust's type system
    pub type_visitor: TypeVisitor<'tcx>,

    // Helper struct to store information about the current crate
    pub crate_context: CrateContext<'compilation, 'tcx>,

    // Stores the tainted local variables when detecting ownership corruption
    // Variables in this set potentially acquire ownership from other allocated memory
    // So keep track of them and check whether they eventually go to terminators like `Return` or `Drop`
    // If so, then mutable shared memory are created or potential use-after-free / double-free are detected
    // We only consider `mir::Local` instead of `mir::Place` for robustness
    // pub tainted_variables: HashSet<mir::Local>,

    // `Place` to `SymbolicValue` Cache, used to extract conditions when analyzing assertions
    pub place_to_abstract_value: HashMap<mir::Place<'tcx>, Rc<SymbolicValue>>,

    // Path-keyed companion cache for transient boolean expressions. Some MIR operands do not
    // reliably hit the Place-keyed cache after moves, but their normalized Path is still stable
    // inside the local block.
    pub path_to_abstract_value: HashMap<Rc<Path>, Rc<SymbolicValue>>,

    // 使用Span映射到Path的方式，存储各个参数的Path, 尝试过Terminator, TerminatorKind等作key, 由于当时的版本这些数据结构还没有实现Eq, Hash，所以都放弃了。
    pub terminator_to_place: HashMap<Span, Vec<(Rc<Path>, Rc<SymbolicValue>)>>,

    // The start index of variables. Because functions may return values that contain local variables, so we
    // increase the index offsets so that returned variables can be distinguished from normal local variables
    pub fresh_variable_offset: usize,

    // The fresh variable offset used for the next call
    pub next_fresh_variable_offset: usize,

    // The call stack, used to detect recursive calls
    pub call_stack: Vec<DefId>,

    // The Z3 SMT solver
    pub z3_solver: Z3Solver,

    // Buffered diagnostics
    pub buffered_diagnostics: Vec<Option<Diagnostic<'compilation>>>,

    /// The HashMap of the replacable function
    pub replace_funcs: HashSet<FuncHandler>,
}

impl<'tcx, 'a, 'compilation, DomainType> WtoFixPointIterator<'tcx, 'a, 'compilation, DomainType>
where
    DomainType: NumericalDomainType,
    IntervalAbstractDomain<DomainType>: GetDomainType,
{
    /// The offset that we add to `fresh_variable_offset` when calling functions
    pub const FRESH_VARIABLE_OFFSET: usize = 1000000;

    /// Create a new w.t.o visitor for a given w.t.o and its initial state
    pub fn new(
        context: &'a mut GlobalContext<'tcx, 'compilation>,
        def_id: DefId,
        init_state: AbstractDomain<DomainType>,
        fresh_variable_offset: usize,
        call_stack: Vec<DefId>,
    ) -> Self {
        let wto = context.get_wto(def_id);
        let type_visitor = TypeVisitor::new(def_id, wto.get_mir().clone(), context.tcx);

        Self {
            current_span: rustc_span::DUMMY_SP,
            current_location: mir::Location::START,
            context,
            def_id,
            init_state,
            wto,
            state: AbstractDomain::default(),
            post: HashMap::new(),
            result_blocks: HashSet::new(),
            type_visitor,
            crate_context: CrateContext::default(),
            // tainted_variables: HashSet::new(),
            place_to_abstract_value: HashMap::new(),
            path_to_abstract_value: HashMap::new(),
            fresh_variable_offset,
            next_fresh_variable_offset: fresh_variable_offset + Self::FRESH_VARIABLE_OFFSET,
            call_stack,
            z3_solver: Z3Solver::default(),
            buffered_diagnostics: vec![],
            terminator_to_place: HashMap::new(),
            replace_funcs: HashSet::new(),
        }
    }

    /// Run analysis
    pub fn run(&mut self) {
        for comp in self.wto.components() {
            self.visit_component(&comp);
        }
    }

    /// Initialize arguments when analyzing a function
    pub fn init_pre_condition(&mut self, actual_args: Vec<(Rc<Path>, Rc<SymbolicValue>)>) {
        for (i, arg) in actual_args.iter().enumerate() {
            // Initialize callee's arguments using caller's values
            // So callee's paths should add an offset to distinguish them from caller's paths
            let new_path = Path::new_parameter(i + 1, self.fresh_variable_offset);
            self.init_state.update_value_at(new_path, arg.1.clone());
        }
        debug!("Initializing pre condition: {:?}", self.init_state);
    }

    pub fn run_checker(&mut self) {
        // Only avoid running the checker twice; never skip draining diagnostics.
        let already_checked = self.context.checked_def_ids.contains(&self.def_id);
        if !already_checked {
            self.context.checked_def_ids.insert(self.def_id);

            let mut assertion_checker = AssertionChecker::<DomainType>::new(self);
            assertion_checker.run();

            // let mut unsafe_func_checker = UnsafeFuncChecker::<DomainType>::new(self);
            // unsafe_func_checker.run();
        }

        // ALWAYS drain buffered diagnostics; otherwise Diag will be dropped un-emitted and ICE.
        let buffered_diagnostics: Vec<Option<Diagnostic<'compilation>>> = self
            .buffered_diagnostics
            .iter_mut()
            .map(|d| d.take())
            .collect();

        // Store into global context (must append, not overwrite; see change #2).
        self.context
            .diagnostics_for
            .insert(self.def_id, buffered_diagnostics);
    }

    pub fn get_exit_state(&self) -> Option<AbstractDomain<DomainType>> {
        self.post
            .clone()
            .into_iter()
            .filter(|(bb, _domain)| self.result_blocks.contains(bb))
            .map(|(_bb, domain)| domain)
            .reduce(|state1, state2| state1.join(&state2))
    }

    // just for those special promoted constants, as the promoted constants are a unique technique in Rust Compiler.
    pub fn init_promote_constants(&mut self)
    where
        DomainType: NumericalDomainType,
        IntervalAbstractDomain<DomainType>: GetDomainType,
    {
        debug!("Promoted-constant initialization is disabled in numerical-only mode");
    }

    /// Evaluates the length value of an Array type and returns its value as usize
    pub fn get_array_length(&self, length: &'tcx Const<'tcx>) -> usize {
        length
            .try_to_target_usize(self.context.tcx)
            .expect("Array length constant to have a known value") as usize
    }

    #[allow(dead_code)]
    fn promote_reference(
        &mut self,
        environment: &mut AbstractDomain<DomainType>,
        result_rustc_type: Ty<'tcx>,
        promoted_root: &Rc<Path>,
        local_path: &Rc<Path>,
        ordinal: usize,
    ) where
        DomainType: NumericalDomainType,
        IntervalAbstractDomain<DomainType>: GetDomainType,
    {
        let _ = environment;
        let _ = result_rustc_type;
        let _ = promoted_root;
        let _ = local_path;
        let _ = ordinal;
        debug!("Reference promotion is disabled in numerical-only mode");
    }

    pub fn get_new_heap_block(
        &mut self,
        _length: Rc<SymbolicValue>,
        _alignment: Rc<SymbolicValue>,
        // is_zeroed: bool,
        ty: Ty<'tcx>,
    ) -> Rc<SymbolicValue> {
        let _ = ty;
        symbolic_value::TOP.into()
    }

    // TODO: check this
    // When executing: path: local_3, result type: &mut i32, where local_3 is a &(local_1), this function returns local1: Reference
    // It should return &(local_1), fixed.
    // When executing: path: <heap0>, result type: [u32; 5], this function returns <heap0>: NonPrimitive
    // It should return <heap0> directly
    pub fn lookup_path_and_refine_result(
        &mut self,
        path: Rc<Path>,
        result_rustc_type: Ty<'tcx>,
    ) -> Rc<SymbolicValue> {
        debug!(
            "lookup_path_and_refine_result: {:?}, result type: {:?}",
            path, result_rustc_type
        );
        let result_type: ExpressionType = (result_rustc_type.kind()).into();
        if let Some(value) = self.state.value_at(&path) {
            return value;
        }

        if result_type == ExpressionType::Reference {
            if let PathEnum::Alias { value } = &path.value {
                match value.expression.infer_type() {
                    ExpressionType::Reference => return value.clone(),
                    _ if value.expression.is_zero() => {
                        return SymbolicValue::make_from(
                            Expression::Cast {
                                operand: value.clone(),
                                target_type: ExpressionType::Reference,
                            },
                            value.expression_size.saturating_add(1),
                        );
                    }
                    _ => {}
                }
            }
        }

        if result_type.is_integer() {
            let result = SymbolicValue::make_from(
                Expression::Variable {
                    path: path.clone(),
                    var_type: result_type.clone(),
                },
                1,
            );
            self.state.update_value_at(path, result.clone());
            return result;
        }

        SymbolicValue::make_typed_unknown(result_type)
    }

    pub fn import_static(&mut self, path: Rc<Path>) -> Rc<Path> {
        debug!("In import_static, path: {:?}", path);
        if let PathEnum::StaticVariable {
            def_id,
            summary_cache_key,
            expression_type,
        } = &path.value
        {
            if self.state.value_at(&path).is_some() {
                return path;
            }
            self.state.update_value_at(
                path.clone(),
                SymbolicValue::make_typed_unknown(expression_type.clone()),
            );
            self.import_def_id_as_static(&path, *def_id, summary_cache_key);
        }
        path
    }

    fn import_def_id_as_static(
        &mut self,
        _path: &Rc<Path>,
        def_id: Option<DefId>,
        _summary_cache_key: &Rc<String>,
    ) {
        debug!("In import_def_id_as_static");
        let environment_before_call = self.state.clone();
        // let saved_analyzing_static_var = self.analyzing_static_var;
        // self.analyzing_static_var = true;
        let mut block_visitor;
        // let summary;
        if let Some(def_id) = def_id {
            if self.call_stack.contains(&def_id) {
                return;
            }
            let generic_args = self.crate_context.generic_args_cache.get(&def_id).cloned();
            let callee_generic_argument_map = if let Some(generic_args) = generic_args {
                self.type_visitor
                    .get_generic_arguments_map(def_id, generic_args, &[])
            } else {
                None
            };
            let ty = self.context.tcx.type_of(def_id).skip_binder();
            let func_const = self
                .crate_context
                .constant_value_cache
                .get_function_constant_for(
                    def_id,
                    ty,
                    generic_args,
                    self.context.tcx,
                    &mut self.crate_context.known_names_cache,
                    // &mut self.cv.summary_cache,
                )
                .clone();
            block_visitor = BlockVisitor::new(self, environment_before_call);
            let mut call_visitor = CallVisitor::new(
                &mut block_visitor,
                def_id,
                generic_args,
                callee_generic_argument_map,
                // environment_before_call,
                func_const,
            );
            let _func_ref = call_visitor
                .callee_func_ref
                .clone()
                .expect("CallVisitor::new should guarantee this");

            debug!("Executing call visitor for static variable...");
            // Run the call visitor and get post states
            let function_post_state = call_visitor
                .get_function_post_state()
                .unwrap_or_else(AbstractDomain::default);

            debug!(
                "Finish call visitor, get function post state {:?}",
                function_post_state
            );
            debug!(
                "Before handling side-effects, pre env {:?}",
                call_visitor.block_visitor.state()
            );
            call_visitor.transfer_and_refine_normal_return_state(&function_post_state, 0);
            debug!(
                "After handling side-effects, post env {:?}",
                call_visitor.block_visitor.state()
            );
        };
    }

    #[allow(dead_code)]
    fn lookup_weak_value(
        &mut self,
        key_qualifier: &Rc<Path>,
        _key_index: &Rc<SymbolicValue>,
    ) -> Option<Rc<SymbolicValue>> {
        let _ = key_qualifier;
        None
    }

    // TODO: do we need to distinguish signed and unsigned integers? And how to use rug to implement it?
    pub fn get_i128_const_val(&mut self, val: i128) -> Rc<SymbolicValue> {
        Rc::new(ConstantValue::Int(Integer::from(val)).into())
    }

    pub fn get_u128_const_val(&mut self, val: u128) -> Rc<SymbolicValue> {
        Rc::new(ConstantValue::Int(Integer::from(val)).into())
    }

    /// Try to get the symbol name of a variable in debug information
    /// If failed to find the symbol, return a string according to its `Debug` trait implementation
    pub fn get_var_name(&self, operand: &mir::Operand<'tcx>) -> String {
        for var_info in &self.wto.get_mir().var_debug_info {
            match var_info.value {
                mir::VarDebugInfoContents::Place(place1) => match operand {
                    mir::Operand::Copy(place2) | mir::Operand::Move(place2) => {
                        if place1 == *place2 {
                            return var_info.name.to_ident_string();
                        }
                        return format!("{:?}", operand);
                    }
                    _ => return format!("{:?}", operand),
                },
                mir::VarDebugInfoContents::Const(constant1) => match operand {
                    mir::Operand::Constant(constant2) => {
                        if constant1 == **constant2 {
                            return var_info.name.to_ident_string();
                        }
                        return format!("{:?}", operand);
                    }
                    _ => return format!("{:?}", operand),
                },
            }
        }
        // Get here if not found
        format!("{:?}", operand)
    }

    /// Recover the variable name for each assert message
    /// This is used to pretty print the diagnostic messages
    pub fn recover_var_name(&self, assert_kind: &mir::AssertKind<mir::Operand<'tcx>>) -> String {
        use mir::AssertKind::*;
        use mir::BinOp;

        // The following code is adapted from the original implementation of the `Debug` trait for `AssertKind`
        match assert_kind {
            BoundsCheck { ref len, ref index } => format!(
                "index out of bounds: the length is {:?} but the index is {:?}",
                self.get_var_name(len),
                self.get_var_name(index)
            ),
            OverflowNeg(op) => format!(
                "attempt to negate `{:#?}`, which would overflow",
                self.get_var_name(op)
            ),
            DivisionByZero(op) => {
                format!("attempt to divide `{:#?}` by zero", self.get_var_name(op))
            }
            RemainderByZero(op) => format!(
                "attempt to calculate the remainder of `{:#?}` with a divisor of zero",
                self.get_var_name(op)
            ),
            Overflow(BinOp::Add, l, r) => {
                format!(
                    "attempt to compute `{:#?} + {:#?}`, which would overflow",
                    self.get_var_name(l),
                    self.get_var_name(r)
                )
            }
            Overflow(BinOp::Sub, l, r) => {
                format!(
                    "attempt to compute `{:#?} - {:#?}`, which would overflow",
                    self.get_var_name(l),
                    self.get_var_name(r)
                )
            }
            Overflow(BinOp::Mul, l, r) => {
                format!(
                    "attempt to compute `{:#?} * {:#?}`, which would overflow",
                    self.get_var_name(l),
                    self.get_var_name(r)
                )
            }
            Overflow(BinOp::Div, l, r) => {
                format!(
                    "attempt to compute `{:#?} / {:#?}`, which would overflow",
                    self.get_var_name(l),
                    self.get_var_name(r)
                )
            }
            Overflow(BinOp::Rem, l, r) => format!(
                "attempt to compute the remainder of `{:#?} % {:#?}`, which would overflow",
                self.get_var_name(l),
                self.get_var_name(r)
            ),
            Overflow(BinOp::Shr, _, r) => {
                format!(
                    "attempt to shift right by `{:#?}`, which would overflow",
                    self.get_var_name(r)
                )
            }
            Overflow(BinOp::Shl, _, r) => {
                format!(
                    "attempt to shift left by `{:#?}`, which would overflow",
                    self.get_var_name(r)
                )
            }
            MisalignedPointerDereference { .. } => format!("misaligned pointer dereferenc"),
            _ => format!("{:?}", assert_kind),
        }
    }

    pub fn emit_diagnostic(
        &mut self,
        diagnostic_builder: Diag<'compilation, ()>,
        is_memory_safety: bool,
        cause: DiagnosticCause,
    ) {
        use rustc_span::hygiene::{ExpnData, ExpnKind, MacroKind};
        if let [span] = &diagnostic_builder.span.primary_spans() {
            if let Some(ExpnData {
                kind: ExpnKind::Macro(MacroKind::Derive, ..),
                ..
            }) = span.source_callee()
            {
                info!("derive macro has warning: {:?}", diagnostic_builder);
                diagnostic_builder.cancel();
                return;
            }
        }
        let diagnostic = Diagnostic::new(diagnostic_builder, is_memory_safety, cause);
        self.buffered_diagnostics.push(Some(diagnostic));
    }

    // The following are private methods

    /// Execute block visitor to analyze a basic block
    fn analyze_basic_block(&mut self, bb: mir::BasicBlock, pre: AbstractDomain<DomainType>) {
        debug!("###########################################################################");
        debug!("Analyzing basic block: {:?} of Func: {:?}", bb, self.def_id);
        debug!("Pre-Condition for {:?}: {:?}", bb, pre);
        let post;
        if !pre.is_bottom() {
            let mut visitor = BlockVisitor::new(self, pre);
            visitor.visit_basic_block(bb);
            post = &self.state;
        } else {
            debug!("The precondition is bottom, ignore the analysis for this block");
            post = &pre;
        }
        debug!(
            "Finish analyzing basic block: {:?} of Func: {:?}",
            bb, self.def_id
        );
        debug!("Post-Condition for {:?}: {:?}", bb, post);
        debug!("Exit condition {:?}: {:?}", bb, post.exit_conditions);
        self.post.insert(bb, post.clone());
        debug!("###########################################################################\n");
    }

    /// Perform widening if the iteration counter exceeds `widening_delay`
    fn extrapolate(
        &mut self,
        circle: &WtoCircle,
        before: AbstractDomain<DomainType>,
        after: AbstractDomain<DomainType>,
    ) -> AbstractDomain<DomainType> {
        let iteration = circle.get_iter_num();
        let widening_delay = self.context.analysis_options.widening_delay;
        let bb = circle.head().node(); // Get head basic block from circle

        if iteration <= widening_delay {
            // We haven't reached the threshold for widening, so we just execute lub
            before.join(&after)
        } else {
            debug!("Widening for {:?} at iteration: {}", bb, iteration);
            // We have reached the threshold for widening, execute widening
            before.widening_with(&after)
        }
    }

    /// Perform narrowing according to the iteration counter
    fn refine(
        &mut self,
        circle: &WtoCircle,
        before: AbstractDomain<DomainType>,
        after: AbstractDomain<DomainType>,
    ) -> AbstractDomain<DomainType> {
        let iteration = circle.get_iter_num();

        if iteration == 1 {
            // Make sure it will converge
            debug!(
                "Narrowing for {:?} at iteration: {}, use `meet` to guarantee convergence",
                circle.head().node(),
                iteration
            );
            before.meet(&after)
        } else {
            debug!(
                "Narrowing for {:?} at iteration: {}",
                circle.head().node(),
                iteration
            );
            before.narrowing_with(&after)
        }
    }

    /// Merge all the predecessors' states
    fn get_state_from_predecessors(&mut self, bb: mir::BasicBlock) -> AbstractDomain<DomainType> {
        debug!("Start merging state from predecessors");
        let pred_states: Vec<AbstractDomain<DomainType>> =
            // For all predecessors of bb
            self.wto.get_mir().basic_blocks.predecessors()[bb]
                .iter()
                .filter_map(|pred_bb| {
                    // For a predecessor pred_bb, get the post condition
                    if let Some(pred_state) = self.post.get(pred_bb) {
                        let mut pred_state = pred_state.clone();
                        debug!("Get state from {:?}: {:?}", pred_bb, pred_state);
                        // If pred_bb has exit conditions that need to be propagated to bb, add constraints in pred_state
                        if let Some(pred_exit_condition) = pred_state.exit_conditions.get(&bb) {
                            debug!("Get exit condition for {:?}: {:?}", bb, pred_exit_condition);
                            debug!("State before adding constraint: {:?}", pred_state);
                            match LinearConstraintSystem::try_from(pred_exit_condition.clone()){
                                Ok(linear_constraint_system) => pred_state.numerical_domain.add_constraints(linear_constraint_system),
                                Err(e) => error!("{}", e),
                            }
                            debug!("State after adding constraint: {:?}", pred_state);
                        }
                        Some(pred_state)
                    } else {
                        None
                    }
                })
                .collect();
        // Merge states using the join operator
        let joined_state = pred_states
            .into_iter()
            .reduce(|state1, state2| state1.join(&state2))
            .expect("Panic while merging states using fold1");
        debug!("Merged state: {:?}", joined_state);
        joined_state
    }
}

/// Implement `visit_vertex` and `visit_circle`
impl<'tcx, 'a, 'compilation, DomainType> WtoVisitor
    for WtoFixPointIterator<'tcx, 'a, 'compilation, DomainType>
where
    DomainType: NumericalDomainType,
    IntervalAbstractDomain<DomainType>: GetDomainType,
{
    /// Visit a node in w.t.o
    fn visit_vertex(&mut self, vertex: &WtoVertex) {
        let bb = vertex.node();
        // If bb is the entry block (block ID is 0), initialize precondition as init state
        let pre = if vertex.is_entry() {
            self.init_state.clone()
        } else {
            // Otherwise, compute the disjunction of all the predecessors' post conditions
            self.get_state_from_predecessors(bb)
        };
        // self.set_pre(bb, pre.clone());

        // Now analyze this node
        self.analyze_basic_block(bb, pre);
    }

    /// Visit a circle in w.t.o, the analysis will only proceed if the circle reaches its fixed-point
    fn visit_circle(&mut self, circle: &WtoCircle) {
        let head = circle.head();
        let head_bb = head.node();
        debug!("Analyzing loop {:?} with head: {:?}", circle, head);
        // First, find out the precondition of the head node of this circle
        let mut pre = if head.is_entry() {
            // If the head of this circle is the entry block (FIXME: Is it possible?)
            warn!("The head of a circle is the entry block");
            self.init_state.clone()
        } else {
            // Compute the disjunction of the predecessors' post conditions
            self.get_state_from_predecessors(head_bb)
        };

        // Perform the fixed-point algorithm
        loop {
            // Increment iteration counter
            circle.inc_iter_num();

            // Analyze the head basic block
            self.analyze_basic_block(head_bb, pre.clone());

            // Analyze the rest blocks in the body
            for comp in circle {
                self.visit_component(&comp);
            }

            // Check whether fixed-point is reached
            let new_pre = self.get_state_from_predecessors(head_bb);
            if new_pre.leq(&pre) {
                debug!("A Fixed-Point has been reached!");
                break;
            } else {
                debug!("Fixed point is not reached because `new_pre <= pre` does not hold");
                debug!("new_pre: {:?}", new_pre);
                debug!("pre:     {:?}", pre);
                // Fixed-point is not reached, try widening if iteration counter exceeds the threshold
                pre = self.extrapolate(circle, pre, new_pre);
            }
        }

        // Fixed-point is reached, try narrowing
        // Narrowing is not guaranteed to converge in general, so we simply iterate at most `narrowing_iteration` times
        let narrowing_iteration = self.context.analysis_options.narrowing_iteration;
        if narrowing_iteration != 0 {
            for _i in 1..narrowing_iteration + 1 {
                // Narrowing: analyze again in order to get a better result
                // Analyze the head basic block
                self.analyze_basic_block(head_bb, pre.clone());

                // Analyze the rest blocks in the body
                for comp in circle {
                    self.visit_component(&comp);
                }

                // Check whether fixed-point is reached
                let new_pre = self.get_state_from_predecessors(head_bb);

                // Note that here the order is different from the above fixed-point check
                if pre.leq(&new_pre) {
                    // No need for refinement
                    // TODO: do we need to restore post condition here?
                    break;
                } else {
                    pre = self.refine(circle, pre, new_pre);
                }
            }
        }
    }
}
impl<'tcx, 'a, 'compilation, DomainType> Drop
    for WtoFixPointIterator<'tcx, 'a, 'compilation, DomainType>
where
    DomainType: NumericalDomainType,
    IntervalAbstractDomain<DomainType>: GetDomainType,
{
    fn drop(&mut self) {
        // If there are any diagnostics still buffered, drain them into the global storage.
        // This prevents rustc ICE: "error was constructed but not emitted".
        if self.buffered_diagnostics.iter().any(|d| d.is_some()) {
            let drained: Vec<Option<Diagnostic<'compilation>>> = self
                .buffered_diagnostics
                .iter_mut()
                .map(|d| d.take())
                .collect();

            self.context.diagnostics_for.insert(self.def_id, drained);
        }
    }
}
