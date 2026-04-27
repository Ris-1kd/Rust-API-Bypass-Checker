use crate::analysis::analyzer::analysis_trait::StaticAnalysis;
use crate::analysis::analyzer::numerical_analysis::NumericalAnalysis;
use crate::analysis::global_context::GlobalContext;
use crate::analysis::option::AnalysisOption;
use log::{error, info};
use rustc_driver::Compilation;
use rustc_interface::interface;
use rustc_middle::ty::TyCtxt;

pub struct BypasserCallbacks {
    pub analysis_options: AnalysisOption,
    pub source_name: String,
}

impl BypasserCallbacks {
    pub fn new(options: AnalysisOption) -> Self {
        Self {
            analysis_options: options,
            source_name: String::new(),
        }
    }
}

impl rustc_driver::Callbacks for BypasserCallbacks {
    /// Called before creating the compiler instance
    fn config(&mut self, config: &mut interface::Config) {
        self.source_name =config
        .input
        .source_name()
        .prefer_remapped_unconditionaly()
        .to_string();
        config.crate_cfg.push("mirai".to_string());
        info!("Source file: {}", self.source_name);
    }

    /// Called after analysis. Return value instructs the compiler whether to
    /// continue the compilation afterwards (defaults to `Compilation::Continue`)
    fn after_analysis<'compiler, 'tcx>(
        &mut self,
        compiler: &'compiler interface::Compiler,
        tcx: TyCtxt<'tcx>,
    ) -> Compilation {
        self.run_analysis(compiler, tcx);
        Compilation::Continue
    }
}

impl BypasserCallbacks {
    fn run_analysis<'tcx, 'compiler>(
        &mut self,
        compiler: &'compiler interface::Compiler,
        tcx: TyCtxt<'tcx>,
    ) {
        if self.source_name.contains("/libcore")
            || self.source_name.contains("/compiler_builtins")
            || self.source_name.contains("/liballoc")
            || self.source_name.contains("/macro")
            || self.source_name.contains("/libc")
        {
            info!(
                "Find filename that should skip the analysis: {}",
                self.source_name
            );
            return;
        }

        // Initialize global analysis context
        if let Some(mut global_context) =
            GlobalContext::new(&compiler.sess, tcx, self.analysis_options.clone())
        {
            // Initialize numerical analyzer
            let mut numerical_analysis = NumericalAnalysis::new(&mut global_context);
            // Run analyzer
            if let Ok(analysis_result) = numerical_analysis.run() {
                info!(
                    "Numerical Analysis Completed: {} ms, diagnostics={}, supported_diagnostics={}, unsupported_diagnostics={}, call_boundary_diagnostics={}, supported_special_calls={}, unsupported_special_calls={}, opaque_call_boundaries={}",
                    analysis_result.analysis_time.as_millis(),
                    analysis_result.total_diagnostics,
                    analysis_result.supported_diagnostics,
                    analysis_result.unsupported_diagnostics,
                    analysis_result.call_boundary_diagnostics,
                    analysis_result.supported_special_calls,
                    analysis_result.unsupported_special_calls,
                    analysis_result.opaque_call_boundaries,
                );
            } else {
                error!("Numerical Analysis Failed");
            }
        } else {
            error!("GlobalContext Initialization Failed");
        }
    }
}
