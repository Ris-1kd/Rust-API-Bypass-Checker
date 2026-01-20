
#![feature(rustc_private)]
extern crate rustc_driver;
extern crate rustc_interface;
extern crate rustc_middle;
extern crate rustc_hir;
extern crate rustc_span;
extern crate rustc_session;

use micro_benchmark::options::Cli;
use micro_benchmark::callback::AnalyzerCallbacks;
use clap::Parser;

use log::info;

fn main(){

    pretty_env_logger::init();

    let rustc_args:Vec<String> =std::env::args().collect();
    let clap_args = Cli::parse_from(&rustc_args);
    let mut callbacks = AnalyzerCallbacks::new();
    let run_compiler = rustc_driver::RunCompiler::new(&rustc_args, &mut callbacks);
    run_compiler.run();

}

