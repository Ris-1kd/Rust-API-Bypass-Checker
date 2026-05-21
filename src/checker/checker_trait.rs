use crate::analysis::mir_visitor::body_visitor::WtoFixPointIterator;
use crate::analysis::numerical::interval_domain::{
    GetDomainType, IntervalAbstractDomain, NumericalDomainType,
};

pub trait CheckerTrait<'tcx, 'a, 'b, 'compiler, DomainType>
where
    DomainType: NumericalDomainType,
    IntervalAbstractDomain<DomainType>: GetDomainType,
{
    fn new(body_visitor: &'b mut WtoFixPointIterator<'tcx, 'a, 'compiler, DomainType>) -> Self;

    fn run(&mut self);
}
