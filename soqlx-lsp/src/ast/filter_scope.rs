#[derive(Clone, Debug)]
pub enum FilterScope {
    Mine,
    Team,
    Custom(String),
}
