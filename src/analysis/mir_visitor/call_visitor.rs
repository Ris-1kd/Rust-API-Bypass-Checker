// This file is adapted from MIRAI (https://github.com/facebookexperimental/MIRAI)
// Original author: Herman Venter <hermanv@fb.com>
// Original copyright header:

// Copyright (c) Facebook, Inc. and its affiliates.
//
// This source code is licensed under the MIT license found in the
// LICENSE file in the root directory of this source tree.

use crate::analysis::abstract_domain::AbstractDomain;
use crate::analysis::diagnostics::DiagnosticCause;
use crate::analysis::memory::constant_value::{ConstantValue, FunctionReference};
use crate::analysis::memory::expression::{Expression, ExpressionType};
use crate::analysis::memory::known_names::KnownNames;
use crate::analysis::memory::path::{Path, PathEnum, PathRefinement};
use crate::analysis::memory::symbolic_value::{self, SymbolicValue, SymbolicValueTrait};
use crate::analysis::mir_visitor::block_visitor::BlockVisitor;
use crate::analysis::mir_visitor::body_visitor::WtoFixPointIterator;
use crate::analysis::numerical::apron_domain::{
    ApronAbstractDomain, ApronDomainType, GetManagerTrait,
};
use crate::analysis::numerical::interval::{Bound, Interval};
use crate::analysis::numerical::linear_constraint::LinearConstraintSystem;
use crate::checker::assertion_checker::{AssertionChecker, CheckerResult};
use crate::checker::checker_trait::CheckerTrait;
use rustc_hir::Mutability;
use rustc_hir::def_id::DefId;
use rustc_index::Idx;
// use rustc_middle::mir;
// use rustc_middle::ty::subst::GenericArgsRef;
// use rustc_middle::ty::{Ty, TyKind};
use rustc_middle::mir;
use rustc_middle::mir::TerminatorKind;
use rustc_middle::ty::{GenericArgsRef, Ty, TyKind};
use rustc_span::source_map::Spanned;
use std::collections::{HashMap, HashSet};
use std::fmt::{Debug, Formatter, Result};
use std::rc::Rc;

pub struct CallVisitor<'call, 'block, 'analysis, 'compilation, 'tcx, DomainType>
where
    DomainType: ApronDomainType,
    ApronAbstractDomain<DomainType>: GetManagerTrait,
{
    /// The upper layer block visitor
    pub block_visitor: &'call mut BlockVisitor<'tcx, 'analysis, 'block, 'compilation, DomainType>,

    /// The callee's DefId
    pub callee_def_id: DefId,

    /// The callee's FunctionReference
    pub callee_func_ref: Option<Rc<FunctionReference>>,

    /// The callee's SymbolicValue
    pub callee_fun_val: Rc<SymbolicValue>,

    /// The callee's generic argument list
    pub callee_generic_arguments: Option<GenericArgsRef<'tcx>>,

    /// The callee's KnownNames
    pub callee_known_name: KnownNames,

    /// The callee's generic arguments' types
    pub callee_generic_argument_map: Option<HashMap<rustc_span::Symbol, Ty<'tcx>>>,

    pub args: &'call [Spanned<mir::Operand<'tcx>>],

    /// The actual arguments of the callee, the paths and symbolic values are from the caller
    pub actual_args: &'call [(Rc<Path>, Rc<SymbolicValue>)],

    /// The list of types of the actual arguments
    pub actual_argument_types: &'call [Ty<'tcx>],

    /// The destination where the return value is assigned
    pub destination: mir::Place<'tcx>,

    /// If the arguments are functions, store them
    pub function_constant_args: &'call [(Rc<Path>, Rc<SymbolicValue>)],

    /// The call stack, used to detect recursive calls
    pub call_stack: Vec<DefId>,
}

impl<'call, 'block, 'analysis, 'compilation, 'tcx, DomainType> Debug
    for CallVisitor<'call, 'block, 'analysis, 'compilation, 'tcx, DomainType>
where
    DomainType: ApronDomainType,
    ApronAbstractDomain<DomainType>: GetManagerTrait,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        "CallVisitor".fmt(f)
    }
}

impl<'call, 'block, 'analysis, 'compilation, 'tcx, DomainType>
    CallVisitor<'call, 'block, 'analysis, 'compilation, 'tcx, DomainType>
where
    DomainType: ApronDomainType,
    ApronAbstractDomain<DomainType>: GetManagerTrait,
{
    pub(crate) fn new(
        block_visitor: &'call mut BlockVisitor<'tcx, 'analysis, 'block, 'compilation, DomainType>,
        callee_def_id: DefId,
        callee_generic_arguments: Option<GenericArgsRef<'tcx>>,
        callee_generic_argument_map: Option<HashMap<rustc_span::Symbol, Ty<'tcx>>>,
        func_const: ConstantValue,
    ) -> CallVisitor<'call, 'block, 'analysis, 'compilation, 'tcx, DomainType> {
        if let ConstantValue::Function(func_ref) = &func_const {
            let callee_known_name = func_ref.known_name;
            let active_calls = block_visitor.body_visitor.call_stack.clone();
            CallVisitor {
                block_visitor, // This is a reference to the caller's block visitor
                callee_def_id,
                callee_func_ref: Some(func_ref.clone()),
                callee_fun_val: Rc::new(func_const.into()),
                callee_generic_arguments,
                callee_known_name,
                callee_generic_argument_map,
                args: &[],
                actual_args: &[],
                actual_argument_types: &[],
                destination: mir::Place::return_place(),
                function_constant_args: &[],
                call_stack: active_calls,
            }
        } else {
            unreachable!("caller should supply a constant function")
        }
    }

    /// Legacy wrapper-only fallback: analyze a callee under the caller state and return its post
    /// state. The default ordinary-call path no longer uses this.
    pub fn create_function_post_state(&mut self) -> AbstractDomain<DomainType> {
        debug!(
            "Creating callee's post state, def_id={:?}, type of def_id={:?}",
            self.callee_def_id,
            self.block_visitor
                .body_visitor
                .context
                .tcx
                .type_of(self.callee_def_id)
        );
        // If MIR is available, analyze it
        if self
            .block_visitor
            .body_visitor
            .context
            .tcx
            .is_mir_available(self.callee_def_id)
        {
            // Get initial state from caller's state
            // We need to get all the values that may be used in callee's analysis
            // So here we get all the values that represent heap allocations

            // let init_abstract_value = self.extract_heap_value(&self.block_visitor.state);
            // TODO: try to include all states of the caller
            let init_abstract_value = self.block_visitor.state().clone();

            info!("====== Fixed-Point Algorithm Starts ======");
            debug!(
                "Initializing Fixed point iterator with abstract domain: {:?}",
                init_abstract_value
            );
            let mut body_visitor = WtoFixPointIterator::new(
                self.block_visitor.body_visitor.context,
                self.callee_def_id,
                init_abstract_value,
                self.block_visitor.body_visitor.next_fresh_variable_offset,
                self.call_stack.clone(),
            );
            body_visitor.type_visitor.actual_argument_types = self.actual_argument_types.into();
            body_visitor.type_visitor.generic_arguments = self.callee_generic_arguments;
            body_visitor.type_visitor.generic_argument_map =
                self.callee_generic_argument_map.clone();

            // Initialize initial precondition using arguments of the callee
            body_visitor.init_pre_condition(self.actual_args.to_vec());

            debug!("Running fixed point iterator");
            body_visitor.run();

            // Run the bug detector
            body_visitor.run_checker();

            // Update the fresh variable offset for the next call
            self.block_visitor.body_visitor.next_fresh_variable_offset =
                body_visitor.next_fresh_variable_offset;

            let post = body_visitor.post.clone();
            debug!("Fixed point iterator finishes, post: {:?}", post);
            // // Compute the join of all the basic blocks that contain a return terminator
            // let joined_state = post
            //     .into_iter()
            //     .filter(|(bb, _domain)| body_visitor.result_blocks.contains(bb))
            //     .map(|(_bb, domain)| domain)
            //     .reduce(|state1, state2| state1.join(&state2))
            //     .expect("panic in fold1");
            // return joined_state;
            // example3/case2触发无return block
            let joined_state = post
                .into_iter()
                .filter(|(bb, _domain)| body_visitor.result_blocks.contains(bb))
                .map(|(_bb, domain)| domain)
                .reduce(|state1, state2| state1.join(&state2))
                .unwrap_or_else(|| AbstractDomain::<DomainType>::default()); // 提供一个默认值
            return joined_state;
        }
        // If MIR is NOT available, return default abstract domain  
        // AbstractDomain::default()
        self.block_visitor.state().clone()
    }

    /// Returns the function reference part of the value, if there is one.
    fn get_func_ref(&mut self, val: &Rc<SymbolicValue>) -> Option<Rc<FunctionReference>> {
        let extract_func_ref = |c: &ConstantValue| match c {
            ConstantValue::Function(func_ref) => Some(func_ref.clone()),
            _ => None,
        };
        match &val.expression {
            Expression::CompileTimeConstant(c) => {
                // debug!("Expression::CompileTimeConstant");
                return extract_func_ref(c);
            }
            Expression::Reference(path)
            | Expression::Variable {
                path,
                var_type: ExpressionType::NonPrimitive,
            }
            | Expression::Variable {
                path,
                var_type: ExpressionType::Reference,
            } => {
                // debug!("Expression::Reference/Variable");
                let closure_ty = self
                    .block_visitor
                    .body_visitor
                    .type_visitor
                    .get_path_rustc_type(path, self.block_visitor.body_visitor.current_span);

                // 实例化后的Ty, 因为Ty内存在一些泛型参数，要把他们实例化。
                let _specialized_closure_ty = self
                    .block_visitor
                    .body_visitor
                    .type_visitor
                    .specialize_generic_argument_type(
                        closure_ty,
                        &self
                            .block_visitor
                            .body_visitor
                            .type_visitor
                            .generic_argument_map,
                    );
                match closure_ty.kind() {
                    TyKind::Closure(def_id, args) => {
                        let args = self
                            .block_visitor
                            .body_visitor
                            .type_visitor
                            .specialize_generic_args(
                                args,
                                &self
                                    .block_visitor
                                    .body_visitor
                                    .type_visitor
                                    .generic_argument_map,
                            );
                        return extract_func_ref(self.block_visitor.visit_function_reference(
                            *def_id,
                            closure_ty,
                            Some(args),
                        ));
                    }
                    TyKind::Ref(_, ty, _) => {
                        if let TyKind::Closure(def_id, args) = ty.kind() {
                            let _specialized_substs = self
                                .block_visitor
                                .body_visitor
                                .type_visitor
                                .specialize_generic_args(
                                    args,
                                    &self
                                        .block_visitor
                                        .body_visitor
                                        .type_visitor
                                        .generic_argument_map,
                                );
                            return extract_func_ref(self.block_visitor.visit_function_reference(
                                *def_id,
                                *ty,
                                Some(args),
                            ));
                        }
                    }
                    _ => {}
                }
            }
            _ => {}
        }
        None
    }

    pub fn get_function_post_state(&mut self) -> Option<AbstractDomain<DomainType>> {
        debug!("Attempting legacy wrapper-only callee analysis fallback");
        // 获得被调用函数的符号值
        let fun_val = self.callee_fun_val.clone();
        // 获得函数引用
        if let Some(func_ref) = self.get_func_ref(&fun_val) {
            // 如果调用栈不存在这个def_id，则推入调用栈，返回这个函数的post_state
            if !self.call_stack.contains(&func_ref.def_id.unwrap()) {
                self.call_stack.push(func_ref.def_id.unwrap());
                debug!("call stack {:?}, ", self.call_stack);
                let res = Some(self.create_function_post_state());
                return res;
            }
            // todo!("The problem occurs in call stack!");
        }
        // 无法获得函数引用，返回None
        warn!("Failed to get_func_ref");
        None
    }

    fn record_supported_special_call(&mut self) {
        self.block_visitor.body_visitor.context.supported_special_calls += 1;
    }

    fn record_call_boundary(&mut self) {
        self.block_visitor.body_visitor.context.opaque_call_boundaries += 1;
    }

    fn forget_destination_value(&mut self) {
        #[allow(irrefutable_let_patterns)]
        let destination_path = if let dest = self.destination {
            Some(self.block_visitor.get_path_for_place(&dest))
        } else {
            None
        };
        if let Some(target_path) = destination_path {
            self.block_visitor
                .body_visitor
                .state
                .update_value_at(target_path, symbolic_value::TOP.into());
        }
    }

    fn emit_unsupported_special_call(&mut self, api_name: &str, reason: &str) -> bool {
        self.block_visitor.body_visitor.context.unsupported_special_calls += 1;
        self.forget_destination_value();
        let warning = self
            .block_visitor
            .body_visitor
            .context
            .session
            .dcx()
            .struct_span_warn(
                self.block_visitor.body_visitor.current_span,
                format!(
                    "[Bypasser] Unsupported analysis fragment at `{}`; result downgraded to unknown: {}",
                    api_name, reason
                ),
            );
        self.block_visitor.body_visitor.emit_diagnostic(
            warning,
            false,
            DiagnosticCause::Unsupported,
        );
        true
    }

    fn is_supported_scalar_type(ty: Ty<'tcx>) -> bool {
        matches!(
            ty.kind(),
            TyKind::Bool | TyKind::Char | TyKind::Int(_) | TyKind::Uint(_) | TyKind::Float(_)
        )
    }

    fn is_supported_integer_type(ty: Ty<'tcx>) -> bool {
        matches!(ty.kind(), TyKind::Int(_) | TyKind::Uint(_))
    }

    fn supported_sequence_element_type(&self, ty: Ty<'tcx>) -> Option<Ty<'tcx>> {
        match ty.kind() {
            TyKind::Array(elem, _) | TyKind::Slice(elem) => Some(*elem),
            TyKind::Ref(_, inner, _) => match inner.kind() {
                TyKind::Array(elem, _) | TyKind::Slice(elem) => Some(*elem),
                _ => None,
            },
            _ => None,
        }
    }

    fn supports_local_bounds_check(&self) -> bool {
        self.actual_argument_types
            .first()
            .and_then(|ty| self.supported_sequence_element_type(*ty))
            .map(Self::is_supported_scalar_type)
            .unwrap_or(false)
    }

    fn supports_local_swap_check(&self) -> bool {
        let receiver_supported = self
            .actual_argument_types
            .first()
            .and_then(|ty| self.supported_sequence_element_type(*ty))
            .is_some();
        let index_a_supported = self
            .actual_argument_types
            .get(1)
            .copied()
            .map(Self::is_supported_integer_type)
            .unwrap_or(false);
        let index_b_supported = self
            .actual_argument_types
            .get(2)
            .copied()
            .map(Self::is_supported_integer_type)
            .unwrap_or(false);
        receiver_supported && index_a_supported && index_b_supported
    }

    fn supports_bool_to_usize_from(&mut self) -> bool {
        if self.actual_argument_types.len() != 1 {
            return false;
        }
        if !matches!(self.actual_argument_types[0].kind(), TyKind::Bool) {
            return false;
        }
        matches!(
            self.block_visitor
                .body_visitor
                .type_visitor
                .get_place_type(
                    &self.destination,
                    self.block_visitor.body_visitor.current_span,
                ),
            ExpressionType::Usize
        )
    }

    fn type_allows_callee_side_effects(ty: Ty<'tcx>) -> bool {
        matches!(
            ty.kind(),
            TyKind::Ref(_, _, Mutability::Mut) | TyKind::RawPtr(_, Mutability::Mut)
        )
    }

    fn forget_argument_related_state(&mut self, path: &Rc<Path>, value: &Rc<SymbolicValue>) {
        self.block_visitor.body_visitor.state.forget_paths_rooted_by(path);
        match &value.expression {
            Expression::Reference(inner)
            | Expression::Variable { path: inner, .. }
            | Expression::Numerical(inner)
            | Expression::Widen { path: inner, .. } => {
                self.block_visitor
                    .body_visitor
                    .state
                    .forget_paths_rooted_by(inner);
            }
            _ => {}
        }
    }

    fn forget_possible_callee_side_effects(&mut self) {
        for ((path, value), ty) in self.actual_args.iter().zip(self.actual_argument_types.iter()) {
            if Self::type_allows_callee_side_effects(*ty) {
                self.forget_argument_related_state(path, value);
            }
        }
    }

    fn unique_predecessor(&self, bb: mir::BasicBlock) -> Option<mir::BasicBlock> {
        let predecessors = &self.block_visitor.mir.basic_blocks.predecessors()[bb];
        if predecessors.len() == 1 {
            Some(predecessors[0])
        } else {
            None
        }
    }

    fn local_place_for_path(&self, path: &Rc<Path>) -> Option<mir::Place<'tcx>> {
        let local = match &path.value {
            PathEnum::Result => mir::Local::from_usize(0),
            PathEnum::Parameter { ordinal } | PathEnum::LocalVariable { ordinal } => {
                mir::Local::from_usize(*ordinal)
            }
            _ => return None,
        };
        (local.index() < self.block_visitor.mir.local_decls.len()).then(|| local.into())
    }

    fn path_from_symbolic_value(value: &Rc<SymbolicValue>) -> Option<Rc<Path>> {
        match &value.expression {
            Expression::Numerical(path)
            | Expression::Reference(path)
            | Expression::Variable { path, .. }
            | Expression::Widen { path, .. } => Some(path.clone()),
            _ => None,
        }
    }

    fn straight_line_predecessor_chain(
        &self,
        bb: mir::BasicBlock,
        limit: usize,
    ) -> Vec<mir::BasicBlock> {
        let mut chain = Vec::new();
        let mut cursor = bb;
        let mut seen = HashSet::new();
        while chain.len() < limit {
            let Some(pred) = self.unique_predecessor(cursor) else {
                break;
            };
            if !seen.insert(pred) {
                break;
            }
            chain.push(pred);
            cursor = pred;
        }
        chain
    }

    fn straight_line_predecessor_edges(
        &self,
        bb: mir::BasicBlock,
        limit: usize,
    ) -> Vec<(mir::BasicBlock, mir::BasicBlock)> {
        let mut edges = Vec::new();
        let mut cursor = bb;
        let mut seen = HashSet::new();
        while edges.len() < limit {
            let Some(pred) = self.unique_predecessor(cursor) else {
                break;
            };
            if !seen.insert((pred, cursor)) {
                break;
            }
            edges.push((pred, cursor));
            cursor = pred;
        }
        edges
    }

    fn symbolic_assert_condition(
        &mut self,
        cond: &mir::Operand<'tcx>,
        expected: bool,
    ) -> Rc<SymbolicValue> {
        let cond_value = if let Some(place) = cond.place() {
            self.block_visitor
                .body_visitor
                .place_to_abstract_value
                .get(&place)
                .cloned()
                .unwrap_or_else(|| {
                    let path = self.block_visitor.get_path_for_place(&place);
                    self.block_visitor.body_visitor.lookup_path_and_refine_result(
                        path,
                        self.block_visitor.body_visitor.context.tcx.types.bool,
                    )
                })
        } else {
            symbolic_value::TOP.into()
        };
        if expected {
            cond_value
        } else {
            cond_value.logical_not()
        }
    }

    fn dominating_assert_conditions(&mut self, bb: mir::BasicBlock) -> Vec<Rc<SymbolicValue>> {
        let mut conditions = Vec::new();
        for (pred, target) in self.straight_line_predecessor_edges(bb, 8) {
            let terminator = self.block_visitor.mir[pred].terminator();
            if let TerminatorKind::Assert {
                cond,
                expected,
                target: assert_target,
                ..
            } = &terminator.kind
            {
                if *assert_target == target {
                    conditions.push(self.symbolic_assert_condition(cond, *expected));
                }
            }
        }
        conditions
    }

    fn add_symbolic_condition_to_state(
        state: &mut AbstractDomain<DomainType>,
        condition: Rc<SymbolicValue>,
    ) {
        if let Ok(system) = LinearConstraintSystem::try_from(condition) {
            state.numerical_domain.add_constraints(system);
        }
    }

    fn enrich_state_with_dominating_asserts(
        &mut self,
        bb: mir::BasicBlock,
        state: &mut AbstractDomain<DomainType>,
    ) {
        for condition in self.dominating_assert_conditions(bb) {
            Self::add_symbolic_condition_to_state(state, condition);
        }
    }

    fn condition_left_operand<'a>(
        condition: &'a Rc<SymbolicValue>,
    ) -> Option<&'a Rc<SymbolicValue>> {
        match &condition.expression {
            Expression::LessThan { left, .. } | Expression::LessOrEqual { left, .. } => Some(left),
            _ => None,
        }
    }

    fn condition_matches_index(
        condition: &Rc<SymbolicValue>,
        index: &Rc<SymbolicValue>,
    ) -> bool {
        Self::condition_left_operand(condition)
            .map(|left| **left == **index)
            .unwrap_or(false)
    }

    fn dominating_bounds_prove_index(
        &mut self,
        bb: mir::BasicBlock,
        index: &Rc<SymbolicValue>,
    ) -> bool {
        self.dominating_assert_conditions(bb)
            .into_iter()
            .any(|condition| Self::condition_matches_index(&condition, index))
    }

    fn emit_call_boundary_diagnostic(&mut self, api_name: &str, reason: &str) {
        self.record_call_boundary();
        self.forget_destination_value();
        self.forget_possible_callee_side_effects();
        let warning = self
            .block_visitor
            .body_visitor
            .context
            .session
            .dcx()
            .struct_span_warn(
                self.block_visitor.body_visitor.current_span,
                format!(
                    "[Bypasser] Interprocedural call downgraded to local unknown at `{}`: {}",
                    api_name, reason
                ),
            );
        self.block_visitor.body_visitor.emit_diagnostic(
            warning,
            false,
            DiagnosticCause::CallBoundary,
        );
    }

    pub fn handle_opaque_call_boundary(&mut self) -> bool {
        let api_name = self
            .callee_func_ref
            .as_ref()
            .map(|func_ref| func_ref.function_name.clone())
            .unwrap_or_else(|| "unknown call".to_string().into());
        self.emit_call_boundary_diagnostic(
            &api_name,
            "default callee descent is disabled outside the special-API and micro-wrapper fragment",
        );
        true
    }

    /// If the current call is to a well known function for which we don't have a cached summary,
    /// this function will update the environment as appropriate and return true. If the return
    /// result is false, just carry on with the normal logic.
    pub fn handled_as_special_function_call(&mut self) -> bool {
        #[allow(irrefutable_let_patterns)]
        let destination_path = if let dest = self.destination {
            Some(self.block_visitor.get_path_for_place(&dest))
        } else {
            None
        };
        match self.callee_known_name {
            KnownNames::VecFromRawParts => {
                return self.emit_unsupported_special_call(
                    "Vec::from_raw_parts",
                    "heap-backed reconstruction and ownership transfer are outside the supported fragment",
                );
            }
            KnownNames::BypasserVerify => {
                assert!(self.actual_args.len() == 1);
                debug!("Handling special function BypasserVerify");
                // if self.block_visitor.body_visitor.check_for_errors {
                self.report_calls_to_special_functions();
                // }
                // self.actual_args = &self.actual_args[0..1];
                // self.handle_assume();
                return true;
            }
            KnownNames::RustDealloc => {
                return true;
            }
            KnownNames::RustAlloc | KnownNames::RustAllocZeroed => {
                return self.emit_unsupported_special_call(
                    "__rust_alloc",
                    "heap allocation is outside the supported numerical fragment",
                );
            }
            KnownNames::StdPanickingBeginPanic | KnownNames::StdPanickingBeginPanicFmt => {
                self.handle_panic();
                return true;
            }
            KnownNames::StdIntoVec => {
                return self.emit_unsupported_special_call(
                    "Into<Vec<_>>",
                    "container reconstruction is outside the supported fragment",
                );
            }
            KnownNames::CoreOpsIndex => {
                if self.supports_local_bounds_check() {
                    self.record_supported_special_call();
                    self.handle_index();
                } else {
                    self.emit_unsupported_special_call(
                        "Index::index",
                        "only local bounds checks over primitive slice/array elements are supported",
                    );
                }
                return true;
            }
            KnownNames::StdPtrMutPtrOffset
            | KnownNames::StdPtrConstPtrOffset
            | KnownNames::StdPtrMutPtrAdd
            | KnownNames::StdPtrConstPtrAdd
            | KnownNames::StdPtrMutPtrSub
            | KnownNames::StdPtrConstPtrSub
            | KnownNames::StdPtrConstPtrWrappingOffset
            | KnownNames::StdPtrMutPtrWrappingOffset
            | KnownNames::StdPtrMutPtrWrappingAdd
            | KnownNames::StdPtrConstPtrWrappingAdd
            | KnownNames::StdPtrMutPtrWrappingSub
            | KnownNames::StdPtrConstPtrWrappingSub => {
                return self.emit_unsupported_special_call(
                    "pointer::offset/add/sub",
                    "pointer arithmetic and alias-sensitive memory reasoning are outside the supported fragment",
                );
            }
            KnownNames::StdPtrMutPtrByteOffset
            | KnownNames::StdPtrConstPtrByteOffset
            | KnownNames::StdPtrMutPtrByteAdd
            | KnownNames::StdPtrConstPtrByteAdd
            | KnownNames::StdPtrMutPtrByteSub
            | KnownNames::StdPtrConstPtrByteSub
            | KnownNames::StdPtrConstPtrWrappingByteOffset
            | KnownNames::StdPtrMutPtrWrappingByteOffset
            | KnownNames::StdPtrMutPtrWrappingByteAdd
            | KnownNames::StdPtrConstPtrWrappingByteAdd
            | KnownNames::StdPtrMutPtrWrappingByteSub
            | KnownNames::StdPtrConstPtrWrappingByteSub => {
                return self.emit_unsupported_special_call(
                    "pointer::byte_offset/byte_add/byte_sub",
                    "byte-level pointer arithmetic is outside the supported fragment",
                );
            }
            KnownNames::StdPtrConstPtrOffsetFrom | KnownNames::StdPtrMutPtrOffsetFrom => {
                return self.emit_unsupported_special_call(
                    "pointer::offset_from",
                    "relational pointer reasoning is outside the supported fragment",
                );
            }
            KnownNames::StdPtrConstPtrByteOffsetFrom | KnownNames::StdPtrMutPtrByteOffsetFrom => {
                return self.emit_unsupported_special_call(
                    "pointer::byte_offset_from",
                    "relational pointer reasoning is outside the supported fragment",
                );
            }
            KnownNames::StdSliceIndexGetUncheckedMut => {
                if self.supports_local_bounds_check() {
                    self.record_supported_special_call();
                    return self.handle_get_unchecked_mut();
                }
                return self.emit_unsupported_special_call(
                    "slice::get_unchecked_mut",
                    "only local bounds checks over primitive slice/array elements are supported",
                );
            }
            KnownNames::StdSliceIndexGetUnchecked => {
                if self.supports_local_bounds_check() {
                    self.record_supported_special_call();
                    return self.handle_get_unchecked();
                }
                return self.emit_unsupported_special_call(
                    "slice::get_unchecked",
                    "only local bounds checks over primitive slice/array elements are supported",
                );
            }
            KnownNames::StdSliceIndexGet => {
                if self.supports_local_bounds_check() {
                    self.record_supported_special_call();
                    return self.handle_get_checked();
                }
                return self.emit_unsupported_special_call(
                    "slice::get",
                    "only local bounds checks over primitive slice/array elements are supported",
                );
            }
            KnownNames::StdSliceIndexGetMut => {
                if self.supports_local_bounds_check() {
                    self.record_supported_special_call();
                    return self.handle_get_checked();
                }
                return self.emit_unsupported_special_call(
                    "slice::get_mut",
                    "only local bounds checks over primitive slice/array elements are supported",
                );
            }
            // KnownNames::StdSliceIndexGetMut =>{
            //     return self.handle_get_checked_mut();
            // }
            KnownNames::StdSliceSplitAt | KnownNames::StdSliceSplitAtMut => {
                if self.supports_local_bounds_check() {
                    self.record_supported_special_call();
                    return self.handle_split_at_checked();
                }
                return self.emit_unsupported_special_call(
                    "slice::split_at[_mut]",
                    "only local split-index checks over primitive slice/array elements are supported",
                );
            }
            // KnownNames::StdSliceSplitAtMut =>{
            //     return self.handle_split_at_mut_checked();
            // }
            KnownNames::StdIntCheckedAdd => {
                if self
                    .actual_argument_types
                    .first()
                    .copied()
                    .map(Self::is_supported_integer_type)
                    .unwrap_or(false)
                {
                    self.record_supported_special_call();
                    return self.handle_checked_add();
                }
                return self.emit_unsupported_special_call(
                    "checked_add",
                    "only integer scalar checked_add calls are supported",
                );
            }
            KnownNames::StdSliceSwap => {
                if self.supports_local_swap_check() {
                    self.record_supported_special_call();
                    return self.handle_swap();
                }
                return self.emit_unsupported_special_call(
                    "slice::swap",
                    "only local index bounds checks over slice/array receivers are supported",
                );
            }
            KnownNames::StdFrom => {
                if self.supports_bool_to_usize_from() {
                    self.record_supported_special_call();
                    return self.handle_bool_to_usize_from();
                }
                return self.emit_unsupported_special_call(
                    "From",
                    "only the local bool-to-usize conversion pattern is supported",
                );
            }
            KnownNames::StdAsMutPtr => {
                return self.emit_unsupported_special_call(
                    "as_mut_ptr",
                    "reference and ownership conversion side effects are outside the supported fragment",
                );
            }
            _ => {
                let result = self.try_to_inline_special_function();
                if !result.is_bottom() {
                    if let Some(target_path) = destination_path {
                        // let target_path = self.block_visitor.visit_place(place);
                        self.block_visitor
                            .body_visitor
                            .state
                            .update_value_at(target_path.clone(), result);
                        // let exit_condition = self.block_visitor.state.entry_condition.clone();
                        // self.block_visitor
                        //     .state
                        //     .exit_conditions
                        //     .insert(*target, exit_condition);
                        return true;
                    }
                }
            }
        }
        false
    }

    /// If the function being called is a special function like mirai_annotations.mirai_verify or
    /// std.panicking.begin_panic then report a diagnostic or create a precondition as appropriate.
    fn report_calls_to_special_functions(&mut self) {
        match self.callee_known_name {
            KnownNames::BypasserVerify => {
                assert!(self.actual_args.len() == 1); // The type checker ensures this.
                let (_, cond) = &self.actual_args[0];
                // let message = self.coerce_to_string(&self.actual_args[1].1);
                let message = Rc::new(String::from("dummy message"));
                self.block_visitor.check_condition(cond, message, false);
            }
            _ => unreachable!(),
        }
    }

    /// Provides special handling of functions that have no MIR bodies or that need to access
    /// internal MIRAI state in ways that cannot be expressed in normal Rust and therefore
    /// cannot be summarized in the standard_contracts crate.
    /// Returns the result of the call, or BOTTOM if the function to call is not a known
    /// special function.
    fn try_to_inline_special_function(&mut self) -> Rc<SymbolicValue> {
        match self.callee_known_name {
            KnownNames::StdMemSizeOf => self.handle_size_of(),
            _ => symbolic_value::BOTTOM.into(),
        }
    }

    // /// Removes the heap block and all paths rooted in it from the current environment.
    // fn handle_rust_dealloc(&mut self) -> Rc<SymbolicValue> {
    //     assert!(self.actual_args.len() == 3);
    //     // The current environment is that that of the caller, but the caller is a standard
    //     // library function and has no interesting state to purge.
    //     // The layout path inserted below will become a side effect of the caller and when that
    //     // side effect is refined by the caller's caller, the refinement will do the purge if the
    //     // qualifier of the path is a heap block path.
    //     // Get path to the heap block to deallocate
    //     let heap_block_path = self.actual_args[0].0.clone();
    //     // Create a layout
    //     let length = self.actual_args[1].1.clone();
    //     let alignment = self.actual_args[2].1.clone();
    //     let layout = SymbolicValue::make_from(
    //         Expression::HeapBlockLayout {
    //             length,
    //             alignment,
    //             source: LayoutSource::DeAlloc,
    //         },
    //         1,
    //     );
    //     // Get a layout path and update the environment
    //     let layout_path =
    //         Path::new_layout(heap_block_path).refine_paths(&self.block_visitor.state());
    //     self.block_visitor
    //         .body_visitor
    //         .state
    //         .update_value_at(layout_path, layout);
    //     // Signal to the caller that there is no return result
    //     symbolic_value::BOTTOM.into()
    // }

    /// Gets the size in bytes of the type parameter T of the std::mem::size_of<T> function.
    /// Returns and unknown value of type u128 if T is not a concrete type.
    fn handle_size_of(&mut self) -> Rc<SymbolicValue> {
        assert!(self.actual_args.is_empty());
        let sym = rustc_span::Symbol::intern("T");
        let t = (self.callee_generic_argument_map.as_ref())
            .expect("std::mem::size_of must be called with generic arguments")
            .get(&sym)
            .expect("std::mem::size must have generic argument T");
        // let param_env = self
        //     .block_visitor
        //     .body_visitor
        //     .context
        //     .tcx
        //     .param_env(self.callee_def_id);
        if let Ok(ty_and_layout) = self
            .block_visitor
            .body_visitor
            .type_visitor
            .layout_of(*t)
        {
            Rc::new((ty_and_layout.layout.size.bytes() as u128).into())
        } else {
            // SymbolicValue::make_typed_unknown(ExpressionType::U128)
            Rc::new(symbolic_value::TOP)
        }
    }

    fn handle_panic(&mut self) {
        assert!(self.actual_args.len() == 1);
        // assert!(self.destination.is_none());
        let body_visitor = &mut self.block_visitor.body_visitor;
        if !body_visitor.state.is_bottom() {
            let warning = body_visitor.context.session.dcx().struct_span_warn(
                body_visitor.current_span,
                format!("[Bypasser] Possible error: run into panic code"),
            );
            // body_visitor.emit_diagnostic(warning, false, DiagnosticCause::Panic);
            warning.cancel();
        }
    }


    fn handle_get_checked(&mut self) -> bool{
        assert!(self.actual_args.len() == 2);
        #[allow(irrefutable_let_patterns)]
        let destination_path = if let dest = self.destination {
            Some(self.block_visitor.get_path_for_place(&dest))
        } else {
            None
        };
        assert!(destination_path.is_some());
        let state = self.block_visitor.state().clone();
        let body_visitor = &mut self.block_visitor.body_visitor;

        let array = &self.actual_args[0].0;
        let array_len = Path::new_length(array.clone()).refine_paths(&body_visitor.state);
        let array_len_val = SymbolicValue::make_from(
            Expression::Variable {
                path: array_len.clone(),
                var_type: ExpressionType::Usize,
            },
            1,
        );
        let index_val = &self.actual_args[1].1;

        let assert_checker = AssertionChecker::new(body_visitor);
        let overflow_safe_cond = SymbolicValue::make_from(
            Expression::LessThan {
                left: index_val.clone(),
                right: array_len_val,
            },
            1,
        );
        let check_result = assert_checker.check_assert_condition(overflow_safe_cond.clone(), true, &state);
        // let info = body_visitor.context.session.dcx().struct_span_warn(
        //             body_visitor.current_span,
        //             format!("[Rust-API-Bypass] There's a get() call"),
        //         );
        // info.emit();
        // // 发出诊断信息
        // match check_result {
        //     CheckerResult::Safe => (),
        //     CheckerResult::Unsafe => {
        //         let error = body_visitor.context.session.dcx().struct_span_warn(
        //             body_visitor.current_span,
        //             format!("[Rust-API-Bypass] Provably error: index out of bound in get() call"),
        //         );
        //         error.emit();
        //     }
        //     CheckerResult::Warning => {
        //         let warning = body_visitor.context.session.dcx().struct_span_warn(
        //             body_visitor.current_span,
        //             format!("[Rust-API-Bypass] Possible error: index out of bound in get() call"),
        //         );
        //         // warning.emit();
        //         warning.cancel();
        //     }
        // }

        match check_result {
            CheckerResult::Safe => {}
            CheckerResult::Unsafe => {
                let error = body_visitor.context.session.dcx().struct_span_warn(
                    body_visitor.current_span,
                    "[Bypasser] Provably error: index out of bound in get()/get_mut()",
                );
                body_visitor.emit_diagnostic(error, false, DiagnosticCause::Index);
            }
            CheckerResult::Warning => {
                let warning = body_visitor.context.session.dcx().struct_span_warn(
                    body_visitor.current_span,
                    "[Bypasser] Possible error: index may be out of bound in get()/get_mut()",
                );
                body_visitor.emit_diagnostic(warning, false, DiagnosticCause::Index);
            }
        }

        // We only claim local bounds reasoning here; the returned Option value is left unknown.
        self.forget_destination_value();
        true
    }

    // fn handle_get_checked_mut(&mut self) -> bool{
    //     assert!(self.actual_args.len() == 2);
    //     #[allow(irrefutable_let_patterns)]
    //     let destination_path = if let dest = self.destination {
    //         Some(self.block_visitor.get_path_for_place(&dest))
    //     } else {
    //         None
    //     };
    //     assert!(destination_path.is_some());
    //     let state = self.block_visitor.state().clone();
    //     let body_visitor = &mut self.block_visitor.body_visitor;
    //     let array = &self.actual_args[0].0;
    //     let array_len = Path::new_length(array.clone()).refine_paths(&body_visitor.state);
    //     let array_len_val = SymbolicValue::make_from(
    //         Expression::Variable {
    //             path: array_len.clone(),
    //             var_type: ExpressionType::Usize,
    //         },
    //         1,
    //     );
    //     let index_val = &self.actual_args[1].1;
    //     let assert_checker = AssertionChecker::new(body_visitor);
    //     let overflow_safe_cond = SymbolicValue::make_from(
    //         Expression::LessThan {
    //             left: index_val.clone(),
    //             right: array_len_val,
    //         },
    //         1,
    //     );
    //     let check_result = assert_checker.check_assert_condition(overflow_safe_cond.clone(), true, &state);
    //     let info = body_visitor.context.session.dcx().struct_span_warn(
    //                 body_visitor.current_span,
    //                 format!("[Rust-API-Bypass] There's a get_mut() call"),
    //             );
    //     info.emit();
    //     // 发出诊断信息
    //     match check_result {
    //         CheckerResult::Safe => (),
    //         CheckerResult::Unsafe => {
    //             let error = body_visitor.context.session.dcx().struct_span_warn(
    //                 body_visitor.current_span,
    //                 format!("[Rust-API-Bypass] Provably error: index out of bound in get_mut() call"),
    //             );
    //             error.emit();
    //         }
    //         CheckerResult::Warning => {
    //             let warning = body_visitor.context.session.dcx().struct_span_warn(
    //                 body_visitor.current_span,
    //                 format!("[Rust-API-Bypass] Possible error: index out of bound in get_mut() call"),
    //             );
    //             // warning.emit();
    //             warning.cancel();
    //         }
    //     }
    //     // 为 get_mut 方法创建 Option<&mut T> 类型的返回值
    //     if let Some(target_path) = destination_path {
    //         // 获取数组元素的类型
    //         let _element_type = get_element_type(self.actual_argument_types[0]);
    //         // 根据边界检查结果创建不同的返回值
    //         let result_val = match check_result {
    //             CheckerResult::Safe | CheckerResult::Warning => {
    //                 // 安全情况：返回 Some(&mut element)
    //                 let array_path = &self.actual_args[0].0;
    //                 let indexed_path = Path::new_index(array_path.clone(), index_val.clone())
    //                     .refine_paths(&self.block_visitor.body_visitor.state);                   
    //                 // 返回可变引用的路径
    //                 let mutable_ref_path = Path::new_deref(indexed_path)
    //                     .refine_paths(&self.block_visitor.body_visitor.state);         
    //                 SymbolicValue::make_from(
    //                     Expression::Variable {
    //                         path: mutable_ref_path,
    //                         var_type: ExpressionType::NonPrimitive,
    //                     },
    //                     1,
    //                 )
    //             },
    //             CheckerResult::Unsafe => {
    //                 // 不安全情况：返回表示 None 的值
    //                 SymbolicValue::make_from(
    //                     Expression::CompileTimeConstant(ConstantValue::Bottom),
    //                     1,
    //                 )
    //             }
    //         };       
    //         // 更新目标路径的值
    //         self.block_visitor
    //             .body_visitor
    //             .state
    //             .update_value_at(target_path.clone(), result_val);               
    //         return true;
    //     }
    //     return false;
    // }

    fn handle_split_at_checked(&mut self) -> bool{
        assert!(self.actual_args.len() == 2);
        #[allow(irrefutable_let_patterns)]
        let destination_path = if let dest = self.destination {
            Some(self.block_visitor.get_path_for_place(&dest))
        } else {
            None
        };
        assert!(destination_path.is_some());
        let state = self.block_visitor.state().clone();
        let body_visitor = &mut self.block_visitor.body_visitor;

        let array = &self.actual_args[0].0;
        let array_len = Path::new_length(array.clone()).refine_paths(&body_visitor.state);
        let array_len_val = SymbolicValue::make_from(
            Expression::Variable {
                path: array_len.clone(),
                var_type: ExpressionType::Usize,
            },
            1,
        );
        let index_val = &self.actual_args[1].1;

        let assert_checker = AssertionChecker::new(body_visitor);
        let overflow_safe_cond = SymbolicValue::make_from(
            Expression::LessOrEqual {
                left: index_val.clone(),
                right: array_len_val,
            },
            1,
        );
        let check_result = assert_checker.check_assert_condition(overflow_safe_cond.clone(), true, &state);
        // let info = body_visitor.context.session.dcx().struct_span_warn(
        //             body_visitor.current_span,
        //             format!("[Rust-API-Bypass] There's a split_at() call"),
        //         );
        // info.emit();
        // // 发出诊断信息
        // match check_result {
        //     CheckerResult::Safe => (),
        //     CheckerResult::Unsafe => {
        //         let error = body_visitor.context.session.dcx().struct_span_warn(
        //             body_visitor.current_span,
        //             format!("[Rust-API-Bypass] Provably error: index out of bound in split_at_checked() call"),
        //         );
        //         error.emit();
        //     }
        //     CheckerResult::Warning => {
        //         let warning = body_visitor.context.session.dcx().struct_span_warn(
        //             body_visitor.current_span,
        //             format!("[Rust-API-Bypass] Possible error: index out of bound in split_at_checked() call"),
        //         );
        //         // warning.emit();
        //         warning.cancel();
        //     }
        // }

        match check_result {
            CheckerResult::Safe => {}
            CheckerResult::Unsafe => {
                let error = body_visitor.context.session.dcx().struct_span_warn(
                    body_visitor.current_span,
                    "[Bypasser] Provably error: split index out of bound in split_at[_mut]()",
                );
                body_visitor.emit_diagnostic(error, false, DiagnosticCause::Index);
            }
            CheckerResult::Warning => {
                let warning = body_visitor.context.session.dcx().struct_span_warn(
                    body_visitor.current_span,
                    "[Bypasser] Possible error: split index may be out of bound in split_at[_mut]()",
                );
                body_visitor.emit_diagnostic(warning, false, DiagnosticCause::Index);
            }
        }

        // We only claim local split-index reasoning here; the returned slices are left unknown.
        self.forget_destination_value();
        true
    }

    // fn handle_split_at_mut_checked(&mut self) -> bool{
    //     assert!(self.actual_args.len() == 2);
    //     #[allow(irrefutable_let_patterns)]
    //     let destination_path = if let dest = self.destination {
    //         Some(self.block_visitor.get_path_for_place(&dest))
    //     } else {
    //         None
    //     };
    //     assert!(destination_path.is_some());
    //     let state = self.block_visitor.state().clone();
    //     let body_visitor = &mut self.block_visitor.body_visitor;
    //     let array = &self.actual_args[0].0;
    //     let array_len = Path::new_length(array.clone()).refine_paths(&body_visitor.state);
    //     let array_len_val = SymbolicValue::make_from(
    //         Expression::Variable {
    //             path: array_len.clone(),
    //             var_type: ExpressionType::Usize,
    //         },
    //         1,
    //     );
    //     let index_val = &self.actual_args[1].1;
    //     let assert_checker = AssertionChecker::new(body_visitor);
    //     let overflow_safe_cond = SymbolicValue::make_from(
    //         Expression::LessOrEqual {
    //             left: index_val.clone(),
    //             right: array_len_val,
    //         },
    //         1,
    //     );
    //     let check_result = assert_checker.check_assert_condition(overflow_safe_cond.clone(), true, &state);
    //     let info = body_visitor.context.session.dcx().struct_span_warn(
    //                 body_visitor.current_span,
    //                 format!("[Rust-API-Bypass] There's a split_at_mut() call"),
    //             );
    //     info.emit();
    //     // 发出诊断信息
    //     match check_result {
    //         CheckerResult::Safe => (),
    //         CheckerResult::Unsafe => {
    //             let error = body_visitor.context.session.dcx().struct_span_warn(
    //                 body_visitor.current_span,
    //                 format!("[Rust-API-Bypass] Provably error: index out of bound in split_at_mut_checked() call"),
    //             );
    //             error.emit();
    //         }
    //         CheckerResult::Warning => {
    //             let warning = body_visitor.context.session.dcx().struct_span_warn(
    //                 body_visitor.current_span,
    //                 format!("[Rust-API-Bypass] Possible error: index out of bound in split_at_mut_checked() call"),
    //             );
    //             // warning.emit();
    //             warning.cancel();
    //         }
    //     }
    //     // 为 split_at_mut_checked 方法创建 Option<(&mut [T], &mut [T])> 类型的返回值
    //     if let Some(target_path) = destination_path {
    //         // 根据边界检查结果创建不同的返回值
    //         let result_val = match check_result {
    //             CheckerResult::Safe => {
    //                 // 安全情况：返回 Some((left_slice, right_slice))
    //                 // 创建表示可变切片元组的符号值
    //                 SymbolicValue::make_from(
    //                     Expression::Variable {
    //                         path: target_path.clone(),
    //                         var_type: ExpressionType::NonPrimitive,
    //                     },
    //                     1,
    //                 )
    //             },
    //             CheckerResult::Unsafe => {
    //                 // 不安全情况：返回 None
    //                 SymbolicValue::make_from(
    //                     Expression::CompileTimeConstant(ConstantValue::Bottom),
    //                     1,
    //                 )
    //             },
    //             CheckerResult::Warning => {
    //                 // 警告情况：返回未知的 Option 值
    //                 SymbolicValue::make_from(
    //                     Expression::Variable {
    //                         path: target_path.clone(),
    //                         var_type: ExpressionType::NonPrimitive,
    //                     },
    //                     1,
    //                 )
    //             }
    //         };
    //         // 更新目标路径的值
    //         self.block_visitor
    //             .body_visitor
    //             .state
    //             .update_value_at(target_path.clone(), result_val);
    //         return true;
    //     }
    //     return false;
    // }


    fn handle_checked_add(&mut self) -> bool {
        assert!(self.actual_args.len() == 2);
        #[allow(irrefutable_let_patterns)]
        let destination_path = if let dest = self.destination {
            Some(self.block_visitor.get_path_for_place(&dest))
        } else { None };
        let state = self.block_visitor.state().clone();
        let elem_ty = self.actual_argument_types[0];
        let body_visitor = &mut self.block_visitor.body_visitor;

        // lhs and rhs symbolic values
        let lhs_val = &self.actual_args[0].1; // self
        let rhs_val = &self.actual_args[1].1; // rhs

        // Create expression lhs + rhs
        let sum_val = SymbolicValue::make_from(
            Expression::Add { left: lhs_val.clone(), right: rhs_val.clone() },
            1,
        );

        // Use AssertionChecker::check_within_range on the sum with element type
        let assert_checker = AssertionChecker::new(body_visitor);
        let check_result = assert_checker.check_within_range(
            Path::new_alias(sum_val.clone()),
            elem_ty,
            &state,
        );
        match check_result {
            CheckerResult::Safe => (),
            CheckerResult::Unsafe => {
                let error = body_visitor.context.session.dcx().struct_span_warn(
                    body_visitor.current_span,
                    format!("[Bypasser] Provably error: integer overflow in checked_add()"),
                );
                body_visitor.emit_diagnostic(error, false, DiagnosticCause::Arithmetic);
            }
            CheckerResult::Warning => {
                let warning = body_visitor.context.session.dcx().struct_span_warn(
                    body_visitor.current_span,
                    format!("[Bypasser] Possible error: integer overflow in checked_add()"),
                );
                body_visitor.emit_diagnostic(warning, false, DiagnosticCause::Arithmetic);
            }
        }

        let _ = destination_path;
        let _ = sum_val;
        // We only claim local overflow reasoning here; the returned Option is left unknown.
        self.forget_destination_value();
        true
    }

    fn handle_swap(&mut self) -> bool {
        assert!(self.actual_args.len() == 3);
        let mut state = self.block_visitor.state().clone();
        self.enrich_state_with_dominating_asserts(self.block_visitor.current_block, &mut state);
        let index_a_already_proved = self
            .dominating_bounds_prove_index(self.block_visitor.current_block, &self.actual_args[1].1);
        let index_b_already_proved = self
            .dominating_bounds_prove_index(self.block_visitor.current_block, &self.actual_args[2].1);
        let body_visitor = &mut self.block_visitor.body_visitor;

        let array = &self.actual_args[0].0;
        let array_len = Path::new_length(array.clone()).refine_paths(&body_visitor.state);
        let array_len_val = SymbolicValue::make_from(
            Expression::Variable {
                path: array_len,
                var_type: ExpressionType::Usize,
            },
            1,
        );
        let index_a_val = &self.actual_args[1].1;
        let index_b_val = &self.actual_args[2].1;

        let index_a_safe_cond = SymbolicValue::make_from(
            Expression::LessThan {
                left: index_a_val.clone(),
                right: array_len_val.clone(),
            },
            1,
        );
        let check_result_a = if index_a_already_proved {
            CheckerResult::Safe
        } else {
            let assert_checker = AssertionChecker::new(body_visitor);
            assert_checker.check_assert_condition(index_a_safe_cond, true, &state)
        };
        let index_b_safe_cond = SymbolicValue::make_from(
            Expression::LessThan {
                left: index_b_val.clone(),
                right: array_len_val,
            },
            1,
        );
        let check_result_b = if index_b_already_proved {
            CheckerResult::Safe
        } else {
            let assert_checker = AssertionChecker::new(body_visitor);
            assert_checker.check_assert_condition(index_b_safe_cond, true, &state)
        };

        match check_result_a {
            CheckerResult::Safe => {}
            CheckerResult::Unsafe => {
                let error = body_visitor.context.session.dcx().struct_span_warn(
                    body_visitor.current_span,
                    "[Bypasser] Provably error: first index out of bound in swap()",
                );
                body_visitor.emit_diagnostic(error, false, DiagnosticCause::Index);
            }
            CheckerResult::Warning => {
                let warning = body_visitor.context.session.dcx().struct_span_warn(
                    body_visitor.current_span,
                    "[Bypasser] Possible error: first index may be out of bound in swap()",
                );
                body_visitor.emit_diagnostic(warning, false, DiagnosticCause::Index);
            }
        }

        match check_result_b {
            CheckerResult::Safe => {}
            CheckerResult::Unsafe => {
                let error = body_visitor.context.session.dcx().struct_span_warn(
                    body_visitor.current_span,
                    "[Bypasser] Provably error: second index out of bound in swap()",
                );
                body_visitor.emit_diagnostic(error, false, DiagnosticCause::Index);
            }
            CheckerResult::Warning => {
                let warning = body_visitor.context.session.dcx().struct_span_warn(
                    body_visitor.current_span,
                    "[Bypasser] Possible error: second index may be out of bound in swap()",
                );
                body_visitor.emit_diagnostic(warning, false, DiagnosticCause::Index);
            }
        }

        // We do not model the post-swap contents precisely; forget sequence-derived facts.
        body_visitor.state.forget_paths_rooted_by(array);
        true
    }

    fn handle_bool_to_usize_from(&mut self) -> bool {
        assert!(self.actual_args.len() == 1);
        #[allow(irrefutable_let_patterns)]
        let destination_path = if let dest = self.destination {
            Some(self.block_visitor.get_path_for_place(&dest))
        } else {
            None
        };
        if let Some(target_path) = destination_path {
            self.block_visitor
                .body_visitor
                .state
                .numerical_domain
                .assign_interval(
                    target_path,
                    Interval::new(Bound::from(0u128), Bound::from(1u128)),
                );
        } else {
            self.forget_destination_value();
        }
        true
    }

    fn handle_get_unchecked(&mut self) -> bool {
        assert!(self.actual_args.len() == 2);
        #[allow(irrefutable_let_patterns)]
        let destination_path = if let dest = self.destination {
            Some(self.block_visitor.get_path_for_place(&dest))
        } else {
            None
        };
        assert!(destination_path.is_some());
        let state = self.block_visitor.state().clone();
        let body_visitor = &mut self.block_visitor.body_visitor;

        let array = &self.actual_args[0].0;
        let array_len = Path::new_length(array.clone()).refine_paths(&body_visitor.state);
        let array_len_val = SymbolicValue::make_from(
            Expression::Variable {
                path: array_len.clone(),
                var_type: ExpressionType::Usize,
            },
            1,
        );
        let index_val = &self.actual_args[1].1;
        let _result = destination_path.as_ref().unwrap();

        let assert_checker = AssertionChecker::new(body_visitor);
        let overflow_safe_cond = SymbolicValue::make_from(
            Expression::LessThan {
                left: index_val.clone(),
                right: array_len_val,
            },
            1,
        );
        let check_result = assert_checker.check_assert_condition(overflow_safe_cond, true, &state);
        //  TODO: 相同Span只能发出一次诊断，未发出的诊断会由编译器进行报错。
        match check_result {
            CheckerResult::Safe => (),
            CheckerResult::Unsafe => {
                let error = body_visitor.context.session.dcx().struct_span_warn(
                    body_visitor.current_span,
                    format!("[Bypasser] Provably error: index out of bound",),
                );
                // error.emit();
                body_visitor.emit_diagnostic(error, false, DiagnosticCause::Index);
                //return;
            }
            CheckerResult::Warning => {
                let warning = body_visitor.context.session.dcx().struct_span_warn(
                    body_visitor.current_span,
                    "[Bypasser] Possible error: index may be out of bound",
                );
                body_visitor.emit_diagnostic(warning, false, DiagnosticCause::Index);
            }
        }

        // We only claim local bounds reasoning here; the returned reference/value is left unknown.
        self.forget_destination_value();
        true
    }

    fn handle_get_unchecked_mut(&mut self) -> bool {
        assert!(self.actual_args.len() == 2);
        #[allow(irrefutable_let_patterns)]
        let destination_path = if let dest = self.destination {
            Some(self.block_visitor.get_path_for_place(&dest))
        } else {
            None
        };
        assert!(destination_path.is_some());
        let state = self.block_visitor.state().clone();
        let body_visitor = &mut self.block_visitor.body_visitor;

        let array = &self.actual_args[0].0;
        let array_len = Path::new_length(array.clone()).refine_paths(&body_visitor.state);
        let array_len_val = SymbolicValue::make_from(
            Expression::Variable {
                path: array_len.clone(),
                var_type: ExpressionType::Usize,
            },
            1,
        );
        let index_val = &self.actual_args[1].1;
        let _result = destination_path.as_ref().unwrap();

        let assert_checker = AssertionChecker::new(body_visitor);
        let overflow_safe_cond = SymbolicValue::make_from(
            Expression::LessThan {
                left: index_val.clone(),
                right: array_len_val,
            },
            1,
        );
        let check_result = assert_checker.check_assert_condition(overflow_safe_cond, true, &state);
        //  TODO: 相同Span只能发出一次诊断，未发出的诊断会由编译器进行报错。
        match check_result {
            CheckerResult::Safe => (),
            CheckerResult::Unsafe => {
                let error = body_visitor.context.session.dcx().struct_span_warn(
                    body_visitor.current_span,
                    format!("[Bypasser] Provably error: index out of bound",),
                );
                // error.emit();
                body_visitor.emit_diagnostic(error, false, DiagnosticCause::Index);
                //return;
            }
            CheckerResult::Warning => {
                let warning = body_visitor.context.session.dcx().struct_span_warn(
                    body_visitor.current_span,
                    "[Bypasser] Possible error: index may be out of bound",
                );
                body_visitor.emit_diagnostic(warning, false, DiagnosticCause::Index);
            }
        }

        // We only claim local bounds reasoning here; the returned mutable reference is left unknown.
        self.forget_destination_value();
        true
    }

    // _17(place) = index(move _18 move _19])
    fn handle_index(&mut self) {
        assert!(self.actual_args.len() == 2);
        #[allow(irrefutable_let_patterns)]
        let destination_path = if let dest = self.destination {
            Some(self.block_visitor.get_path_for_place(&dest))
        } else {
            None
        };
        assert!(destination_path.is_some());
        let state = self.block_visitor.state().clone();
        let body_visitor = &mut self.block_visitor.body_visitor;

        let array = &self.actual_args[0].0;
        let array_len = Path::new_length(array.clone()).refine_paths(&body_visitor.state);
        let array_len_val = SymbolicValue::make_from(
            Expression::Variable {
                path: array_len.clone(),
                var_type: ExpressionType::Usize,
            },
            1,
        );
        let index_val = &self.actual_args[1].1;
        let result = destination_path.as_ref().unwrap();

        let assert_checker = AssertionChecker::new(body_visitor);
        let overflow_safe_cond = SymbolicValue::make_from(
            Expression::LessThan {
                left: index_val.clone(),
                right: array_len_val,
            },
            1,
        );
        let check_result = assert_checker.check_assert_condition(overflow_safe_cond, true, &state);

        match check_result {
            CheckerResult::Safe => (),
            CheckerResult::Unsafe => {
                let error = body_visitor.context.session.dcx().struct_span_warn(
                    body_visitor.current_span,
                    format!("[Bypasser] Provably error: index out of bound",),
                );

                body_visitor.emit_diagnostic(error, false, DiagnosticCause::Index);
                return;
            }
            CheckerResult::Warning => {
                let warning = body_visitor.context.session.dcx().struct_span_warn(
                    body_visitor.current_span,
                    format!("[Bypasser] Possible error: index out of bound"),
                );
                body_visitor.emit_diagnostic(warning, false, DiagnosticCause::Index);
            }
        }

        let _ = result;
        // We only claim local bounds reasoning here; the returned reference is left unknown.
        self.forget_destination_value();
    }

    /// Returns a list of (path, value) pairs where each path is rooted by an argument (or the result).
    /// In numerical-only mode we do not propagate heap-reachable side effects.
    fn extract_side_effects(
        &self,
        env: &AbstractDomain<DomainType>,
        argument_count: usize,
        offset: usize,
    ) -> Vec<(Rc<Path>, Rc<SymbolicValue>)> {
        let mut result = Vec::new();
        for ordinal in 0..=argument_count {
            let root = if ordinal == 0 {
                Path::new_result()
            } else {
                Path::new_parameter(ordinal, offset)
            };

            // `path` is `result`, or `path` is rooted by `result` or parameters
            for path in env
                .get_paths_iter()
                .iter()
                .filter(|p| (ordinal == 0 && (**p) == root) || p.is_rooted_by(&root))
            {
                if let Some(value) = env.value_at(path) {
                    if let Expression::Variable { path: vpath, .. } = &value.expression {
                        if ordinal > 0 && vpath.eq(path) {
                            // The value is not an update, but just what was there at function entry.
                            // TODO: path=path, when will this happen?
                            continue;
                        }
                    }
                    // We are extracting a subset of information out of env, which has not overflowed.
                    result.push((path.clone(), value.clone()));
                }
            }
        }
        result
    }

    /// Updates the current state to reflect the effects of a normal return from the function call.
    pub fn transfer_and_refine_normal_return_state(
        &mut self,
        function_post_state: &AbstractDomain<DomainType>,
        old_offset: usize,
    ) {
        self.block_visitor.body_visitor.state = function_post_state.clone();

        debug!("Start to transfer and refine normal return state");
        #[allow(irrefutable_let_patterns)]
        let destination_path = if let dest = self.destination {
            Some(self.block_visitor.get_path_for_place(&dest))
        } else {
            None
        };
        // let destination = self.destination.clone();
        // debug!("destination: {:?}", destination);
        if let Some(target_path) = &destination_path {
            // Assign function result to target path
            debug!("target_path: {:?}", target_path);
            let return_value_path = Path::new_result();

            let side_effects =
                self.extract_side_effects(function_post_state, self.actual_args.len(), old_offset);

            debug!("side_effects: {:?}", side_effects);

            // Transfer side effects
            if !function_post_state.is_empty() {
                // TODO
                // Effects on the heap
                // debug!("Handling side effects on the heap");
                // for (path, value) in side_effects.iter() {
                //     if path.is_rooted_by_abstract_heap_block() {
                //         let rvalue = value
                //             .clone()
                //             .refine_parameters(
                //                 self.actual_args,
                //                 self.block_visitor.body_visitor.fresh_variable_offset,
                //             )
                //             .refine_paths(&self.block_visitor.state);
                //         self.block_visitor
                //             .state
                //             .update_value_at(path.clone(), rvalue);
                //     }
                //     // check_for_early_return!(self.block_visitor.body_visitor);
                // }

                // TODO
                // Effects on the call result
                debug!("Handling side effects on call result");
                self.block_visitor.transfer_and_refine(
                    &side_effects,
                    target_path.clone(),
                    &return_value_path,
                    self.actual_args,
                );

            // todo!(" Maybe we should delete all the param values in symbolic domain?");

            // Effects on the call arguments
            // debug!("Handling side effects on call arguments");
            // for (i, (target_path, _)) in self.actual_args.iter().enumerate() {
            //     let parameter_path = Path::new_parameter(i + 1);
            //     self.block_visitor.transfer_and_refine(
            //         &side_effects,
            //         target_path.clone(),
            //         &parameter_path,
            //         self.actual_args,
            //     );
            //     // check_for_early_return!(self.block_visitor.body_visitor);
            // }
            }
            // funtion_post_state is empty
            else {
                // TODO
                debug!("funtion_post_state is empty");
                // We don't know anything other than the return value type.
                // We'll assume there were no side effects and no preconditions (but check this later if possible).
                // let result_type = self
                //     .block_visitor
                //     .body_visitor
                //     .type_visitor
                //     .get_place_type(place, self.block_visitor.current_span);
                let _result_type: ExpressionType = self
                    .block_visitor
                    .body_visitor
                    .type_visitor
                    .get_path_rustc_type(target_path, self.block_visitor.body_visitor.current_span)
                    .kind()
                    .into();
                // let result = SymbolicValue::make_from(
                //     Expression::UninterpretedCall {
                //         callee: self.callee_fun_val.clone(),
                //         arguments: self
                //             .actual_args
                //             .iter()
                //             .map(|(_, arg)| arg.clone())
                //             .collect(),
                //         result_type,
                //         path: return_value_path.clone(),
                //     },
                //     1,
                // );
                let result = symbolic_value::TOP.into();
                debug!("Before updating top: {:?}", self.block_visitor.state());
                self.block_visitor
                    .body_visitor
                    .state
                    .update_value_at(return_value_path, result);
            }
        }

        // 该部分是新增的删除掉此前重复调用产生的符号域内容的功能 并且由日志输出记录, 由gpt生成
        self.block_visitor
            .body_visitor
            .state
            .drop_call_frame_vars_from(old_offset);
        debug!(
            "after call-frame cleanup: numerical={}",
            self.block_visitor.body_visitor.state.numerical_domain.get_paths_iter().len(),
        );
    }
}
