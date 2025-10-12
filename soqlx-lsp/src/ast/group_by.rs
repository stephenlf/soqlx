use super::typestate::{Grouped, Ungrouped};
use nonempty::NonEmpty;

#[derive(Clone, Debug)]
pub struct GroupBy<G: HasGroupBy>(<G>::GroupByExpr);

// --- type-level switch: group by ---
pub trait HasGroupBy {
    type GroupByExpr: std::fmt::Debug + Clone; // Grouped => NonEmptyVec<String>, Ungrouped => ()
}
impl HasGroupBy for Grouped {
    type GroupByExpr = NonEmpty<String>;
}
impl HasGroupBy for Ungrouped {
    type GroupByExpr = ();
}
