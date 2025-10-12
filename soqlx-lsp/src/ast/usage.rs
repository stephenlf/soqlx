// This also might need typestate. I'm not sure that you can query groups for view

#[derive(Debug, Clone, Copy)]
pub enum Usage {
    ForView,
    ForReference
}