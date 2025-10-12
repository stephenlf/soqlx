// --- from clause
#[derive(Clone, Debug)]
pub struct From {
    from: (ObjectName, Option<Alias>),
    aliases: Vec<(LookupName, Option<Alias>)>,
}

#[derive(Clone, Debug)]
pub struct ObjectName(pub String);

#[derive(Clone, Debug)]
pub struct LookupName(pub String);

#[derive(Clone, Debug)]
pub struct Alias(pub String);
