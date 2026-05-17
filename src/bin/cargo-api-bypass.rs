#![feature(rustc_private)]

/// This file is derived from miri:
/// https://github.com/rust-lang/miri/blob/master/src/bin/cargo-miri.rs
use rust_api_bypass::utils;
use serde_json;
use std::ffi::OsString;
use std::path::Path;
use std::process::Command;

const CARGO_BYPASSER_HELP: &str = r#"Bypasser static analysis tool for Rust programs

Usage:
    cargo api-bypass
"#;

fn show_help() {
    println!("{}", CARGO_BYPASSER_HELP);
}

fn show_version() {
    println!("rust-bypasser {}", env!("CARGO_PKG_VERSION"));
}

fn show_error(msg: String) -> ! {
    eprintln!("fatal error: {}", msg);
    std::process::exit(1)
}

// Determines whether a flag `name` is present before `--`.
// For example, has_arg_flag("-v")
fn has_arg_flag(name: &str) -> bool {
    let mut args = std::env::args().take_while(|val| val != "--");
    args.any(|val| val == name)
}

// Gets the value of a `name`.
// For example, get_arg_flag_value("--manifest-path")
// Supports two styles: `--name value` or `--name=value`
fn get_arg_flag_value(name: &str) -> Option<String> {
    // Stop searching at `--`.
    let mut args = std::env::args().take_while(|val| val != "--");
    loop {
        let arg = match args.next() {
            Some(arg) => arg,
            None => return None,
        };
        if !arg.starts_with(name) {
            continue;
        }
        // Strip leading `name`.
        let suffix = &arg[name.len()..];
        if suffix.is_empty() {
            // This argument is exactly `name`; the next one is the value.
            return args.next();
        } else if suffix.starts_with('=') {
            // This argument is `name=value`; get the value.
            // Strip leading `=`.
            return Some(suffix[1..].to_owned());
        }
    }
}

// Get the top level crate that we need to analyze
fn current_crate() -> cargo_metadata::Package {
    // We need to get the manifest, and then the metadata, to enumerate targets.

    // Path to the `Cargo.toml` file
    let manifest_path =
        get_arg_flag_value("--manifest-path").map(|m| Path::new(&m).canonicalize().unwrap());

    let mut cmd = cargo_metadata::MetadataCommand::new();
    if let Some(ref manifest_path) = manifest_path {
        cmd.manifest_path(manifest_path);
    }
    let mut metadata = if let Ok(metadata) = cmd.exec() {
        metadata
    } else {
        show_error("Could not obtain Cargo metadata; likely an ill-formed manifest".to_string());
    };

    let current_dir = std::env::current_dir();

    let package_index = metadata
        .packages
        .iter()
        .position(|package| {
            let package_manifest_path = Path::new(&package.manifest_path);
            if let Some(ref manifest_path) = manifest_path {
                package_manifest_path == manifest_path
            } else {
                let current_dir = current_dir
                    .as_ref()
                    .expect("could not read current directory");
                let package_manifest_directory = package_manifest_path
                    .parent()
                    .expect("could not find parent directory of package manifest");
                package_manifest_directory == current_dir
            }
        })
        .unwrap_or_else(|| {
            show_error(
                "This seems to be a workspace, which is not supported by cargo-miri".to_string(),
            )
        });
    let package = metadata.packages.remove(package_index);

    package
}

fn bypasser() -> Command {
    let mut path = std::env::current_exe().expect("current executable path invalid");
    path.set_file_name("api-bypass");
    Command::new(path)
}

fn cargo() -> Command {
    Command::new(std::env::var_os("CARGO").unwrap_or_else(|| OsString::from("cargo")))
}

fn forward_to_rustc() -> ! {
    let first = std::env::args().nth(1).unwrap_or_else(|| "rustc".to_string());
    let first_is_rustc = Path::new(&first)
        .file_name()
        .and_then(|n| n.to_str())
        .map(|file_name| file_name == "rustc")
        .unwrap_or(false);
    let mut cmd = if first_is_rustc {
        let mut cmd = Command::new(first);
        cmd.args(std::env::args().skip(2));
        cmd
    } else {
        let mut cmd = Command::new("rustc");
        cmd.args(std::env::args().skip(1));
        cmd
    };
    let status = cmd
        .status()
        .expect("could not forward invocation to rustc");
    std::process::exit(status.code().unwrap_or(-1))
}

fn main() {
    if let Some(arg1) = std::env::args().nth(1) {
        let tool_subcommand_mode = Path::new(&arg1)
            .file_name()
            .and_then(|n| n.to_str())
            .map(|file_name| matches!(file_name, "api-bypass" | "bypasser" | "mir-checker"))
            .unwrap_or(false);
        let rustc_version_query = std::env::args()
            .skip(1)
            .any(|arg| matches!(arg.as_str(), "--version" | "-V" | "-vV"));
        if !tool_subcommand_mode && rustc_version_query {
            forward_to_rustc();
        }
    }

    // Check for version and help flags even when invoked through the cargo wrapper.
    if std::env::args().any(|a| a == "--help" || a == "-h") {
        show_help();
        return;
    }
    if std::env::args().any(|a| a == "--version" || a == "-V") {
        show_version();
        return;
    }

    // 获取第一个参数并解析为路径
    if let Some(arg1) = std::env::args().nth(1) {
        let path = Path::new(&arg1);
        if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
            match file_name {
                "api-bypass" | "bypasser" | "mir-checker" => {
                    // 当以 `cargo api-bypass` 调用时执行此分支
                    in_cargo_bypasser();
                }
                "rustc" => {
                    // 当以 wrapper 方式运行 `cargo rustc`，且 `RUSTC_WRAPPER` 环境变量设置为自身时执行此分支
                    inside_cargo_rustc();
                }
                _ => {
                    show_error(format!(
                        "`cargo-api-bypass` must be called with either `api-bypass`/`bypasser` or `rustc` as first argument.",
                    ));
                }
            }
        } else {
            show_error("无法解析第一个参数的文件名。".to_string());
        }
    } else {
        show_error("缺少必要的命令行参数。".to_string());
    }

}

// This will construct command line like:
// `cargo rustc --bin some_crate_name -v -- cargo-api-bypass-marker-begin --top_crate_name some_top_crate_name --domain interval -v cargo-api-bypass-marker-end`
// And set the following environment variables:
// `RUSTC_WRAPPER` is set to `cargo-api-bypass` itself so the execution will come back to the second branch as described above
// `BYPASSER_ARGS` is set to the user-provided arguments for `bypasser`
// `BYPASSER_TOP_CRATE_NAME` is set to the name of the crate being analyzed
// `BYPASSER_VERBOSE` is set if `-v` is provided
fn in_cargo_bypasser() {
    let verbose = has_arg_flag("-v");

    let current_crate = current_crate();

    // Now run the command.
    for target in current_crate.targets.into_iter() {
        let mut args = std::env::args().skip(2);
        // let kind = target
        //     .kind
        //     .get(0)
        //     .expect("badly formatted cargo metadata: target::kind is an empty array");

        // Now we run `cargo rustc $FLAGS $ARGS`, giving the user the
        // chance to add additional arguments. `FLAGS` is set to identify
        // this target. The user gets to control what gets actually passed to bypasser.
        let mut cmd = cargo();
        cmd.arg("check"); // using `check` may speed up the analysis than using `rustc`
        // match kind.as_str() {
        //     "bin" => {
        //         cmd.arg("--bin").arg(target.name);
        //     }
        //     "lib" => {
        //         cmd.arg("--lib");
        //     }
        //     _ => continue,
        // }

        // modified here to fix the "TargetKind" compilation error.
        if target.kind.iter().any(|k| k.to_string() == "bin") {
            cmd.arg("--bin").arg(&target.name);
        } else if target.kind.iter().any(|k| k.to_string() == "lib") {
            cmd.arg("--lib");
        } else {
            continue;
        }

        // Add cargo args until first `--`.
        while let Some(arg) = args.next() {
            if arg == "--" {
                break;
            }
            cmd.arg(arg);
        }

        // Serialize the remaining args into a special environemt variable.
        // This will be read by `inside_cargo_rustc` when we go to invoke
        // our actual target crate.
        // Since we're using "cargo check", we have no other way of passing
        // these arguments.
        // We also add `BYPASSER_TOP_CRATE_NAME` to specify the top-level
        // crate name that we want to analyze, by doing this we can dispatch
        // dependencies to the real `rustc` and top-level crate to `bypasser`
        let args_vec: Vec<String> = args.collect();
        cmd.env(
            "BYPASSER_ARGS",
            serde_json::to_string(&args_vec).expect("failed to serialize args"),
        );
        cmd.env(
            "MIR_CHECKER_ARGS",
            serde_json::to_string(&args_vec).expect("failed to serialize args"),
        );
        cmd.env("BYPASSER_TOP_CRATE_NAME", current_crate.name.as_str().to_string());
        cmd.env("MIR_CHECKER_TOP_CRATE_NAME", current_crate.name.as_str().to_string());


        // Replace the rustc executable through RUSTC_WRAPPER environment variable
        let path = std::env::current_exe().expect("current executable path invalid");
        cmd.env("RUSTC_WRAPPER", path);

        if verbose {
            cmd.env("BYPASSER_VERBOSE", "");
            cmd.env("MIR_CHECKER_VERBOSE", ""); // compatibility
            eprintln!("+ {:?}", cmd);
        }

        // Execute cmd
        let exit_status = cmd
            .spawn()
            .expect("could not run cargo")
            .wait()
            .expect("failed to wait for cargo?");

        if !exit_status.success() {
            std::process::exit(exit_status.code().unwrap_or(-1))
        }
    }
}

// This will construct command line like:
// `api-bypass --crate-name some_crate_name --edition=2018 src/lib.rs --crate-type lib --domain interval`
// And sets the environment variable `BYPASSER_BE_RUSTC`
// if `bypasser` is going to analyze crates that are dependencies
fn inside_cargo_rustc() {
    let mut cmd = bypasser();
    cmd.args(std::env::args().skip(2)); // skip `cargo-api-bypass rustc`

    // Add sysroot
    let sysroot = utils::compile_time_sysroot().expect("Cannot find sysroot");
    cmd.arg("--sysroot");
    cmd.arg(sysroot);

    let top_crate_name = std::env::var("BYPASSER_TOP_CRATE_NAME")
        .or_else(|_| std::env::var("MIR_CHECKER_TOP_CRATE_NAME"))
        .expect("missing BYPASSER_TOP_CRATE_NAME");
    let top_crate_name = top_crate_name.replace("-", "_"); // Cargo seems to rename hyphens to underscores

    if get_arg_flag_value("--crate-name").as_deref() == Some(&top_crate_name) {
        // If we are analyzing the crate that we want to analyze, add args for `bypasser`
        let magic = std::env::var("BYPASSER_ARGS")
            .or_else(|_| std::env::var("MIR_CHECKER_ARGS"))
            .expect("missing BYPASSER_ARGS");
        let bypasser_args: Vec<String> =
            serde_json::from_str(&magic).expect("failed to deserialize BYPASSER_ARGS");
        cmd.args(bypasser_args);
    } else {
        // If we are analyzing dependencies, set this environment variable so
        // that `bypasser` will behave just like the real `rustc` and do the
        // compilation instead of analysis
        cmd.env("BYPASSER_BE_RUSTC", "1");
        cmd.env("MIR_CHECKER_BE_RUSTC", "1");
    }

    let verbose = std::env::var_os("BYPASSER_VERBOSE").is_some()
        || std::env::var_os("MIR_CHECKER_VERBOSE").is_some();
    if verbose {
        eprintln!("+ {:?}", cmd);
    }

    match cmd.status() {
        Ok(exit) => {
            if !exit.success() {
                std::process::exit(exit.code().unwrap_or(42));
            }
        }
        Err(ref e) => panic!("error during bypasser run: {:?}", e),
    }
}
