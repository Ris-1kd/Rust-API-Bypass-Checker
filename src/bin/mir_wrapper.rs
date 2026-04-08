#![feature(rustc_private)]
#![feature(box_patterns)]

extern crate rustc_driver;
extern crate rustc_errors;
extern crate rustc_interface;
extern crate rustc_middle;
extern crate rustc_session;



use std::{env, fs};
use serde_json::{json, Value};

fn main() {
    // 1) replay 模式：用于 IDE 单步调试
    {
        let mut it = env::args().skip(1);
        if let Some(a1) = it.next() {
            if a1 == "--replay" {
                let path = it.next().expect("usage: mir_wrapper --replay <invocation.json>");
                let text = fs::read_to_string(&path).expect("failed to read replay file");
                let v: Value = serde_json::from_str(&text).expect("invalid json replay file");

                // cwd
                if let Some(cwd) = v.get("cwd").and_then(|x| x.as_str()) {
                    env::set_current_dir(cwd).expect("failed to set cwd from replay");
                }

                // env（只恢复 dump 里保存的那些）
                if let Some(obj) = v.get("env").and_then(|x| x.as_object()) {
                    for (k, val) in obj {
                        if let Some(s) = val.as_str() {
                            env::set_var(k, s);
                        }
                    }
                }

                // argv
                let argv = v.get("argv")
                    .and_then(|x| x.as_array())
                    .expect("replay file missing argv")
                    .iter()
                    .map(|x| x.as_str().expect("argv must be string").to_string())
                    .collect::<Vec<_>>();

                let code = rust_api_bypass::driver::run_with_rustc_args(argv);
                std::process::exit(code);
            }
        }
    }

    // 2) wrapper 模式：Cargo 调用形式： mir_wrapper <rustc-path> <rustc-args...>
    let argv: Vec<String> = env::args().collect();
    if argv.len() < 2 {
        eprintln!("usage: mir_wrapper <rustc-path> <rustc-args...>");
        std::process::exit(2);
    }
    let rustc_path = argv[1].clone();
    let rustc_args = argv[2..].to_vec();

    // 非主包：直接透传 rustc，保证依赖/脚本/宏等正常编译
    let is_primary = env::var("CARGO_PRIMARY_PACKAGE").ok().as_deref() == Some("1");
    if !is_primary {
        // 用同一套 rustc_driver 编译依赖，避免 metadata 版本不一致
        let mut full = Vec::with_capacity(1 + rustc_args.len());
        full.push(rustc_path.clone());
        full.extend(rustc_args);

        std::env::set_var("BYPASSER_BE_RUSTC", "1");
        std::env::set_var("MIR_CHECKER_BE_RUSTC", "1");
        let code = rust_api_bypass::driver::run_with_rustc_args(full);
        std::env::remove_var("BYPASSER_BE_RUSTC");
        std::env::remove_var("MIR_CHECKER_BE_RUSTC");

        std::process::exit(code);
    }


    // 主包：调用你自己的 driver（同进程，便于断点）
    let mut full = Vec::with_capacity(1 + rustc_args.len() + 16);
    full.push(rustc_path.clone());
    full.extend(rustc_args);

    // 可选：用环境变量追加分析参数（格式：JSON 数组，例如 ["--entry","foo","--domain","interval"]）
    if let Ok(extra) = env::var("BYPASSER_ARGS").or_else(|_| env::var("MIR_CHECKER_ARGS")) {
        if let Ok(v) = serde_json::from_str::<Vec<String>>(&extra) {
            full.extend(v);
        } else {
            eprintln!("warning: BYPASSER_ARGS is not valid JSON array, ignored");
        }
    }

    // dump：把本次 invocation 存起来，后续可 --replay 调试
    if let Ok(dump_path) = env::var("MIR_WRAPPER_DUMP") {
        dump_invocation(&dump_path, &full);
    }

    let code = rust_api_bypass::driver::run_with_rustc_args(full);
    std::process::exit(code);
}

fn dump_invocation(path: &str, argv: &Vec<String>) {
    let cwd = env::current_dir()
        .ok()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|| ".".to_string());

    // 只保存一小撮关键 env，避免 dump 文件过大
    let keys = [
        "RUSTUP_TOOLCHAIN",
        "RUSTFLAGS",
        "CARGO_ENCODED_RUSTFLAGS",
        "CARGO_TARGET_DIR",
        "PATH",
        "RUST_BACKTRACE",
        "RUST_LOG",
        "BYPASSER_ARGS",
        "MIR_CHECKER_ARGS",
    ];

    let mut env_obj = serde_json::Map::new();
    for k in keys {
        if let Ok(v) = env::var(k) {
            env_obj.insert(k.to_string(), Value::String(v));
        }
    }

    let j = json!({
        "cwd": cwd,
        "argv": argv,
        "env": env_obj
    });

    let text = serde_json::to_string_pretty(&j).expect("json serialize failed");
    fs::write(path, text).expect("failed to write dump file");
}
