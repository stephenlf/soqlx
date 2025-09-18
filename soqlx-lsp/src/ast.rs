use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Identifier(pub String);

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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FieldPath(pub Vec<Identifier>);

impl FieldPath {
    pub fn new(parts: Vec<Identifier>) -> Self {
        Self(parts)
    }
}

/// Represents a complete SOQL [SELECT](https://developer.salesforce.com/docs/atlas.en-us.soql_sosl.meta/soql_sosl/sforce_api_calls_soql_select.htm) string
#[derive(Debug, Clone, PartialEq)]
pub struct Query {
    pub select: SelectClause,
    pub from: FromClause,
    pub where_clause: Option<Expression>,
    pub with: Option<WithClause>,
    pub group_by: Option<GroupByClause>,
    pub having: Option<Expression>,
    pub order_by: Option<OrderByClause>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
    pub for_clause: Option<ForClause>,
}

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
pub enum SelectItem {
    Field(FieldSelection),
    Function(FunctionSelection),
    Typeof(TypeofSelect),
    Subquery(Subquery),
}

#[derive(Debug, Clone, PartialEq)]
pub struct FieldSelection {
    pub field: FieldPath,
    pub alias: Option<Identifier>,
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
pub struct TypeofBranch {
    pub type_names: Vec<Identifier>,
    pub fields: Vec<FieldPath>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionCall {
    pub name: Identifier,
    pub distinct: bool,
    pub arguments: Vec<ValueExpression>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Subquery {
    pub query: Box<Query>,
    pub alias: Option<Identifier>,
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
