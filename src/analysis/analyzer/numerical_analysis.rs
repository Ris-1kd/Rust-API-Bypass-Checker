use crate::analysis::abstract_domain::AbstractDomain;
use crate::analysis::analysis_result::{AnalysisInfo, Result};
use crate::analysis::analyzer::analysis_trait::StaticAnalysis;
use crate::analysis::diagnostics::Diagnostic;
use crate::analysis::global_context::GlobalContext;
use crate::analysis::mir_visitor::body_visitor::WtoFixPointIterator;
use crate::analysis::numerical::apron_domain::{
    ApronAbstractDomain, ApronDomainType, ApronInterval, GetManagerTrait,
};
use crate::analysis::option::AbstractDomainType;
use log::info;
use rustc_hir::def_id::DefId;
use std::cmp::Ordering;
use std::time::Instant;

/// Traverse over a crate, analyze all functions and emit diagnoses
pub struct NumericalAnalysis<'tcx, 'a, 'compiler> {
    /// The global context
    pub context: &'a mut GlobalContext<'tcx, 'compiler>,
}

impl<'tcx, 'a, 'compiler> NumericalAnalysis<'tcx, 'a, 'compiler> {
    fn collect_diagnostics(&mut self) -> Vec<Diagnostic<'compiler>> {
        let mut diagnostics: Vec<Diagnostic<'_>> = self
            .context
            .diagnostics_for
            .map
            .values_mut()
            .map(|v| v.iter_mut().filter_map(|x| x.take()))
            .flatten()
            .collect();

        diagnostics.sort_by(|a, b| Diagnostic::compare(a, b));

        let diagnostics: Vec<Diagnostic<'_>> =
            if let Some(suppressed_warnings) = &self.context.analysis_options.suppressed_warnings {
                let mut res: Vec<Diagnostic<'_>> = Vec::new();
                for diag in diagnostics.into_iter() {
                    if suppressed_warnings.contains(&diag.cause) {
                        diag.cancel();
                    } else {
                        res.push(diag);
                    }
                }
                res
            } else {
                diagnostics.into_iter().collect()
            };

        let mut deduped: Vec<Diagnostic<'_>> = Vec::new();
        for diag in diagnostics.into_iter() {
            let is_duplicate = deduped
                .last()
                .map(|prev| {
                    prev.cause == diag.cause
                        && Diagnostic::compare(prev, &diag) == Ordering::Equal
                        && format!("{:?}", prev.builder) == format!("{:?}", diag.builder)
                })
                .unwrap_or(false);
            if is_duplicate {
                diag.cancel();
            } else {
                deduped.push(diag);
            }
        }
        deduped
    }
}

impl<'tcx, 'a, 'compiler> StaticAnalysis<'tcx, 'a, 'compiler>
    for NumericalAnalysis<'tcx, 'a, 'compiler>
{
    fn new(context: &'a mut GlobalContext<'tcx, 'compiler>) -> Self {
        NumericalAnalysis { context }
    }

    fn emit_diagnostics(&mut self) {
        self.collect_diagnostics()
            .into_iter()
            .for_each(|diag| diag.emit());
    }

    fn run(&mut self) -> Result<AnalysisInfo> {
        let timer = Instant::now();

        info!("================== Numerical Analysis Starts ==================");
        info!("Abstract Domain Type: {:?}", self.context.analysis_options.domain_type);
        info!("Widening Delay: {}", self.context.analysis_options.widening_delay);
        info!("Start Analyzing Entry Point Function: {}", self.context.tcx.item_name(self.context.entry_point));

        // Start analysis with the entry point
        let def_id = self.context.entry_point;

        match self.context.analysis_options.domain_type {
            AbstractDomainType::Interval => {
                self.analyze_function(def_id, AbstractDomain::<ApronInterval>::default());
            }
            __ => {} // ignored all other numerical domains, only retain the interval.
        }

        info!("================== Numerical Analysis Ends ==================");

        let diagnostics = self.collect_diagnostics();
        let total_diagnostics = diagnostics.len();
        let unsupported_diagnostics = diagnostics
            .iter()
            .filter(|diag| diag.cause == crate::analysis::diagnostics::DiagnosticCause::Unsupported)
            .count();
        let supported_diagnostics = total_diagnostics.saturating_sub(unsupported_diagnostics);

        info!("================== Start To Output Diagnostics ==================");
        diagnostics.into_iter().for_each(|diag| diag.emit());

        Ok(AnalysisInfo {
            analysis_time: timer.elapsed(),
            total_diagnostics,
            supported_diagnostics,
            unsupported_diagnostics,
            supported_special_calls: self.context.supported_special_calls,
            unsupported_special_calls: self.context.unsupported_special_calls,
        })
    }

    fn analyze_function<DomainType>(
        &mut self,
        def_id: DefId,
        abstract_domain: AbstractDomain<DomainType>,
    ) where
        DomainType: ApronDomainType,
        ApronAbstractDomain<DomainType>: GetManagerTrait,
    {
        let func_name = self.context.tcx.item_name(def_id);
        info!(
            "================== Fixed-Point Algorithm Starts To Analyze: {} ==================",
            func_name
        );

        // Compute the fixed-point of the function specified by `def_id`
        let mut wto_visitor =
            WtoFixPointIterator::new(self.context, def_id, abstract_domain, 0, vec![]);
        wto_visitor.init_promote_constants();
        wto_visitor.run();

        info!("The final current state of user's crate: {:?}", wto_visitor.state);
        // Execute bug detector
        wto_visitor.run_checker();

        debug!(
            "{} diagnositcs for function {:?}",
            wto_visitor.buffered_diagnostics.len(),
            func_name
        );

        info!("================== Fixed-Point Algorithm Ends To Analyze:{} ==================",
            func_name
        );
    }
}
