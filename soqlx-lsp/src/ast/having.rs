use super::typestate::{Grouped, Ungrouped};
use nonempty::NonEmpty;

#[derive(Clone, Debug)]
pub struct Having<G: HavingPermitted>(<G>::HavingExpr);

// --- type-level switch: group by ---
pub trait HavingPermitted {
    type HavingExpr: std::fmt::Debug + Clone; // Grouped => NonEmptyVec<String>, Ungrouped => ()
}
impl HavingPermitted for Grouped {
    type HavingExpr = Option<HavingExpression>;
}
impl HavingPermitted for Ungrouped {
    type HavingExpr = ();
}

#[derive(Clone, Debug)]
struct HavingExpression(/*...*/);