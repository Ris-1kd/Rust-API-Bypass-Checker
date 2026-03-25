use std::{env, path::Path, process::Command};

fn main() {
    // Cargo 调用形式： wrapper <path-to-rustc> <rustc-args...>
    let mut args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("usage: mir_wrapper <rustc-path> <rustc-args...>");
        std::process::exit(2);
    }

    let rustc_path = args[1].clone();
    let rustc_args = args.split_off(2);

    // 关键：默认只对“主包”做分析，其他依赖直接透传给 rustc
    // 否则你会在依赖 crate 上反复触发“没有 main/没有入口”的问题，几乎必炸。
    let is_primary = env::var("CARGO_PRIMARY_PACKAGE").ok().as_deref() == Some("1");
    if !is_primary {
        let status = Command::new(&rustc_path)
            .args(&rustc_args)
            .status()
            .expect("failed to exec rustc");
        std::process::exit(status.code().unwrap_or(1));
    }

    // 这里开始：对主包执行你的分析（典型是 rustc_driver::RunCompiler）
    // 你需要把 rustc_path + rustc_args 重新拼成 rustc_driver 期望的参数数组：
    let mut full = Vec::with_capacity(1 + rustc_args.len());
    full.push(rustc_path.clone());
    full.extend(rustc_args);

    // 如果你的 driver 需要 sysroot，而 full 中没有 --sysroot，建议补上（后面再做也行）
    // run_analysis(full);
    

    // 先给你一个“能跑通编译链路”的占位：直接调用 rustc（确保 wrapper 能工作）
    // 确认没问题后，再把这里替换为你的 rustc_driver 分析入口。
    let status = Command::new(&rustc_path)
        .args(&full[1..])
        .status()
        .expect("failed to exec rustc");
    std::process::exit(status.code().unwrap_or(1));
}
