use soqlx_lsp::{
    // ast,
    parser::parse_query
};

#[test]
fn test_simple() {
    let input = "SELECT Id FROM Account";
    assert!(parse_query(input).is_ok(), "{} parses", input);
}

#[test]
fn test_multi_fields() {
    let input = "
        SELECT Id, Name
        FROM Account
    ";
    assert!(parse_query(input).is_ok(), "{} parses", input);
}

#[test]
fn test_custom_fields() {
    let input = "
        SELECT Id, Custom_Field__c, Number_1__c
        FROM Custom_Object__c
    ";
    assert!(parse_query(input).is_ok(), "{} parses", input);
}

#[test]
fn test_lookups() {
    let input = "
        SELECT Id,
            Parent.Name,
            Parent.Parent.Name,
            Parent.Parent.Parent.Name,
            Parent.Parent.Parent.Parent.Name,
            Parent.Parent.Parent.Parent.Parent.Name
        FROM Account
    ";
    assert!(parse_query(input).is_ok(), "{} parses", input);
}

#[test]
fn test_subqueries() {
    let input = "
        SELECT
          ( SELECT 
            ( SELECT Id, Lookup__r.Field__c,
              ( SELECT Id
                FROM ChildAccounts )
              FROM ChildAccounts ),
                Name
            FROM ChildAccounts ),
            Id, Parent.Name
        FROM Account
    ";
    assert!(parse_query(input).is_ok(), "{} parses", input);
}

#[test]
fn test_field_aliases() {
    let input = "
        SELECT a.Name, b.Name
        FROM Account a, Account.Parent.Parent.Parent b
    ";
    assert!(parse_query(input).is_ok(), "{} parses", input);
}

#[test]
fn test_typeof_clause() {
    let input = "
        SELECT 
            TYPEOF What
                WHEN Account THEN Phone
                ELSE Name
            END
        FROM Event
    ";
    assert!(parse_query(input).is_ok(), "{} parses", input);
}

#[test]
fn test_errant_commas() {
    let input = "SELECT Name, FROM Account";
    assert!(parse_query(input).is_err(), "{} parses", input);
}