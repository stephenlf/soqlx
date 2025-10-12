// Consider typestate. This only applies to knowledgebase articles.

#[derive(Clone, Debug, Copy)]
pub enum Update {
    Tracking,
    ViewStat,
}