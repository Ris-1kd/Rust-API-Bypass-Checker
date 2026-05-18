// Compatibility wrapper for the numerical abstract domain.
//
// The analyzer now uses a single pure Rust interval domain. The public names in this module are
// kept temporarily so the MIR visitor/checker layers do not need a broad mechanical rename.

use crate::analysis::memory::path::Path;
use crate::analysis::numerical::interval::{Bound, Interval};
use crate::analysis::numerical::lattice::LatticeTrait;
use crate::analysis::numerical::linear_constraint::{
    LinearConstraint, LinearConstraintSystem, LinearExpression,
};
use crate::analysis::option::AbstractDomainType;
use rug::Integer;
use std::collections::BTreeMap;
use std::fmt::{self, Debug};
use std::marker::PhantomData;
use std::rc::Rc;

/// The operators that numerical abstract domain supports.
#[derive(Clone, Copy, Debug)]
pub enum ApronOperation {
    Add,
    Sub,
    Mul,
    Div,
    Rem,
    Shl,
    Shr,
    And,
    Or,
    Xor,
    Not,
    Neg,
}

#[derive(Clone)]
pub struct ApronInterval;
#[derive(Clone)]
pub struct ApronOctagon;
#[derive(Clone)]
pub struct ApronPolyhedra;
#[derive(Clone)]
pub struct ApronLinearEqualities;
#[derive(Clone)]
pub struct ApronPplPolyhedra;
#[derive(Clone)]
pub struct ApronPplLinearCongruences;
#[derive(Clone)]
pub struct ApronPkgridPolyhedraLinCongruences;

pub trait ApronDomainType: Clone {}

impl ApronDomainType for ApronInterval {}
impl ApronDomainType for ApronOctagon {}
impl ApronDomainType for ApronPolyhedra {}
impl ApronDomainType for ApronLinearEqualities {}
impl ApronDomainType for ApronPplPolyhedra {}
impl ApronDomainType for ApronPplLinearCongruences {}
impl ApronDomainType for ApronPkgridPolyhedraLinCongruences {}

pub trait GetManagerTrait {
    fn get_domain_type() -> AbstractDomainType;
}

/// A map-based interval abstract state. A missing variable means top.
#[derive(Clone)]
pub struct ApronAbstractDomain<Type>
where
    Type: ApronDomainType,
{
    intervals: BTreeMap<Rc<Path>, Interval>,
    bottom: bool,
    phantom: PhantomData<Type>,
}

impl GetManagerTrait for ApronAbstractDomain<ApronInterval> {
    fn get_domain_type() -> AbstractDomainType {
        AbstractDomainType::Interval
    }
}

impl GetManagerTrait for ApronAbstractDomain<ApronPolyhedra> {
    fn get_domain_type() -> AbstractDomainType {
        AbstractDomainType::Polyhedra
    }
}

impl GetManagerTrait for ApronAbstractDomain<ApronOctagon> {
    fn get_domain_type() -> AbstractDomainType {
        AbstractDomainType::Octagon
    }
}

impl GetManagerTrait for ApronAbstractDomain<ApronLinearEqualities> {
    fn get_domain_type() -> AbstractDomainType {
        AbstractDomainType::LinearEqualities
    }
}

impl GetManagerTrait for ApronAbstractDomain<ApronPplPolyhedra> {
    fn get_domain_type() -> AbstractDomainType {
        AbstractDomainType::PplPolyhedra
    }
}

impl GetManagerTrait for ApronAbstractDomain<ApronPplLinearCongruences> {
    fn get_domain_type() -> AbstractDomainType {
        AbstractDomainType::PplLinearCongruences
    }
}

impl GetManagerTrait for ApronAbstractDomain<ApronPkgridPolyhedraLinCongruences> {
    fn get_domain_type() -> AbstractDomainType {
        AbstractDomainType::PkgridPolyhedraLinCongruences
    }
}

impl<Type> Default for ApronAbstractDomain<Type>
where
    Type: ApronDomainType,
    ApronAbstractDomain<Type>: GetManagerTrait,
{
    fn default() -> Self {
        Self::top()
    }
}

impl<Type> LatticeTrait for ApronAbstractDomain<Type>
where
    Type: ApronDomainType,
    ApronAbstractDomain<Type>: GetManagerTrait,
{
    fn top() -> Self {
        Self {
            intervals: BTreeMap::new(),
            bottom: false,
            phantom: PhantomData,
        }
    }

    fn is_top(&self) -> bool {
        !self.bottom && self.intervals.is_empty()
    }

    fn set_to_top(&mut self) {
        self.bottom = false;
        self.intervals.clear();
    }

    fn bottom() -> Self {
        Self {
            intervals: BTreeMap::new(),
            bottom: true,
            phantom: PhantomData,
        }
    }

    fn is_bottom(&self) -> bool {
        self.bottom
    }

    fn set_to_bottom(&mut self) {
        self.bottom = true;
        self.intervals.clear();
    }

    fn lub(&self, other: &Self) -> Self {
        self.join(other)
    }

    fn widening_with(&self, other: &Self) -> Self {
        self.widening_with(other)
    }
}

impl<Type> ApronAbstractDomain<Type>
where
    Type: ApronDomainType,
    ApronAbstractDomain<Type>: GetManagerTrait,
{
    pub fn leq(&self, other: &Self) -> bool {
        if self.is_bottom() || other.is_top() {
            return true;
        }
        if other.is_bottom() {
            return self.is_bottom();
        }
        for (path, itv) in &self.intervals {
            if !interval_leq(itv, &other.var2itv(path)) {
                return false;
            }
        }
        for (path, other_itv) in &other.intervals {
            if !self.intervals.contains_key(path) && !other_itv.is_top() {
                return false;
            }
        }
        true
    }

    pub fn rename(&mut self, old_path: &Rc<Path>, new_path: &Rc<Path>) {
        if self.contains(old_path) {
            self.assign_var(new_path.clone(), old_path.clone());
            self.forget(old_path);
        }
    }

    pub fn duplicate(&mut self, old_path: &Rc<Path>, new_path: &Rc<Path>) {
        if self.contains(old_path) {
            self.assign_var(new_path.clone(), old_path.clone());
        }
    }

    pub fn get_paths_iter(&self) -> Vec<Rc<Path>> {
        self.intervals.keys().cloned().collect()
    }

    pub fn contains(&self, path: &Rc<Path>) -> bool {
        self.intervals.contains_key(path)
    }

    pub fn get_domain_type() -> AbstractDomainType {
        <Self as GetManagerTrait>::get_domain_type()
    }

    pub fn get_interval(&self, var: &Rc<Path>) -> Interval {
        self.var2itv(var)
    }

    pub fn assign_int(&mut self, var: Rc<Path>, n: Integer) {
        self.assign_interval(var, singleton(n));
    }

    pub fn assign_var(&mut self, var: Rc<Path>, rvalue: Rc<Path>) {
        let itv = self.var2itv(&rvalue);
        self.assign_interval(var, itv);
    }

    pub fn assign_interval(&mut self, var: Rc<Path>, itv: Interval) {
        self.set_interval(&var, itv);
    }

    pub fn narrowing_with(&self, rhs: &Self) -> Self {
        self.meet(rhs)
    }

    pub fn widening_with(&self, rhs: &Self) -> Self {
        if self.is_bottom() {
            return rhs.clone();
        }
        if rhs.is_bottom() {
            return self.clone();
        }
        let mut res = Self::top();
        for path in union_paths(self, rhs) {
            let old = self.var2itv(&path);
            let new = rhs.var2itv(&path);
            let widened = widen_interval(&old, &new);
            res.set_interval(&path, widened);
        }
        res
    }

    pub fn join(&self, rhs: &Self) -> Self {
        if self.is_bottom() || rhs.is_top() {
            return rhs.clone();
        }
        if rhs.is_bottom() || self.is_top() {
            return self.clone();
        }
        let mut res = Self::top();
        for path in union_paths(self, rhs) {
            let joined = join_interval(&self.var2itv(&path), &rhs.var2itv(&path));
            res.set_interval(&path, joined);
        }
        res
    }

    pub fn meet(&self, rhs: &Self) -> Self {
        if self.is_bottom() || rhs.is_bottom() {
            return Self::bottom();
        }
        if self.is_top() {
            return rhs.clone();
        }
        if rhs.is_top() {
            return self.clone();
        }
        let mut res = Self::top();
        for path in union_paths(self, rhs) {
            let met = meet_interval(&self.var2itv(&path), &rhs.var2itv(&path));
            if met.is_bottom() {
                return Self::bottom();
            }
            res.set_interval(&path, met);
        }
        res
    }

    pub fn apply_bin_op_place_place(
        &mut self,
        op: ApronOperation,
        lhs: &Rc<Path>,
        rhs: &Rc<Path>,
        res: &Rc<Path>,
    ) {
        if !self.is_bottom() {
            let lhs_itv = self.var2itv(lhs);
            let rhs_itv = self.var2itv(rhs);
            self.set_interval(res, eval_bin_op(op, lhs_itv, rhs_itv));
        }
    }

    pub fn apply_bin_op_const_place(
        &mut self,
        op: ApronOperation,
        cst: &Integer,
        rhs: &Rc<Path>,
        res: &Rc<Path>,
    ) {
        if !self.is_bottom() {
            let lhs_itv = singleton(cst.clone());
            let rhs_itv = self.var2itv(rhs);
            self.set_interval(res, eval_bin_op(op, lhs_itv, rhs_itv));
        }
    }

    pub fn apply_bin_op_place_const(
        &mut self,
        op: ApronOperation,
        lhs: &Rc<Path>,
        cst: &Integer,
        res: &Rc<Path>,
    ) {
        if !self.is_bottom() {
            let lhs_itv = self.var2itv(lhs);
            let rhs_itv = singleton(cst.clone());
            self.set_interval(res, eval_bin_op(op, lhs_itv, rhs_itv));
        }
    }

    pub fn apply_un_op_place(&mut self, op: ApronOperation, rhs: &Rc<Path>, res: &Rc<Path>) {
        if !self.is_bottom() {
            let rhs_itv = self.var2itv(rhs);
            let res_itv = match op {
                ApronOperation::Neg => negate_interval(rhs_itv),
                ApronOperation::Not => Interval::top(),
                _ => unreachable!("Undefined UnOp, this is a bug"),
            };
            self.set_interval(res, res_itv);
        }
    }

    pub fn forget(&mut self, var: &Rc<Path>) {
        self.intervals.remove(var);
    }

    pub fn add_constraints(&mut self, conds: LinearConstraintSystem) {
        if self.is_bottom() {
            return;
        }
        if conds.is_false() {
            self.set_to_bottom();
            return;
        }
        for cst in &conds {
            self.add_constraint(cst);
            if self.is_bottom() {
                return;
            }
        }
    }

    fn add_constraint(&mut self, cst: &LinearConstraint) {
        if cst.is_contradiction() {
            self.set_to_bottom();
            return;
        }
        if cst.is_tautology() {
            return;
        }
        match cst {
            LinearConstraint::Equality(expr) => self.refine_equality(expr),
            LinearConstraint::LessEq(expr) => self.refine_less_equal(expr),
            LinearConstraint::LessThan(_) => {
                let non_strict = cst.strict_to_non_strict();
                if let LinearConstraint::LessEq(expr) = non_strict {
                    self.refine_less_equal(&expr);
                }
            }
            LinearConstraint::Inequality(_) => {}
        }
    }

    fn refine_equality(&mut self, expr: &LinearExpression) {
        if let Some((path, low, high)) = interval_from_unary_expr(expr) {
            self.refine_path_interval(&path, Interval::new(low, high));
        }
    }

    fn refine_less_equal(&mut self, expr: &LinearExpression) {
        let Some((path, coeff, cst)) = unary_linear_expr(expr) else {
            return;
        };
        if coeff == 1 {
            self.refine_upper_bound(&path, -cst);
        } else if coeff == -1 {
            self.refine_lower_bound(&path, cst);
        }
    }

    fn refine_lower_bound(&mut self, path: &Rc<Path>, lower: Integer) {
        let old = self.var2itv(path);
        self.refine_path_interval(path, Interval::new(Bound::Int(lower), old.high.clone()));
    }

    fn refine_upper_bound(&mut self, path: &Rc<Path>, upper: Integer) {
        let old = self.var2itv(path);
        self.refine_path_interval(path, Interval::new(old.low.clone(), Bound::Int(upper)));
    }

    fn refine_path_interval(&mut self, path: &Rc<Path>, itv: Interval) {
        let met = meet_interval(&self.var2itv(path), &itv);
        if met.is_bottom() {
            self.set_to_bottom();
        } else {
            self.set_interval(path, met);
        }
    }

    fn var2itv(&self, var: &Rc<Path>) -> Interval {
        if self.is_bottom() {
            Interval::bottom()
        } else {
            self.intervals
                .get(var)
                .cloned()
                .unwrap_or_else(Interval::top)
        }
    }

    fn set_interval(&mut self, var: &Rc<Path>, itv: Interval) {
        if self.is_bottom() {
            return;
        }
        if itv.is_bottom() {
            self.set_to_bottom();
        } else if itv.is_top() {
            self.intervals.remove(var);
        } else {
            self.intervals.insert(var.clone(), itv);
        }
    }
}

impl<Type> Debug for ApronAbstractDomain<Type>
where
    Type: ApronDomainType,
    ApronAbstractDomain<Type>: GetManagerTrait,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let constraints = LinearConstraintSystem::from(self);
        write!(f, "{:?}", constraints)
    }
}

fn singleton(n: Integer) -> Interval {
    Interval::new(Bound::Int(n.clone()), Bound::Int(n))
}

fn union_paths<Type>(
    lhs: &ApronAbstractDomain<Type>,
    rhs: &ApronAbstractDomain<Type>,
) -> Vec<Rc<Path>>
where
    Type: ApronDomainType,
    ApronAbstractDomain<Type>: GetManagerTrait,
{
    let mut paths: Vec<Rc<Path>> = lhs.intervals.keys().cloned().collect();
    for path in rhs.intervals.keys() {
        if !paths.contains(path) {
            paths.push(path.clone());
        }
    }
    paths
}

fn interval_leq(lhs: &Interval, rhs: &Interval) -> bool {
    lhs.is_bottom()
        || rhs.is_top()
        || (!rhs.is_bottom() && rhs.low <= lhs.low && lhs.high <= rhs.high)
}

fn join_interval(lhs: &Interval, rhs: &Interval) -> Interval {
    if lhs.is_bottom() {
        rhs.clone()
    } else if rhs.is_bottom() {
        lhs.clone()
    } else {
        Interval::new(
            lhs.low.clone().min(rhs.low.clone()),
            lhs.high.clone().max(rhs.high.clone()),
        )
    }
}

fn meet_interval(lhs: &Interval, rhs: &Interval) -> Interval {
    if lhs.is_bottom() || rhs.is_bottom() {
        Interval::bottom()
    } else {
        Interval::new(
            lhs.low.clone().max(rhs.low.clone()),
            lhs.high.clone().min(rhs.high.clone()),
        )
    }
}

fn widen_interval(old: &Interval, new: &Interval) -> Interval {
    if old.is_bottom() {
        return new.clone();
    }
    if new.is_bottom() {
        return old.clone();
    }
    let low = if new.low < old.low {
        Bound::NINF
    } else {
        old.low.clone()
    };
    let high = if new.high > old.high {
        Bound::INF
    } else {
        old.high.clone()
    };
    Interval::new(low, high)
}

fn eval_bin_op(op: ApronOperation, lhs: Interval, rhs: Interval) -> Interval {
    if lhs.is_bottom() || rhs.is_bottom() {
        return Interval::bottom();
    }
    match op {
        ApronOperation::Add => lhs + rhs,
        ApronOperation::Sub => lhs - rhs,
        ApronOperation::Mul => lhs * rhs,
        ApronOperation::Div => {
            if interval_contains_zero(&rhs) {
                Interval::top()
            } else {
                lhs / rhs
            }
        }
        ApronOperation::Rem => Interval::top(),
        ApronOperation::Shl => lhs << rhs,
        ApronOperation::Shr => lhs >> rhs,
        ApronOperation::And => lhs & rhs,
        ApronOperation::Or => lhs | rhs,
        ApronOperation::Xor => lhs ^ rhs,
        ApronOperation::Not | ApronOperation::Neg => unreachable!("Undefined BinOp"),
    }
}

fn interval_contains_zero(itv: &Interval) -> bool {
    !itv.is_bottom()
        && itv.low <= Bound::Int(Integer::from(0))
        && Bound::Int(Integer::from(0)) <= itv.high
}

fn negate_interval(itv: Interval) -> Interval {
    Interval::new(negate_bound(itv.high), negate_bound(itv.low))
}

fn negate_bound(bound: Bound) -> Bound {
    match bound {
        Bound::INF => Bound::NINF,
        Bound::NINF => Bound::INF,
        Bound::Int(n) => Bound::Int(-n),
    }
}

fn unary_linear_expr(expr: &LinearExpression) -> Option<(Rc<Path>, Integer, Integer)> {
    if expr.term_count() != 1 {
        return None;
    }
    let (path, coeff) = expr.terms().next()?;
    Some((path.clone(), coeff.clone(), expr.constant()))
}

fn interval_from_unary_expr(expr: &LinearExpression) -> Option<(Rc<Path>, Bound, Bound)> {
    let (path, coeff, cst) = unary_linear_expr(expr)?;
    if coeff == 1 {
        let value = -cst;
        Some((path, Bound::Int(value.clone()), Bound::Int(value)))
    } else if coeff == -1 {
        Some((path, Bound::Int(cst.clone()), Bound::Int(cst)))
    } else {
        None
    }
}
