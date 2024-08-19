use std::fmt::Display;

struct Statement<'a> {
    query: Query<'a>,
    modifiers: Vec<Modifier>
}

struct Query<'a> {
    object: Object<'a>,
    fields: Vec<Field<'a>>,
} 

enum Modifier {
    
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct Object<'a> (&'a str);
impl<'a> From<&'a str> for Object<'a> {
    fn from(s: &'a str) -> Self {
        Self(s)
    }
}
impl<'a> Display for Object<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", &self.0)
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct Field<'a> (&'a str);
impl<'a> From<&'a str> for Field<'a> {
    fn from(s: &'a str) -> Self {
        Self(s)
    }
}
impl<'a> Display for Field<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", &self.0)
    }
}

impl<'a> Query<'a> {
    pub fn to_apex(&'a self) -> String {
        format!(
            "{}[] {} = [SELECT {} FROM {}];", 
            &self.object, 
            &self.object.0.to_lowercase(),
            &self.fields.iter().map(|field| field.0).collect::<Vec<&str>>().join(", "),
            &self.object, 
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::Query;

    #[test]
    fn test() {
        assert!(true, "This works");
    }

    #[test]
    fn simple_query() {
        let query = Query {
            object: "Account".into(),
            fields:
            vec!["Name".into(), "CreatedDate".into()]
        };
        let parsed = query.to_apex();
        assert_eq!(&parsed, "Account[] account = [SELECT Name, CreatedDate FROM Account];")
    }
}