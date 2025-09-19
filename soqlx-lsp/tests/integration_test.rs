use soqlx_lsp::{ast::*, parser::parse_query};

fn ident(value: &str) -> Identifier {
    Identifier::new(value)
}

fn field_path(parts: &[&str]) -> FieldPath {
    FieldPath::new(
        parts
            .iter()
            .copied()
            .map(Identifier::new)
            .collect::<Vec<_>>(),
    )
}

fn field_item(path: &[&str], alias: Option<&str>) -> SelectItem {
    SelectItem::Field(FieldSelection {
        field: field_path(path),
        alias: alias.map(ident),
    })
}

fn function_call(name: &str, args: Vec<ValueExpression>, distinct: bool) -> FunctionCall {
    FunctionCall {
        name: ident(name),
        distinct,
        arguments: args,
    }
}

fn function_item(name: &str, args: Vec<ValueExpression>, alias: Option<&str>) -> SelectItem {
    SelectItem::Function(FunctionSelection {
        function: function_call(name, args, false),
        alias: alias.map(ident),
    })
}

fn function_item_distinct(
    name: &str,
    args: Vec<ValueExpression>,
    alias: Option<&str>,
) -> SelectItem {
    SelectItem::Function(FunctionSelection {
        function: function_call(name, args, true),
        alias: alias.map(ident),
    })
}

fn value_field(parts: &[&str]) -> ValueExpression {
    ValueExpression::Field(field_path(parts))
}

fn value_string(value: &str) -> ValueExpression {
    ValueExpression::Literal(Literal::String(value.into()))
}

fn value_integer(value: i64) -> ValueExpression {
    ValueExpression::Literal(Literal::Number(NumberLiteral::Integer(value)))
}

fn value_boolean(value: bool) -> ValueExpression {
    ValueExpression::Literal(Literal::Boolean(value))
}

fn value_null() -> ValueExpression {
    ValueExpression::Literal(Literal::Null)
}

fn value_date(date: &str) -> ValueExpression {
    ValueExpression::Literal(Literal::Date(DateLiteral::Date(date.into())))
}

fn value_relative_date(name: &str, offset: Option<i32>) -> ValueExpression {
    ValueExpression::Literal(Literal::RelativeDate(RelativeDateLiteral {
        name: ident(name),
        offset,
    }))
}

fn value_bind(parts: &[&str]) -> ValueExpression {
    ValueExpression::BindVariable(BindVariable {
        path: parts
            .iter()
            .copied()
            .map(Identifier::new)
            .collect::<Vec<_>>(),
    })
}

fn from_source(name_parts: &[&str], alias: Option<&str>, using: Option<&str>) -> FromSource {
    FromSource {
        name: field_path(name_parts),
        alias: alias.map(ident),
        using: using.map(|scope| UsingScope {
            scope: ident(scope),
        }),
    }
}

#[test]
fn test_simple() {
    let query = parse_query("SELECT Id FROM Account").expect("parse");
    assert_eq!(
        query,
        Query {
            select: SelectClause {
                modifier: None,
                items: vec![field_item(&["Id"], None)],
            },
            from: FromClause {
                sources: vec![from_source(&["Account"], None, None)],
            },
            where_clause: None,
            with: None,
            group_by: None,
            having: None,
            order_by: None,
            limit: None,
            offset: None,
            for_clause: None,
        }
    );
}

#[test]
fn test_multi_fields() {
    let query = parse_query("SELECT Id, Name FROM Account").expect("parse");
    assert_eq!(
        query.select.items,
        vec![field_item(&["Id"], None), field_item(&["Name"], None)]
    );
    assert_eq!(
        query.from,
        FromClause {
            sources: vec![from_source(&["Account"], None, None)]
        }
    );
}

#[test]
fn test_custom_fields() {
    let query = parse_query("SELECT Id, Custom_Field__c, Number_1__c FROM Custom_Object__c")
        .expect("parse");
    assert_eq!(
        query.select.items,
        vec![
            field_item(&["Id"], None),
            field_item(&["Custom_Field__c"], None),
            field_item(&["Number_1__c"], None)
        ]
    );
    assert_eq!(
        query.from.sources,
        vec![from_source(&["Custom_Object__c"], None, None)]
    );
}

#[test]
fn test_lookups() {
    let query = parse_query("SELECT Id, Parent.Name, Parent.Parent.Name, Parent.Parent.Parent.Name, Parent.Parent.Parent.Parent.Name, Parent.Parent.Parent.Parent.Parent.Name FROM Account").expect("parse");
    assert_eq!(
        query.select.items,
        vec![
            field_item(&["Id"], None),
            field_item(&["Parent", "Name"], None),
            field_item(&["Parent", "Parent", "Name"], None),
            field_item(&["Parent", "Parent", "Parent", "Name"], None),
            field_item(&["Parent", "Parent", "Parent", "Parent", "Name"], None),
            field_item(
                &["Parent", "Parent", "Parent", "Parent", "Parent", "Name"],
                None
            )
        ]
    );
}

#[test]
fn test_subqueries() {
    let input = "SELECT ( SELECT ( SELECT Id, Lookup__r.Field__c, ( SELECT Id FROM ChildAccounts ) FROM ChildAccounts ), Name FROM ChildAccounts ), Id, Parent.Name FROM Account";
    let query = parse_query(input).expect("parse");
    assert_eq!(query.select.items.len(), 3);
    match &query.select.items[0] {
        SelectItem::Subquery(Subquery {
            query: first_level,
            alias: None,
        }) => {
            let first_level = first_level.as_ref();
            assert_eq!(
                first_level.from.sources,
                vec![from_source(&["ChildAccounts"], None, None)]
            );
            assert_eq!(first_level.select.items.len(), 2);
            match &first_level.select.items[0] {
                SelectItem::Subquery(Subquery {
                    query: second_level,
                    alias: None,
                }) => {
                    let second_level = second_level.as_ref();
                    assert_eq!(
                        second_level.from.sources,
                        vec![from_source(&["ChildAccounts"], None, None)]
                    );
                    assert_eq!(second_level.select.items.len(), 3);
                    match &second_level.select.items[0] {
                        SelectItem::Field(field) => {
                            assert_eq!(field.field, field_path(&["Id"]));
                        }
                        other => panic!("unexpected select item {other:?}"),
                    }
                    match &second_level.select.items[1] {
                        SelectItem::Field(field) => {
                            assert_eq!(field.field, field_path(&["Lookup__r", "Field__c"]));
                        }
                        other => panic!("unexpected select item {other:?}"),
                    }
                    match &second_level.select.items[2] {
                        SelectItem::Subquery(Subquery {
                            query: third_level,
                            alias: None,
                        }) => {
                            let third_level = third_level.as_ref();
                            assert_eq!(third_level.select.items, vec![field_item(&["Id"], None)]);
                            assert_eq!(
                                third_level.from.sources,
                                vec![from_source(&["ChildAccounts"], None, None)]
                            );
                        }
                        other => panic!("unexpected select item {other:?}"),
                    }
                }
                other => panic!("unexpected select item {other:?}"),
            }
            match &first_level.select.items[1] {
                SelectItem::Field(field) => {
                    assert_eq!(field.field, field_path(&["Name"]));
                }
                other => panic!("unexpected select item {other:?}"),
            }
        }
        other => panic!("expected subquery, got {other:?}"),
    }
    assert_eq!(query.select.items[1], field_item(&["Id"], None));
    assert_eq!(query.select.items[2], field_item(&["Parent", "Name"], None));
    assert_eq!(
        query.from.sources,
        vec![from_source(&["Account"], None, None)]
    );
}

#[test]
fn test_field_aliases() {
    let query = parse_query("SELECT a.Name, b.Name FROM Account a, Account.Parent.Parent.Parent b")
        .expect("parse");
    assert_eq!(
        query.select.items,
        vec![
            field_item(&["a", "Name"], None),
            field_item(&["b", "Name"], None)
        ]
    );
    assert_eq!(
        query.from.sources,
        vec![
            from_source(&["Account"], Some("a"), None),
            from_source(&["Account", "Parent", "Parent", "Parent"], Some("b"), None),
        ]
    );
}

#[test]
fn test_typeof_clause() {
    let query = parse_query("SELECT TYPEOF What WHEN Account THEN Phone, NumberOfEmployees WHEN Opportunity THEN Amount, CloseDate ELSE Subject END FROM Event").expect("parse");
    match &query.select.items[0] {
        SelectItem::Typeof(typeof_select) => {
            assert_eq!(typeof_select.field, field_path(&["What"]));
            assert_eq!(
                typeof_select.branches,
                vec![
                    TypeofBranch {
                        type_names: vec![ident("Account")],
                        fields: vec![field_path(&["Phone"]), field_path(&["NumberOfEmployees"])],
                    },
                    TypeofBranch {
                        type_names: vec![ident("Opportunity")],
                        fields: vec![field_path(&["Amount"]), field_path(&["CloseDate"])],
                    }
                ]
            );
            assert_eq!(typeof_select.else_fields, vec![field_path(&["Subject"])]);
        }
        other => panic!("expected TYPEOF select, found {other:?}"),
    }
    assert_eq!(
        query.from,
        FromClause {
            sources: vec![from_source(&["Event"], None, None)]
        }
    );
}

#[test]
fn test_errant_commas() {
    let input = "SELECT Name, FROM Account";
    assert!(parse_query(input).is_err(), "{input} parses");
}

#[test]
fn test_aggregate_rollup_and_having() {
    let input = "SELECT COUNT(Id) total, SUM(Amount) sum_amount, COUNT(DISTINCT OwnerId) unique_owners FROM Opportunity WHERE StageName IN ('Closed Won','Closed Lost') GROUP BY ROLLUP(StageName) HAVING SUM(Amount) > 1000 ORDER BY SUM(Amount) DESC NULLS LAST LIMIT 50 OFFSET 5";
    let query = parse_query(input).expect("parse");
    assert_eq!(
        query.select.items,
        vec![
            function_item("COUNT", vec![value_field(&["Id"])], Some("total")),
            function_item("SUM", vec![value_field(&["Amount"])], Some("sum_amount")),
            function_item_distinct(
                "COUNT",
                vec![value_field(&["OwnerId"])],
                Some("unique_owners")
            ),
        ]
    );
    let where_clause = query.where_clause.as_ref().expect("where clause");
    match where_clause {
        Expression::Comparison(ComparisonExpression::In {
            left,
            operator,
            list,
        }) => {
            assert_eq!(left, &value_field(&["StageName"]));
            assert_eq!(*operator, InOperator::In);
            assert_eq!(
                list,
                &vec![value_string("Closed Won"), value_string("Closed Lost")]
            );
        }
        other => panic!("unexpected where clause: {other:?}"),
    }
    assert_eq!(
        query.group_by,
        Some(GroupByClause::Rollup(vec![GroupByExpression::Field(
            field_path(&["StageName"])
        )]))
    );
    let having = query.having.as_ref().expect("having clause");
    match having {
        Expression::Comparison(ComparisonExpression::Binary {
            left,
            operator,
            right,
        }) => {
            assert_eq!(*operator, ComparisonOperator::Gt);
            assert_eq!(right, &value_integer(1000));
            match left {
                ValueExpression::Function(function) => {
                    assert_eq!(function.name, ident("SUM"));
                    assert_eq!(function.arguments, vec![value_field(&["Amount"])]);
                }
                other => panic!("unexpected having lhs: {other:?}"),
            }
        }
        other => panic!("unexpected having clause: {other:?}"),
    }
    let order_by = query.order_by.as_ref().expect("order by");
    assert_eq!(order_by.fields.len(), 1);
    let order_field = &order_by.fields[0];
    assert_eq!(
        order_field.expression,
        ValueExpression::Function(function_call("SUM", vec![value_field(&["Amount"])], false))
    );
    assert_eq!(order_field.direction, Some(OrderDirection::Desc));
    assert_eq!(order_field.nulls, Some(NullsOrder::Last));
    assert_eq!(query.limit, Some(50));
    assert_eq!(query.offset, Some(5));
}

#[test]
fn test_complex_where_clauses() {
    let input = "SELECT Id FROM Opportunity WHERE NOT (IsClosed = TRUE OR Amount BETWEEN 10000 AND 50000) AND CloseDate >= 2021-01-01 AND Name LIKE 'Acme!_%' ESCAPE '!'";
    let query = parse_query(input).expect("parse");
    let where_clause = query.where_clause.as_ref().expect("where clause");
    let top_and = match where_clause {
        Expression::Logical(expr) => expr,
        other => panic!("unexpected where clause: {other:?}"),
    };
    match &top_and.right {
        Expression::Comparison(ComparisonExpression::Like {
            expression,
            pattern,
            escape,
        }) => {
            assert_eq!(expression, &value_field(&["Name"]));
            assert_eq!(pattern, &value_string("Acme!_%"));
            assert_eq!(*escape, Some('!'));
        }
        other => panic!("unexpected right branch: {other:?}"),
    }
    let left = &top_and.left;
    let second_and = match left {
        Expression::Logical(expr) => expr,
        other => panic!("unexpected left branch: {other:?}"),
    };
    match &second_and.right {
        Expression::Comparison(ComparisonExpression::Binary {
            left,
            operator,
            right,
        }) => {
            assert_eq!(left, &value_field(&["CloseDate"]));
            assert_eq!(*operator, ComparisonOperator::Gte);
            assert_eq!(right, &value_date("2021-01-01"));
        }
        other => panic!("unexpected middle branch: {other:?}"),
    }
    match &second_and.left {
        Expression::Not(inner) => match inner.as_ref() {
            Expression::Parenthesized(inner_expr) => match inner_expr.as_ref() {
                Expression::Logical(or_expr) => {
                    match &or_expr.left {
                        Expression::Comparison(ComparisonExpression::Binary {
                            left,
                            operator,
                            right,
                        }) => {
                            assert_eq!(left, &value_field(&["IsClosed"]));
                            assert_eq!(*operator, ComparisonOperator::Eq);
                            assert_eq!(right, &value_boolean(true));
                        }
                        other => panic!("unexpected left comparison: {other:?}"),
                    }
                    match &or_expr.right {
                        Expression::Comparison(ComparisonExpression::Between {
                            expression,
                            lower,
                            upper,
                        }) => {
                            assert_eq!(expression, &value_field(&["Amount"]));
                            assert_eq!(lower, &value_integer(10000));
                            assert_eq!(upper, &value_integer(50000));
                        }
                        other => panic!("unexpected right comparison: {other:?}"),
                    }
                }
                other => panic!("unexpected parenthesized expr: {other:?}"),
            },
            other => panic!("unexpected not expr: {other:?}"),
        },
        other => panic!("unexpected left branch: {other:?}"),
    }
}

#[test]
fn test_includes_and_excludes() {
    let input = "SELECT Id FROM Case WHERE RecordTypeId INCLUDES ('Support','Customer') AND Region__c EXCLUDES ('Legacy')";
    let query = parse_query(input).expect("parse");
    let where_clause = query.where_clause.as_ref().expect("where");
    let logical = match where_clause {
        Expression::Logical(expr) => expr,
        other => panic!("expected logical expression, found {other:?}"),
    };
    match &logical.left {
        Expression::Comparison(ComparisonExpression::Includes {
            left,
            operator,
            list,
        }) => {
            assert_eq!(left, &value_field(&["RecordTypeId"]));
            assert_eq!(*operator, IncludesOperator::Includes);
            assert_eq!(
                list,
                &vec![value_string("Support"), value_string("Customer")]
            );
        }
        other => panic!("unexpected includes clause: {other:?}"),
    }
    match &logical.right {
        Expression::Comparison(ComparisonExpression::Includes {
            left,
            operator,
            list,
        }) => {
            assert_eq!(left, &value_field(&["Region__c"]));
            assert_eq!(*operator, IncludesOperator::Excludes);
            assert_eq!(list, &vec![value_string("Legacy")]);
        }
        other => panic!("unexpected excludes clause: {other:?}"),
    }
}

#[test]
fn test_relative_date_literals_and_bind_variables() {
    let input = "SELECT Id FROM Event WHERE ActivityDate >= NEXT_N_DAYS:7 AND CreatedDate = YESTERDAY AND OwnerId = :context.userId";
    let query = parse_query(input).expect("parse");
    let where_clause = query.where_clause.as_ref().expect("where");
    let first_and = match where_clause {
        Expression::Logical(expr) => expr,
        other => panic!("unexpected where clause: {other:?}"),
    };
    match &first_and.right {
        Expression::Comparison(ComparisonExpression::Binary {
            left,
            operator,
            right,
        }) => {
            assert_eq!(left, &value_field(&["OwnerId"]));
            assert_eq!(*operator, ComparisonOperator::Eq);
            assert_eq!(right, &value_bind(&["context", "userId"]));
        }
        other => panic!("unexpected right comparison: {other:?}"),
    }
    let second_and = match &first_and.left {
        Expression::Logical(expr) => expr,
        other => panic!("unexpected left branch: {other:?}"),
    };
    match &second_and.right {
        Expression::Comparison(ComparisonExpression::Binary {
            left,
            operator,
            right,
        }) => {
            assert_eq!(left, &value_field(&["CreatedDate"]));
            assert_eq!(*operator, ComparisonOperator::Eq);
            assert_eq!(
                right,
                &ValueExpression::Literal(Literal::RelativeDate(RelativeDateLiteral {
                    name: ident("YESTERDAY"),
                    offset: None
                }))
            );
        }
        other => panic!("unexpected middle comparison: {other:?}"),
    }
    match &second_and.left {
        Expression::Comparison(ComparisonExpression::Binary {
            left,
            operator,
            right,
        }) => {
            assert_eq!(left, &value_field(&["ActivityDate"]));
            assert_eq!(*operator, ComparisonOperator::Gte);
            assert_eq!(right, &value_relative_date("NEXT_N_DAYS", Some(7)));
        }
        other => panic!("unexpected left comparison: {other:?}"),
    }
}

#[test]
fn test_data_category_filter_multiple_entries() {
    let input = "SELECT Title FROM Knowledge__kav WITH DATA CATEGORY Geography__c ABOVE usa__c, Products__c AT Laptop__c";
    let query = parse_query(input).expect("parse");
    match query.with.expect("with clause") {
        WithClause::DataCategory(filter) => {
            assert_eq!(
                filter,
                DataCategoryFilter {
                    conditions: vec![
                        DataCategoryCondition {
                            category: ident("Geography__c"),
                            op: DataCategoryOperator::Above,
                            value: ident("usa__c"),
                        },
                        DataCategoryCondition {
                            category: ident("Products__c"),
                            op: DataCategoryOperator::At,
                            value: ident("Laptop__c"),
                        }
                    ]
                }
            );
        }
        other => panic!("unexpected with clause: {other:?}"),
    }
}

#[test]
fn test_with_security_and_for_update() {
    let input = "SELECT Id FROM Case WITH SECURITY_ENFORCED FOR UPDATE TRACKING";
    let query = parse_query(input).expect("parse");
    assert!(matches!(query.with, Some(WithClause::SecurityEnforced)));
    assert_eq!(
        query.for_clause,
        Some(ForClause::Update(Some(UpdateType::Tracking)))
    );
}

#[test]
fn test_using_scope_clause() {
    let input = "SELECT Id FROM Contact USING SCOPE TEAM WHERE LastName != NULL ORDER BY LastName DESC NULLS FIRST";
    let query = parse_query(input).expect("parse");
    assert_eq!(
        query.from.sources,
        vec![from_source(&["Contact"], None, Some("TEAM"))]
    );
    let where_clause = query.where_clause.as_ref().expect("where");
    match where_clause {
        Expression::Comparison(ComparisonExpression::Binary {
            left,
            operator,
            right,
        }) => {
            assert_eq!(left, &value_field(&["LastName"]));
            assert_eq!(*operator, ComparisonOperator::Neq);
            assert_eq!(right, &value_null());
        }
        other => panic!("unexpected where clause: {other:?}"),
    }
    let order_by = query.order_by.as_ref().expect("order by");
    assert_eq!(
        order_by.fields,
        vec![OrderByField {
            expression: value_field(&["LastName"]),
            direction: Some(OrderDirection::Desc),
            nulls: Some(NullsOrder::First),
        }]
    );
}

#[test]
fn test_polymorphic_fields() {
    let input = "SELECT Id, What.Type, What.Name FROM Event";
    let query = parse_query(input).expect("parse");
    assert_eq!(
        query.select.items,
        vec![
            field_item(&["Id"], None),
            field_item(&["What", "Type"], None),
            field_item(&["What", "Name"], None),
        ]
    );
}

#[test]
fn test_in_subquery() {
    let input =
        "SELECT Id FROM Account WHERE Id IN (SELECT AccountId FROM Opportunity WHERE IsClosed = TRUE)";
    let query = parse_query(input).expect("parse");
    let where_clause = query.where_clause.as_ref().expect("where");
    match where_clause {
        Expression::Comparison(ComparisonExpression::InSubquery {
            left,
            operator,
            subquery,
        }) => {
            assert_eq!(left, &value_field(&["Id"]));
            assert_eq!(*operator, InOperator::In);
            let inner = subquery.query.as_ref();
            assert_eq!(inner.select.items, vec![field_item(&["AccountId"], None)]);
            assert_eq!(
                inner.from.sources,
                vec![from_source(&["Opportunity"], None, None)]
            );
            let inner_where = inner.where_clause.as_ref().expect("inner where");
            match inner_where {
                Expression::Comparison(ComparisonExpression::Binary {
                    left,
                    operator,
                    right,
                }) => {
                    assert_eq!(left, &value_field(&["IsClosed"]));
                    assert_eq!(*operator, ComparisonOperator::Eq);
                    assert_eq!(right, &value_boolean(true));
                }
                other => panic!("unexpected inner where: {other:?}"),
            }
        }
        other => panic!("unexpected where clause: {other:?}"),
    }
}

#[test]
fn test_select_modifier_and_order_by() {
    let input = "SELECT DISTINCT Name FROM Contact ORDER BY Name ASC NULLS LAST";
    let query = parse_query(input).expect("parse");
    assert_eq!(query.select.modifier, Some(SelectModifier::Distinct));
    assert_eq!(query.select.items, vec![field_item(&["Name"], None)]);
    let order_by = query.order_by.as_ref().expect("order by");
    assert_eq!(
        order_by.fields,
        vec![OrderByField {
            expression: value_field(&["Name"]),
            direction: Some(OrderDirection::Asc),
            nulls: Some(NullsOrder::Last),
        }]
    );
}

#[test]
fn test_group_by_grouping_sets() {
    let input =
        "SELECT COUNT(Id) total FROM Opportunity GROUP BY GROUPING SETS ((StageName, Type), (Type))";
    let query = parse_query(input).expect("parse");
    assert_eq!(
        query.select.items,
        vec![function_item(
            "COUNT",
            vec![value_field(&["Id"])],
            Some("total")
        )]
    );
    assert_eq!(
        query.group_by,
        Some(GroupByClause::GroupingSets(vec![
            vec![
                GroupByExpression::Field(field_path(&["StageName"])),
                GroupByExpression::Field(field_path(&["Type"]))
            ],
            vec![GroupByExpression::Field(field_path(&["Type"]))]
        ]))
    );
}
