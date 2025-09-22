//! # The [SOQL SELECT](https://developer.salesforce.com/docs/atlas.en-us.soql_sosl.meta/soql_sosl/sforce_api_calls_soql_select.htm) Abstract Syntax Tree
//! 
//! This module is architected to leverage the typestate pattern to enforce
//! certain query constructs. Specifically, all queries are bucketed into one
//! of two states: `Grouped` or `Ungrouped`. `Grouped` and `Ungrouped` queries
//! have different functions available to them. Additionally, only `Grouped`
//! queries support field aliasing.
//! 

use std::{convert::Infallible, fmt};
use nonempty::NonEmpty;

// ---- Select, parameterized by grouping ----
#[derive(Clone, Debug)]
pub struct Select<G>
where
    G: Grouping + HasAlias + HasFunctionSet + HasGroupBy,
{
    pub select: Vec<FieldIdentifier<G>>,
    pub from: String, // ObjectIdentifier
    pub using_scope: Optional<UserScope>,
    pub group_by: <G as HasGroupBy>::GroupBy,
}

// ---- typestate markers Grouped and Ungrouped ----
pub struct Grouped;
pub struct Ungrouped;

pub trait Grouping {}
impl Grouping for Grouped {}
impl Grouping for Ungrouped {}

// ---- type-level switch: alias permitted ----
pub trait HasAlias {
    type Alias;
}
impl HasAlias for Grouped    { type Alias = Option<String>; }
impl HasAlias for Ungrouped  { type Alias = Infallible; }

// ---- type-level switch: functions ----
#[derive(Clone, Debug)]
pub enum AggFunc {
    Avg(FieldLabel),
    Sum,
    Min,
    Max,
}

#[derive(Clone, Debug)]
pub enum ScalarFunc { Upper, Lower, Length,  /* ... SOQL scalar funcs ... */ }

pub trait HasFunctionSet {
    type Func;
}
impl HasFunctionSet for Grouped   { type Func = AggFunc; }
impl HasFunctionSet for Ungrouped { type Func = ScalarFunc; }

// --- type-level switch: group by ---
pub trait HasGroupBy {
    type GroupBy; // Grouped => NonEmptyVec<String>, Ungrouped => ()
}
impl HasGroupBy for Grouped   { type GroupBy = NonEmpty<String>; }
impl HasGroupBy for Ungrouped { type GroupBy = (); }

// --- type-level switch: subqueries ---
pub trait HasSubqueries {
    /// Payload used by the `Subquery` select item.
    /// - Ungrouped => Box<Select<Ungrouped>>
    /// - Grouped   => Infallible (uninhabited; impossible to construct)
    type Subquery;
}
impl HasSubqueries for Ungrouped { type Subquery = Box<Select<Ungrouped>>; }
impl HasSubqueries for Grouped   { type Subquery = Infallible; }

// --- type-level switch: typeof ---
pub trait HasTypeof {
    /// Payload used by the `TYPEOF` select item.
    /// - Ungrouped => TypeofStatement
    /// - Grouped => Infallible (uninhabited; impossible to construct)
    type Typeof;
}
impl HasTypeof for Ungrouped { type Typeof = TypeofExpression; }
impl HasTypeof for Grouped { type Typeof = Infallible; }

pub struct TypeofExpression {}

// ---- FieldIdentifier, parameterized by grouping ----
#[derive(Clone, Debug)]
pub enum FieldIdentifier<G>
where
    G: Grouping + HasAlias + HasFunctionSet + HasTypeof,
{
    Field {
        label: String,
        alias: <G as HasAlias>::Alias,   // alias allowed only when Grouped
    },
    Function {
        function: <G as HasFunctionSet>::Func, // function set changes by grouping
        argument: String,
        alias: <G as HasAlias>::Alias,   // alias allowed only when Grouped
    },
    Subquery(<G as HasSubqueries>::Subquery),
    Typeof(<G as HasTypeof>::Typeof),
}

// Convenient aliases: SimpleSelect and AggregateSelect
pub type SimpleSelect   = Select<Ungrouped>;
pub type AggregateSelect = Select<Grouped>;

// --- from clause
pub struct FromClause {
    from: (ObjectName, Option<Alias>),
    aliases: Vec<(LookupName, Option<Alias>)>
}

pub struct ObjectName(pub String);
pub struct LookupName(pub String);
pub struct Alias(pub String);

// --- user scope
pub enum UserScope {
    Mine,
    Team,
    Custom(pub String),
}

/**

/// # 
/// 
/// SOQL query syntax consists of a required SELECT statement followed by one or more optional clauses, such as TYPEOF, WHERE, WITH, GROUP BY, and ORDER BY.
/// 
/// The SOQL SELECT statement uses the following syntax:
/// 
/// ```text
/// SELECT fieldList [subquery][...]
/// [TYPEOF typeOfField whenExpression[...] elseExpression END][...]
/// FROM objectType[,...] 
///     [USING SCOPE filterScope]
/// [WHERE conditionExpression]
/// [WITH [DATA CATEGORY] filteringExpression]
/// [GROUP BY {fieldGroupByList|ROLLUP (fieldSubtotalGroupByList)|CUBE (fieldSubtotalGroupByList)} 
///     [HAVING havingConditionExpression] ] 
/// [ORDER BY fieldOrderByList {ASC|DESC} [NULLS {FIRST|LAST}] ]
/// [LIMIT numberOfRowsToReturn]
/// [OFFSET numberOfRowsToSkip]
/// [{FOR VIEW  | FOR REFERENCE} ]
/// [UPDATE {TRACKING|VIEWSTAT} ]
/// [FOR UPDATE]
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct Query {
    pub select: Vec<SelectItem>,
    pub from: FromClause,
    pub with: Option<WithClause>,
    pub group_by: Option<GroupByClause>,
    pub having: Option<Expression>,
    pub order_by: Option<OrderByClause>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
    pub for_clause: Option<ForClause>,
    pub update: Option<UpdateClause>,
}

pub enum FieldList {
    FieldList(pub Vec<SelectItem>),
    AggregateFieldList(pub Vec<String>),
}



/// Specifies a list of one or more fields, separated by commas, that you want to retrieve from the specified object. The bold elements in the following examples are fieldlist values:
///   - SELECT **Id, Name, BillingCity** FROM Account
///   - SELECT **count()** FROM Contact
///   - SELECT **Contact.Firstname, Contact.Account.Name** FROM Contact
///   - SELECT **FIELDS(STANDARD)** FROM Contact
/// 
/// Use valid field names and include read-level permissions for each specified field. The fieldList defines the ordering of fields in the query results.
/// If the query traverses a relationship, fieldList can include a subquery. For example:
/// 
/// ```soql
/// SELECT Account.Name, (SELECT Contact.LastName FROM Account.Contacts)
/// FROM Account
/// ```
/// 
/// The fieldlist can also be an aggregate function, such as `COUNT()` and `COUNT(fieldName)`, or be wrapped in the [toLabel()](https://developer.salesforce.com/docs/atlas.en-us.soql_sosl.meta/soql_sosl/sforce_api_calls_soql_select_tolabel.htm) function to translate returned results. See [`SELECT`](https://developer.salesforce.com/docs/atlas.en-us.soql_sosl.meta/soql_sosl/sforce_api_calls_soql_select_fields.htm#topic-title) for more information.
#[derive(Debug, Clone, PartialEq)]
pub enum SelectItem {
    Field(FieldPath),
    AggregateField(Aggregate),
    FunctionField(FunctionSelection),
    Typeof(TypeofSelect),
    Subquery(Subquery),
}

pub enum AggregateItem {

}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FieldPath(pub Vec<Identifier>);

pub struct Aggregate {
    field: AggregateFunction,
    alias: Optional<Identifier>
}

pub enum AggregateFunction {
    Avg(FieldPath),
    Count(Option<FieldPath>),
    CountDistinct(FieldPath),
    Min(FieldPath),
    Max(FieldPath),
    Sum(FieldPath),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Identifier(pub String);

/**
#[derive(Debug, Clone, PartialEq)]
pub struct SelectClause {
    pub modifier: Option<SelectModifier>,
    pub items: Vec<SelectItem>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SelectModifier {
    All,
    Distinct,
}


#[derive(Debug, Clone, PartialEq)]
pub struct FunctionSelection {
    pub function: FunctionCall,
    pub alias: Option<Identifier>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TypeofSelect {
    pub field: FieldPath,
    pub branches: Vec<TypeofBranch>,
    pub else_fields: Vec<FieldPath>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Subquery {
    pub query: Box<Query>,
    pub alias: Option<Identifier>,
}






// ------------------------------------------------

impl Identifier {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Identifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}



impl FieldPath {
    pub fn new(parts: Vec<Identifier>) -> Self {
        Self(parts)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionCall {
    pub name: Identifier,
    pub distinct: bool,
    pub arguments: Vec<ValueExpression>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TypeofBranch {
    pub type_names: Vec<Identifier>,
    pub fields: Vec<FieldPath>,
}





#[derive(Debug, Clone, PartialEq)]
pub struct FromClause {
    pub sources: Vec<FromSource>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FromSource {
    pub name: FieldPath,
    pub alias: Option<Identifier>,
    pub using: Option<UsingScope>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UsingScope {
    pub scope: Identifier,
}

#[derive(Debug, Clone, PartialEq)]
pub enum WithClause {
    DataCategory(DataCategoryFilter),
    SecurityEnforced,
    SystemMode,
    UserMode,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DataCategoryFilter {
    pub conditions: Vec<DataCategoryCondition>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DataCategoryCondition {
    pub category: Identifier,
    pub op: DataCategoryOperator,
    pub value: Identifier,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DataCategoryOperator {
    Above,
    Below,
    At,
}

#[derive(Debug, Clone, PartialEq)]
pub enum GroupByClause {
    Standard(Vec<GroupByExpression>),
    Rollup(Vec<GroupByExpression>),
    Cube(Vec<GroupByExpression>),
    GroupingSets(Vec<Vec<GroupByExpression>>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum GroupByExpression {
    Field(FieldPath),
    Function(FunctionCall),
}

#[derive(Debug, Clone, PartialEq)]
pub struct OrderByClause {
    pub fields: Vec<OrderByField>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct OrderByField {
    pub expression: ValueExpression,
    pub direction: Option<OrderDirection>,
    pub nulls: Option<NullsOrder>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OrderDirection {
    Asc,
    Desc,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NullsOrder {
    First,
    Last,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ForClause {
    View,
    Reference,
    Update(Option<UpdateType>),
    Share,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UpdateType {
    Tracking,
    Viewstat,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    Logical(Box<LogicalExpression>),
    Not(Box<Expression>),
    Comparison(ComparisonExpression),
    Parenthesized(Box<Expression>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct LogicalExpression {
    pub left: Expression,
    pub operator: LogicalOperator,
    pub right: Expression,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LogicalOperator {
    And,
    Or,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ComparisonExpression {
    Binary {
        left: ValueExpression,
        operator: ComparisonOperator,
        right: ValueExpression,
    },
    In {
        left: ValueExpression,
        operator: InOperator,
        list: Vec<ValueExpression>,
    },
    InSubquery {
        left: ValueExpression,
        operator: InOperator,
        subquery: Subquery,
    },
    Includes {
        left: ValueExpression,
        operator: IncludesOperator,
        list: Vec<ValueExpression>,
    },
    Between {
        expression: ValueExpression,
        lower: ValueExpression,
        upper: ValueExpression,
    },
    Like {
        expression: ValueExpression,
        pattern: ValueExpression,
        escape: Option<char>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ComparisonOperator {
    Eq,
    Neq,
    Lt,
    Lte,
    Gt,
    Gte,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InOperator {
    In,
    NotIn,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IncludesOperator {
    Includes,
    Excludes,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ValueExpression {
    Field(FieldPath),
    Literal(Literal),
    Function(FunctionCall),
    BindVariable(BindVariable),
    Subquery(Subquery),
    Typeof(Box<TypeofSelect>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct BindVariable {
    pub path: Vec<Identifier>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    String(String),
    Number(NumberLiteral),
    Boolean(bool),
    Null,
    Date(DateLiteral),
    RelativeDate(RelativeDateLiteral),
}

#[derive(Debug, Clone, PartialEq)]
pub struct RelativeDateLiteral {
    pub name: Identifier,
    pub offset: Option<i32>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DateLiteral {
    Date(String),
    DateTime(String),
    Time(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum NumberLiteral {
    Integer(i64),
    Decimal(String),
}
*/