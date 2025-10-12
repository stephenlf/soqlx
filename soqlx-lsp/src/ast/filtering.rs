#[derive(Clone, Debug)]
pub enum FilteringExpression {
    DataCategory(CategoryFilter),
    SecurityEnforced,
    UserMode,
    SystemMode,
}

#[derive(Clone, Debug)]
struct CategoryFilter();