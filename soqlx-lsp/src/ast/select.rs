use super::typestate::*;
use std::convert::Infallible;

pub struct TypeofExpression {}

// ---- FieldIdentifier, parameterized by grouping ----
#[derive(Clone, Debug)]
pub enum FieldIdentifier<G>
where
    G: AliasPermitted + FunctionSet + SubqueryPermitted + TypeofPermitted,
{
    Field {
        label: String,
        alias: <G as AliasPermitted>::Alias, // alias allowed only when Grouped
    },
    Function {
        function: <G as FunctionSet>::Func, // function set changes by grouping
        argument: String,
        alias: <G as AliasPermitted>::Alias, // alias allowed only when Grouped
    },
    Subquery(<G as SubqueryPermitted>::Subquery),
    Typeof(<G as TypeofPermitted>::Typeof),
}

// ---- type-level switch: alias permitted ----
pub trait AliasPermitted {
    type Alias;
}
impl AliasPermitted for Grouped {
    type Alias = Option<String>;
}
impl AliasPermitted for Ungrouped {
    type Alias = Infallible;
}

// ---- type-level switch: functions ----
#[derive(Clone, Debug)]
pub enum AggFunc {
    Avg(FieldLabel),
    Sum,
    Min,
    Max,
}

#[derive(Clone, Debug)]
pub enum ScalarFunc {
    Upper,
    Lower,
    Length, /* ... SOQL scalar funcs ... */
}

pub trait FunctionSet {
    type Func;
}
impl FunctionSet for Grouped {
    type Func = AggFunc;
}
impl FunctionSet for Ungrouped {
    type Func = ScalarFunc;
}

// Convenient aliases: SimpleSelect and AggregateSelect
pub type Select<G: AliasPermitted + FunctionSet + TypeofPermitted> = Vec<FieldIdentifier<G>>;

// --- type-level switch: typeof ---
pub trait TypeofPermitted {
    /// Payload used by the `TYPEOF` select item.
    /// - Ungrouped => TypeofStatement
    /// - Grouped => Infallible (uninhabited; impossible to construct)
    type Typeof;
}
impl TypeofPermitted for Grouped {
    type Typeof = Infallible;
}
impl TypeofPermitted for Ungrouped {
    type Typeof = super::select::TypeofExpression;
}

// --- type-level switch: subqueries ---
pub trait SubqueryPermitted {
    /// Payload used by the `Subquery` select item.
    /// - Ungrouped => Box<Select<Ungrouped>>
    /// - Grouped   => Infallible (uninhabited; impossible to construct)
    type Subquery;
}
impl SubqueryPermitted for Ungrouped {
    type Subquery = Box<super::select::Select<Ungrouped>>;
}
impl SubqueryPermitted for Grouped {
    type Subquery = Infallible;
}

type FieldLabel = String;
