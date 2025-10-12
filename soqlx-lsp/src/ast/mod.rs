//! # The [SOQL SELECT](https://developer.salesforce.com/docs/atlas.en-us.soql_sosl.meta/soql_sosl/sforce_api_calls_soql_select.htm) Abstract Syntax Tree
//!
//! This module is architected to leverage the typestate pattern to enforce
//! certain query constructs. Specifically, all queries are bucketed into one
//! of two states: `Grouped` or `Ungrouped`. `Grouped` and `Ungrouped` queries
//! have different functions available to them. Additionally, only `Grouped`
//! queries support field aliasing.
//!

pub mod filter_scope;
pub mod from;
pub mod group_by;
pub mod select;
pub mod typestate;
pub mod condition;
pub mod filtering;
pub mod having;
pub mod order;
pub mod usage;
pub mod article;

use std::marker::PhantomData;

// ---- Select, parameterized by grouping ----
#[derive(Clone, Debug)]
pub struct Soql<G, K>
where
    G: typestate::Grouping
        + select::AliasPermitted
        + select::FunctionSet
        + select::TypeofPermitted
        + select::SubqueryPermitted
        + group_by::HasGroupBy
        + having::HavingPermitted,
    K: typestate::QueriesArticle,
{
    pub select: select::Select<G>,
    pub from: from::From,
    pub using_scope: Option<filter_scope::FilterScope>,
    pub where_expression: Option<condition::Where>,
    pub filtering: Option<filtering::FilteringExpression>,
    pub group_by: group_by::GroupBy<G>,
    pub having: having::Having<G>,
    pub order_by: Option<order::OrderBy>,
    pub limit: Option<u8>,
    pub offset: Option<u8>,
    pub usage: Option<usage::Usage>,
    pub update_article: Option<article::Update>,
    pub for_update: Option<()>,
    _grouping: PhantomData<G>,
    _queries_knowledge_article: PhantomData<K>,
}