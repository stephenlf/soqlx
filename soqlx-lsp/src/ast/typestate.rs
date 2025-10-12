// ---- typestate markers Grouped and Ungrouped ----

#[derive(Debug, Clone, Copy)]
pub struct Grouped;
#[derive(Debug, Clone, Copy)]
pub struct Ungrouped;

pub trait Grouping {}
impl Grouping for Grouped {}
impl Grouping for Ungrouped {}


#[derive(Debug, Clone, Copy)]
pub struct Article;
#[derive(Debug, Clone, Copy)]
pub struct NotArticle;

pub trait QueriesArticle {}
impl QueriesArticle for Article {}
impl QueriesArticle for NotArticle {}