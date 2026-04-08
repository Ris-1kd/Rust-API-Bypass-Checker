#![feature(rustc_private)]

fn main() {
    std::process::exit(rust_api_bypass::driver::run_from_env_args());
}
