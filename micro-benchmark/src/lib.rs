#![feature(rustc_private)]
#![feature(box_patterns)]
extern crate rustc_driver;
extern crate rustc_interface;
extern crate rustc_middle;
extern crate rustc_hir;
extern crate rustc_data_structures;
extern crate rustc_session;
extern crate rustc_span;
extern crate rustc_abi;

pub mod callback;
pub mod options;