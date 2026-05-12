use crate::analysis::memory::constant_value::ConstantValue;
use crate::analysis::memory::expression::Expression;
use crate::analysis::memory::nullness_domain::{NullnessDomain, PointerNullness};
use crate::analysis::memory::path::{Path, PathEnum};
use crate::analysis::memory::symbolic_value::{SymbolicValue, SymbolicValueTrait};
use crate::analysis::numerical::apron_domain::{
    ApronAbstractDomain, ApronDomainType, GetManagerTrait,
};
use crate::analysis::numerical::lattice::LatticeTrait;
use rug::Integer;
use rustc_middle::mir;
use std::collections::{HashMap, HashSet};
use std::convert::TryFrom;
use std::fmt;
use std::rc::Rc;

#[derive(Clone)]
pub struct AbstractDomain<DomainType>
where
    DomainType: ApronDomainType,
    ApronAbstractDomain<DomainType>: GetManagerTrait,
{
    // Only stores the values of paths that are integers
    pub numerical_domain: ApronAbstractDomain<DomainType>,
    // Stores must-null / must-non-null facts for pointer-like values
    pub nullness_domain: NullnessDomain,
    // Stores branch conditions
    pub exit_conditions: HashMap<mir::BasicBlock, Rc<SymbolicValue>>,
}

impl<DomainType> fmt::Debug for AbstractDomain<DomainType>
where
    DomainType: ApronDomainType,
    ApronAbstractDomain<DomainType>: GetManagerTrait,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "numerical: {:?}, nullness: {:?}",
            self.numerical_domain, self.nullness_domain
        )
    }
}

impl<DomainType: ApronDomainType> AbstractDomain<DomainType>
where
    DomainType: ApronDomainType,
    ApronAbstractDomain<DomainType>: GetManagerTrait,
{
    // A bottom domain means the basic block is unreachable
    pub fn is_bottom(&self) -> bool {
        self.numerical_domain.is_bottom()
    }

    pub fn is_top(&self) -> bool {
        self.numerical_domain.is_top() && self.nullness_domain.is_empty()
    }

    pub fn leq(&self, other: &Self) -> bool {
        self.numerical_domain.leq(&other.numerical_domain)
            && self.nullness_domain.leq(&other.nullness_domain)
    }

    pub fn is_empty(&self) -> bool {
        self.numerical_domain.is_top() && self.nullness_domain.is_empty()
    }

    pub fn default() -> Self {
        Self {
            numerical_domain: ApronAbstractDomain::default(),
            nullness_domain: NullnessDomain::default(),
            exit_conditions: HashMap::new(),
        }
    }

    pub fn get_paths_iter(&self) -> Vec<Rc<Path>> {
        let paths: HashSet<Rc<Path>> = self
            .numerical_domain
            .get_paths_iter()
            .into_iter()
            .chain(self.nullness_domain.get_paths_iter())
            .collect();
        paths.into_iter().collect()
    }

    /// Drop all local/parameter variables that belong to callee call frames created with a fresh
    /// variable offset >= `cutoff_ordinal`.
    ///
    /// This is critical to avoid unbounded growth of the symbolic/numerical domains when calls
    /// occur inside loops (each call uses a new fresh offset, creating a new namespace).
    /// 
    /// 该函数是gpt生成用来进行清理symbolic domain中的重复参数调用的
    pub fn drop_call_frame_vars_from(&mut self, cutoff_ordinal: usize) {
        fn root_ordinal_if_local_or_param(path: &Rc<Path>) -> Option<usize> {
            match &path.value {
                PathEnum::QualifiedPath { qualifier, .. } => root_ordinal_if_local_or_param(qualifier),
                PathEnum::LocalVariable { ordinal } => Some(*ordinal),
                PathEnum::Parameter { ordinal } => Some(*ordinal),
                _ => None,
            }
        }

        // NOTE: use get_paths_iter() to cover BOTH symbolic-domain keys and numerical-domain vars.
        let to_remove: Vec<Rc<Path>> = self
            .get_paths_iter()
            .into_iter()
            .filter(|p| matches!(root_ordinal_if_local_or_param(p), Some(o) if o >= cutoff_ordinal))
            .collect();

        for p in to_remove {
            self.remove(&p); // forget in both symbolic and numerical domains
        }

        // Exit conditions from the callee must not leak into the caller.
        self.exit_conditions.clear();
    }


    pub fn remove(&mut self, path: &Rc<Path>) {
        self.numerical_domain.forget(path);
        self.nullness_domain.forget(path);
    }

    pub fn forget_paths_rooted_by(&mut self, root: &Rc<Path>) {
        let to_remove: Vec<Rc<Path>> = self
            .get_paths_iter()
            .into_iter()
            .filter(|path| **path == **root || path.is_rooted_by(root))
            .collect();
        for path in to_remove {
            self.remove(&path);
        }
    }

    pub fn rename(&mut self, old_path: &Rc<Path>, new_path: &Rc<Path>) {
        debug!("Renaming {:?} to {:?}", old_path, new_path);
        self.numerical_domain.rename(old_path, new_path);
        self.nullness_domain.rename(old_path, new_path);
    }

    pub fn duplicate(&mut self, old_path: &Rc<Path>, new_path: &Rc<Path>) {
        self.numerical_domain.duplicate(old_path, new_path);
    }

    /// Returns a reference to the value associated with the given path, if there is one.
    pub fn value_at(&self, path: &Rc<Path>) -> Option<Rc<SymbolicValue>> {
        if self.numerical_domain.contains(path) {
            let interval = self.numerical_domain.get_interval(path);
            if let Ok(const_int) = Integer::try_from(interval) {
                Some(SymbolicValue::make_from(
                    Expression::CompileTimeConstant(ConstantValue::Int(const_int)),
                    1,
                ))
            } else {
                let e = Expression::Numerical(path.clone());
                Some(SymbolicValue::make_from(e, 1))
            }
        } else {
            None
        }
    }

    /// Updates the path to value map so that the given path now points to the given value.
    pub fn update_value_at(&mut self, path: Rc<Path>, value: Rc<SymbolicValue>) {
        debug!("Updating value at {:?}, value: {:?}", path, value);

        if value.is_bottom() || value.is_top() {
            self.numerical_domain.forget(&path);
            return;
        }

        match &value.expression {
            Expression::Numerical(rpath) => {
                self.numerical_domain.assign_var(path.clone(), rpath.clone());
            }
            Expression::CompileTimeConstant(c) => {
                if let Some(i) = c.try_get_integer() {
                    self.numerical_domain.assign_int(path.clone(), i);
                } else {
                    self.numerical_domain.forget(&path);
                }
            }
            Expression::Variable { path: rpath, var_type } if var_type.is_integer() => {
                self.numerical_domain.assign_var(path.clone(), rpath.clone());
            }
            _ => {
                self.numerical_domain.forget(&path);
            }
        }
    }

    // pub fn update_value_at_backup(&mut self, path: Rc<Path>, value: Rc<SymbolicValue>) {
    //     debug!("Updating value at {:?}, value: {:?}", path, value);
    //     if value.is_bottom() || value.is_top() {
    //         debug!("Value is bottom or top, ignore");
    //         self.numerical_domain.forget(&path);
    //         return;
    //     }

    //     // Handle numerical values, store them in numerical domain
    //     // Case 1: value is already in numerical domain, so there is only a path in expression
    //     if let Expression::Numerical(rpath) = &value.expression {
    //         debug!("Value is numerical, store in numerical domain");
    //         self.numerical_domain.assign_var(path, rpath.clone());
    //     }
    //     // Case 2: value is a compile time constant, and is of type integer
    //     else if let Expression::CompileTimeConstant(constant_domain) = &value.expression {
    //         if let Some(integer) = constant_domain.try_get_integer() {
    //             debug!("Value is constant integer, store in numerical domain");
    //             self.numerical_domain.assign_int(path, integer);
    //         } else {
    //             debug!("Value is constant but not integer, store in symbolic domain");
    //         }
    //     }
    //     // Case 3: value is a variable of type integer
    //     else if let Expression::Variable {
    //         path: rpath,
    //         var_type,
    //     } = &value.expression
    //     {
    //         if var_type.is_integer() {
    //             debug!("Value is integer variable, store in both numerical and symbolic domain");
    //             self.numerical_domain
    //                 .assign_var(path.clone(), rpath.clone());

    //             // GPT给出意见在此处直接移除来防止symbolic domain大量膨胀
    //         } else {
    //             debug!("Value is a variable but not integer store in symbolic domain");
    //         }
    //     } else {
    //         // Reach here if value is not numerical, store them in symbolic domain
    //         debug!("Value is not numerical, store in symbolic domain");
    //     }
    // }

    pub fn join(&self, other: &Self) -> Self {
        let numerical = self.numerical_domain.join(&other.numerical_domain);
        Self {
            numerical_domain: numerical,
            exit_conditions: HashMap::new(),
        }
    }

    // TODO: implement meet for symbolic domain
    pub fn meet(&self, other: &Self) -> Self {
        let numerical = self.numerical_domain.meet(&other.numerical_domain);
        Self {
            numerical_domain: numerical,
            exit_conditions: HashMap::new(),
        }
    }

    pub fn widening_with(&self, other: &Self) -> Self {
        let numerical = self.numerical_domain.widening_with(&other.numerical_domain);
        Self {
            numerical_domain: numerical,
            exit_conditions: HashMap::new(),
        }
    }

    // TODO: implement narrowing for numerical domain and symbolic domain
    pub fn narrowing_with(&self, other: &Self) -> Self {
        let numerical = self
            .numerical_domain
            .narrowing_with(&other.numerical_domain);

        Self {
            numerical_domain: numerical,
            exit_conditions: HashMap::new(),
        }
    }

    pub fn subset(&self, other: &Self) -> bool {
        self.numerical_domain.leq(&other.numerical_domain)
    }
}
