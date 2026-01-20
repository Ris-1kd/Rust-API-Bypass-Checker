// 该文件定义的结构体是处理我们在遇到成对的函数的时候, 在对应的处理函数中应该把函数的信息都整合起来, 存储到调用该函数的调用者所属的WTO Visitor结构体中的一个HashMap或者HashSet中. 
// 虽然DefId可以唯一的标识一个库函数, 但是考虑到一个库函数可以在多处被调用的情况, 我们还是把DefId+Span同时作为区分, 不用DefId单独来区分, 所以直接在WTO visitor中用HashSet来收集

use std::rc::Rc;
use rustc_hir::def_id::DefId;
use rustc_span::Span;
use crate::analysis::memory::{known_names::KnownNames, path::Path, symbolic_value::SymbolicValue};
use std::collections::HashSet;
use std::collections::HashMap;
use log::debug;

#[derive(Debug, Hash,PartialEq, Eq, Clone, Copy)]
pub enum FuncClass {
    Index,
    NonZero,
    Overflow,
    String,
    Nullness,
}

#[derive(Debug,Clone)]
pub struct FunctionBase{
    pub value: HashMap<FuncClass, HashSet<String>>
}

impl FunctionBase {
    pub fn new() -> Self{
        let mut functions = HashMap::new();
        functions.insert(
            FuncClass::Index, 
            HashSet::from([
                "core::slice::<impl [T]>::get".to_string(),
                "core::slice::<impl [T]>::get_mut".to_string(),
                "core::slice::<impl [T]>::swap".to_string(),
                "core::slice::<impl [T]>::split_at".to_string(),
                "core::slice::<impl [T]>::split_at_mut".to_string(),
                "std::char::from_u32".to_string(),
                "std::array::ascii::<impl [u8; N]>::as_ascii".to_string(),
                "core::num::<impl u8>::as_ascii".to_string(),
                "core::str::<impl str>::get:".to_string(),
                "core::str::<impl str>::get_mut:".to_string(),
                "std::alloc::Layout::from_size_align".to_string(),
                "std::ptr::Alignment::new".to_string(),
                "std::simd::Mask::<i32, 4>::test".to_string(),
                "std::simd::Mask::<i32, 4>::set".to_string(),
            ])
        );
        functions.insert(
            FuncClass::NonZero, 
            HashSet::from([
                "core::slice::<impl [T]>::as_chunks".to_string(),
                "core::slice::<impl [T]>::as_chunks_mut".to_string(),
                "std::num::NonZero::<T>::from_mut".to_string(),
                "std::num::NonZero::<T>::new".to_string(),
            ])
        );

        functions.insert(
            FuncClass::Overflow, 
            HashSet::from([
                "core::num::<impl i32>::checked_add".to_string(),
                "core::num::<impl i32>::checked_mul".to_string(),
                "core::num::<impl i32>::checked_neg".to_string(),
                "core::num::<impl i32>::checked_shl".to_string(),
                "core::num::<impl i32>::checked_shr".to_string(),
                "core::num::<impl i32>::checked_sub".to_string(),
                "core::num::<impl u32>::checked_add".to_string(),
                "core::num::<impl u32>::checked_mul".to_string(),
                "core::num::<impl u32>::checked_shl".to_string(),
                "core::num::<impl u32>::checked_shr".to_string(),
                "core::num::<impl u32>::checked_sub".to_string(),
                "std::num::NonZero::<u32>::checked_add".to_string(),
                "std::num::NonZero::<u32>::checked_mul".to_string(),
            ])
        );
        functions.insert(
            FuncClass::String, 
            HashSet::from([
                "std::ffi::CStr::from_bytes_with_nul".to_string(),
                "std::ffi::CString::from_vec_with_nul".to_string(),
                "std::ffi::CString::new".to_string(),
                "std::str::from_utf8".to_string(),
                "std::str::from_utf8_mut".to_string(),
                "std::string::String::from_utf8".to_string(),
            ])
        );
        functions.insert(
            FuncClass::Nullness, 
            HashSet::from([
                "std::ptr::NonNull::<T>::new".to_string(),
                "std::ptr::Unique::<T>::new".to_string(),
                "std::ptr::const_ptr::<impl *const T>::as_ref".to_string(),
                "std::ptr::mut_ptr::<impl *mut T>::as_mut".to_string(),
                "std::ptr::mut_ptr::<impl *mut T>::as_ref".to_string(),
            ])
        );
        Self {value:functions}
    }

    pub fn show(&self){
        debug!("The function we care:");
        debug!("{:?}",self.value);
    }

    pub fn contains_and_get_kind(&self, func_name:&String) -> Option<FuncClass>{
        for (kind, set) in &self.value{
            if set.contains(func_name) {
                return Some(kind.clone());
            }
        }
        None
    }
}

#[derive(Debug,Hash,Eq,PartialEq,Clone)]
pub struct FuncHandler{
    /// The DefId of the target function
    pub def_id: DefId,

    /// The category we manually class of the funciton, like index, type or nullptr
    pub class: FuncClass,

    /// The location where the func was called
    pub span: Span,

    /// The vector of the params, 
    pub args: Vec<(Rc<Path>, Rc<SymbolicValue>)>,

    /// The known name in the call visitor
    pub callee_known_name:KnownNames,

    /// Maybe should set a reference of the body_visitor or the assert_checker.
    pub to_be_done:u32,
}


impl FuncHandler{
    pub fn new(
        def_id:DefId,
        class:FuncClass,
        span:Span,
        args: Vec<(Rc<Path>, Rc<SymbolicValue>)>,
        callee_known_name: KnownNames,
        to_be_done:u32
    )->Self{
        Self { def_id, 
            class, 
            span, 
            args: args.clone(), 
            callee_known_name, 
            to_be_done 
        }
    }

}