use std::fmt;

use crate::ast::*;
use nom::branch::alt;
use nom::bytes::complete::{tag_no_case, take_while, take_while_m_n};
use nom::character::complete::{char, digit1, multispace0, multispace1, one_of};
use nom::combinator::{all_consuming, map, map_res, opt, recognize, value};
use nom::error::{convert_error, VerboseError, VerboseErrorKind};
use nom::multi::{many0, many1, separated_list0, separated_list1};
use nom::sequence::{delimited, preceded, terminated, tuple};
use nom::{Err as NomErr, IResult};

pub type Res<'a, T> = IResult<&'a str, T, VerboseError<&'a str>>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
    message: String,
}

impl ParseError {
    fn from_nom(input: &str, err: NomErr<VerboseError<&str>>) -> Self {
        match err {
            NomErr::Incomplete(_) => Self {
                message: "incomplete input".to_string(),
            },
            NomErr::Error(e) | NomErr::Failure(e) => Self {
                message: convert_error(input, e),
            },
        }
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.message.fmt(f)
    }
}

impl std::error::Error for ParseError {}

pub fn parse_query(input: &str) -> Result<Query, ParseError> {
    let trimmed = input.trim();
    match all_consuming(terminated_query)(trimmed) {
        Ok((_, query)) => Ok(query),
        Err(err) => Err(ParseError::from_nom(trimmed, err)),
    }
}

fn terminated_query(input: &str) -> Res<'_, Query> {
    let (input, query) = query(input)?;
    let (input, _) = multispace0(input)?;
    Ok((input, query))
}

fn query(input: &str) -> Res<'_, Query> {
    alt((select_first_query, from_first_query))(input)
}

fn select_first_query(input: &str) -> Res<'_, Query> {
    let (input, select) = select_clause(input)?;
    let (input, from) = from_clause(input)?;
    finish_query(input, select, from)
}

fn from_first_query(input: &str) -> Res<'_, Query> {
    let (input, from) = from_clause(input)?;
    let (input, select) = select_clause(input)?;
    finish_query(input, select, from)
}

fn finish_query(mut input: &str, select: SelectClause, from: FromClause) -> Res<'_, Query> {
    let (i, where_clause) = opt(where_clause)(input)?;
    input = i;
    let (i, with) = opt(with_clause)(input)?;
    input = i;
    let (i, group_by) = opt(group_by_clause)(input)?;
    input = i;
    let (i, having) = opt(having_clause)(input)?;
    input = i;
    let (i, order_by) = opt(order_by_clause)(input)?;
    input = i;
    let (i, limit) = opt(limit_clause)(input)?;
    input = i;
    let (i, offset) = opt(offset_clause)(input)?;
    input = i;
    let (input, for_clause) = opt(for_clause)(input)?;

    Ok((
        input,
        Query {
            select,
            from,
            where_clause,
            with,
            group_by,
            having,
            order_by,
            limit,
            offset,
            for_clause,
        },
    ))
}

fn select_clause(input: &str) -> Res<'_, SelectClause> {
    let (input, _) = ws_keyword("SELECT")(input)?;
    let (input, _) = multispace1(input)?;
    let (input, modifier) = opt(terminated(select_modifier, multispace1))(input)?;
    let (input, first) = select_item(input)?;
    let (input, mut rest) = many0(preceded(comma, select_item))(input)?;
    let mut items = Vec::with_capacity(1 + rest.len());
    items.push(first);
    items.append(&mut rest);
    Ok((input, SelectClause { modifier, items }))
}

fn select_modifier(input: &str) -> Res<'_, SelectModifier> {
    alt((
        value(SelectModifier::Distinct, keyword("DISTINCT")),
        value(SelectModifier::All, keyword("ALL")),
    ))(input)
}

fn select_item(input: &str) -> Res<'_, SelectItem> {
    preceded(
        multispace0,
        alt((
            select_subquery_item,
            select_typeof_item,
            select_function_item,
            select_field_item,
        )),
    )(input)
}

fn select_field_item(input: &str) -> Res<'_, SelectItem> {
    let (input, field) = field_path(input)?;
    let (input, alias) = optional_alias(input)?;
    Ok((input, SelectItem::Field(FieldSelection { field, alias })))
}

fn select_function_item(input: &str) -> Res<'_, SelectItem> {
    let (input, function) = function_call(input)?;
    let (input, alias) = optional_alias(input)?;
    Ok((
        input,
        SelectItem::Function(FunctionSelection { function, alias }),
    ))
}

fn select_typeof_item(input: &str) -> Res<'_, SelectItem> {
    let (input, typeof_expr) = typeof_expression(input)?;
    Ok((input, SelectItem::Typeof(typeof_expr)))
}

fn select_subquery_item(input: &str) -> Res<'_, SelectItem> {
    let (input, _) = multispace0(input)?;
    let (input, _) = char('(')(input)?;
    let (input, _) = multispace0(input)?;
    let (input, inner_query) = query(input)?;
    let (input, _) = multispace0(input)?;
    let (input, _) = char(')')(input)?;
    let (input, alias) = optional_alias(input)?;
    Ok((
        input,
        SelectItem::Subquery(Subquery {
            query: Box::new(inner_query),
            alias,
        }),
    ))
}

fn from_clause(input: &str) -> Res<'_, FromClause> {
    let (input, _) = ws_keyword("FROM")(input)?;
    let (input, _) = multispace1(input)?;
    let (input, first) = from_source(input)?;
    let (input, mut rest) = many0(preceded(comma, from_source))(input)?;
    let mut sources = Vec::with_capacity(1 + rest.len());
    sources.push(first);
    sources.append(&mut rest);
    Ok((input, FromClause { sources }))
}

fn from_source(input: &str) -> Res<'_, FromSource> {
    let (input, name) = field_path(input)?;
    let (input, alias) = optional_alias(input)?;
    let (input, using) = opt(using_scope)(input)?;
    Ok((input, FromSource { name, alias, using }))
}

fn using_scope(input: &str) -> Res<'_, UsingScope> {
    let (input, _) = multispace1(input)?;
    let (input, _) = keyword("USING")(input)?;
    let (input, _) = multispace1(input)?;
    let (input, _) = keyword("SCOPE")(input)?;
    let (input, _) = multispace1(input)?;
    let (input, scope) = identifier(input)?;
    Ok((input, UsingScope { scope }))
}

fn where_clause(input: &str) -> Res<'_, Expression> {
    let (input, _) = ws_keyword("WHERE")(input)?;
    expression(input)
}

fn with_clause(input: &str) -> Res<'_, WithClause> {
    let (input, _) = ws_keyword("WITH")(input)?;
    let (input, _) = multispace1(input)?;
    alt((
        with_data_category,
        value(WithClause::SecurityEnforced, keyword("SECURITY_ENFORCED")),
        value(WithClause::SystemMode, keyword("SYSTEM_MODE")),
        value(WithClause::UserMode, keyword("USER_MODE")),
    ))(input)
}

fn with_data_category(input: &str) -> Res<'_, WithClause> {
    let (input, _) = keyword("DATA")(input)?;
    let (input, _) = multispace1(input)?;
    let (input, _) = keyword("CATEGORY")(input)?;
    let (input, _) = multispace1(input)?;
    let (input, conditions) = separated_list1(comma, data_category_condition)(input)?;
    Ok((
        input,
        WithClause::DataCategory(DataCategoryFilter { conditions }),
    ))
}

fn data_category_condition(input: &str) -> Res<'_, DataCategoryCondition> {
    let (input, category) = identifier(input)?;
    let (input, _) = multispace1(input)?;
    let (input, op) = alt((
        value(DataCategoryOperator::Above, keyword("ABOVE")),
        value(DataCategoryOperator::Below, keyword("BELOW")),
        value(DataCategoryOperator::At, keyword("AT")),
    ))(input)?;
    let (input, _) = multispace1(input)?;
    let (input, value_id) = identifier(input)?;
    Ok((
        input,
        DataCategoryCondition {
            category,
            op,
            value: value_id,
        },
    ))
}

fn group_by_clause(input: &str) -> Res<'_, GroupByClause> {
    let (input, _) = ws_keyword("GROUP")(input)?;
    let (input, _) = multispace1(input)?;
    let (input, _) = keyword("BY")(input)?;
    let (input, _) = multispace1(input)?;
    alt((
        group_by_rollup,
        group_by_cube,
        group_by_grouping_sets,
        group_by_standard,
    ))(input)
}

fn group_by_standard(input: &str) -> Res<'_, GroupByClause> {
    map(group_by_list, GroupByClause::Standard)(input)
}

fn group_by_rollup(input: &str) -> Res<'_, GroupByClause> {
    let (input, _) = keyword("ROLLUP")(input)?;
    let (input, items) = parenthesized_list(group_by_expression)(input)?;
    Ok((input, GroupByClause::Rollup(items)))
}

fn group_by_cube(input: &str) -> Res<'_, GroupByClause> {
    let (input, _) = keyword("CUBE")(input)?;
    let (input, items) = parenthesized_list(group_by_expression)(input)?;
    Ok((input, GroupByClause::Cube(items)))
}

fn group_by_grouping_sets(input: &str) -> Res<'_, GroupByClause> {
    let (input, _) = keyword("GROUPING")(input)?;
    let (input, _) = multispace1(input)?;
    let (input, _) = keyword("SETS")(input)?;
    let (input, sets) = delimited(
        preceded(multispace0, char('(')),
        separated_list1(
            comma,
            delimited(
                preceded(multispace0, char('(')),
                group_by_list,
                preceded(multispace0, char(')')),
            ),
        ),
        preceded(multispace0, char(')')),
    )(input)?;
    Ok((input, GroupByClause::GroupingSets(sets)))
}

fn group_by_list(input: &str) -> Res<'_, Vec<GroupByExpression>> {
    separated_list1(comma, group_by_expression)(input)
}

fn group_by_expression(input: &str) -> Res<'_, GroupByExpression> {
    preceded(
        multispace0,
        alt((
            map(function_call, GroupByExpression::Function),
            map(field_path, GroupByExpression::Field),
        )),
    )(input)
}

fn having_clause(input: &str) -> Res<'_, Expression> {
    let (input, _) = ws_keyword("HAVING")(input)?;
    expression(input)
}

fn order_by_clause(input: &str) -> Res<'_, OrderByClause> {
    let (input, _) = ws_keyword("ORDER")(input)?;
    let (input, _) = multispace1(input)?;
    let (input, _) = keyword("BY")(input)?;
    let (input, _) = multispace1(input)?;
    let (input, first) = order_by_field(input)?;
    let (input, mut rest) = many0(preceded(comma, order_by_field))(input)?;
    let mut fields = Vec::with_capacity(1 + rest.len());
    fields.push(first);
    fields.append(&mut rest);
    Ok((input, OrderByClause { fields }))
}

fn order_by_field(input: &str) -> Res<'_, OrderByField> {
    let (input, expression) = value_expression(input)?;
    let (input, direction) = opt(preceded(multispace1, order_direction))(input)?;
    let (input, nulls) = opt(preceded(multispace1, nulls_order))(input)?;
    Ok((
        input,
        OrderByField {
            expression,
            direction,
            nulls,
        },
    ))
}

fn order_direction(input: &str) -> Res<'_, OrderDirection> {
    alt((
        value(OrderDirection::Asc, keyword("ASC")),
        value(OrderDirection::Desc, keyword("DESC")),
    ))(input)
}

fn nulls_order(input: &str) -> Res<'_, NullsOrder> {
    let (input, _) = keyword("NULLS")(input)?;
    let (input, _) = multispace1(input)?;
    alt((
        value(NullsOrder::First, keyword("FIRST")),
        value(NullsOrder::Last, keyword("LAST")),
    ))(input)
}

fn limit_clause(input: &str) -> Res<'_, u32> {
    let (input, _) = ws_keyword("LIMIT")(input)?;
    unsigned_integer(input)
}

fn offset_clause(input: &str) -> Res<'_, u32> {
    let (input, _) = ws_keyword("OFFSET")(input)?;
    unsigned_integer(input)
}

fn for_clause(input: &str) -> Res<'_, ForClause> {
    let (input, _) = ws_keyword("FOR")(input)?;
    let (input, _) = multispace1(input)?;
    alt((
        value(ForClause::View, keyword("VIEW")),
        value(ForClause::Reference, keyword("REFERENCE")),
        value(ForClause::Share, keyword("SHARE")),
        for_update_clause,
    ))(input)
}

fn for_update_clause(input: &str) -> Res<'_, ForClause> {
    let (input, _) = keyword("UPDATE")(input)?;
    let (input, update) = opt(preceded(
        multispace1,
        alt((
            value(UpdateType::Tracking, keyword("TRACKING")),
            value(UpdateType::Viewstat, keyword("VIEWSTAT")),
        )),
    ))(input)?;
    Ok((input, ForClause::Update(update)))
}

fn expression(input: &str) -> Res<'_, Expression> {
    or_expression(input)
}

fn or_expression(input: &str) -> Res<'_, Expression> {
    let (mut input, mut expr) = and_expression(input)?;
    loop {
        match preceded(
            tuple((multispace0, keyword("OR"), multispace0)),
            and_expression,
        )(input)
        {
            Ok((next_input, rhs)) => {
                expr = Expression::Logical(Box::new(LogicalExpression {
                    left: expr,
                    operator: LogicalOperator::Or,
                    right: rhs,
                }));
                input = next_input;
            }
            Err(NomErr::Error(_)) => break,
            Err(e) => return Err(e),
        }
    }
    Ok((input, expr))
}

fn and_expression(input: &str) -> Res<'_, Expression> {
    let (mut input, mut expr) = not_expression(input)?;
    loop {
        match preceded(
            tuple((multispace0, keyword("AND"), multispace0)),
            not_expression,
        )(input)
        {
            Ok((next_input, rhs)) => {
                expr = Expression::Logical(Box::new(LogicalExpression {
                    left: expr,
                    operator: LogicalOperator::And,
                    right: rhs,
                }));
                input = next_input;
            }
            Err(NomErr::Error(_)) => break,
            Err(e) => return Err(e),
        }
    }
    Ok((input, expr))
}

fn not_expression(input: &str) -> Res<'_, Expression> {
    let (input, _) = multispace0(input)?;
    if let Ok((input, _)) = terminated(keyword("NOT"), multispace1)(input) {
        let (input, expr) = not_expression(input)?;
        Ok((input, Expression::Not(Box::new(expr))))
    } else {
        primary_expression(input)
    }
}

fn primary_expression(input: &str) -> Res<'_, Expression> {
    alt((
        map(
            delimited(
                preceded(multispace0, char('(')),
                expression,
                preceded(multispace0, char(')')),
            ),
            |expr| Expression::Parenthesized(Box::new(expr)),
        ),
        comparison_expression,
    ))(input)
}

fn comparison_expression(input: &str) -> Res<'_, Expression> {
    let (input, left) = value_expression(input)?;
    match between_comparison(left.clone(), input) {
        Ok(result) => return Ok(result),
        Err(NomErr::Error(_)) => {}
        Err(e) => return Err(e),
    }
    match in_comparison(left.clone(), input) {
        Ok(result) => return Ok(result),
        Err(NomErr::Error(_)) => {}
        Err(e) => return Err(e),
    }
    match includes_comparison(left.clone(), input) {
        Ok(result) => return Ok(result),
        Err(NomErr::Error(_)) => {}
        Err(e) => return Err(e),
    }
    match like_comparison(left.clone(), input) {
        Ok(result) => return Ok(result),
        Err(NomErr::Error(_)) => {}
        Err(e) => return Err(e),
    }
    binary_comparison(left, input)
}

fn binary_comparison(left: ValueExpression, input: &str) -> Res<'_, Expression> {
    let (input, operator) = comparison_operator(input)?;
    let (input, right) = value_expression(input)?;
    Ok((
        input,
        Expression::Comparison(ComparisonExpression::Binary {
            left,
            operator,
            right,
        }),
    ))
}

fn comparison_operator(input: &str) -> Res<'_, ComparisonOperator> {
    preceded(
        multispace0,
        alt((
            value(ComparisonOperator::Eq, char('=')),
            value(ComparisonOperator::Neq, tag_no_case("!=")),
            value(ComparisonOperator::Neq, tag_no_case("<>")),
            value(ComparisonOperator::Lte, tag_no_case("<=")),
            value(ComparisonOperator::Gte, tag_no_case(">=")),
            value(ComparisonOperator::Lt, char('<')),
            value(ComparisonOperator::Gt, char('>')),
        )),
    )(input)
}

fn between_comparison(left: ValueExpression, input: &str) -> Res<'_, Expression> {
    let (input, _) = multispace0(input)?;
    let (input, _) = keyword("BETWEEN")(input)?;
    let (input, _) = multispace1(input)?;
    let (input, lower) = value_expression(input)?;
    let (input, _) = multispace1(input)?;
    let (input, _) = keyword("AND")(input)?;
    let (input, _) = multispace1(input)?;
    let (input, upper) = value_expression(input)?;
    Ok((
        input,
        Expression::Comparison(ComparisonExpression::Between {
            expression: left,
            lower,
            upper,
        }),
    ))
}

fn like_comparison(left: ValueExpression, input: &str) -> Res<'_, Expression> {
    let (input, _) = multispace0(input)?;
    let (input, _) = keyword("LIKE")(input)?;
    let (input, _) = multispace1(input)?;
    let (input, pattern) = value_expression(input)?;
    let (input, escape) = opt(escape_clause)(input)?;
    Ok((
        input,
        Expression::Comparison(ComparisonExpression::Like {
            expression: left,
            pattern,
            escape,
        }),
    ))
}

fn escape_clause(input: &str) -> Res<'_, char> {
    let (input, _) = multispace1(input)?;
    let (input, _) = keyword("ESCAPE")(input)?;
    let (input, _) = multispace1(input)?;
    let (input, value) = string_literal(input)?;
    if value.chars().count() == 1 {
        Ok((input, value.chars().next().unwrap()))
    } else {
        Err(NomErr::Failure(VerboseError {
            errors: vec![(
                input,
                VerboseErrorKind::Context("escape must be single character"),
            )],
        }))
    }
}

fn in_comparison(left: ValueExpression, input: &str) -> Res<'_, Expression> {
    let (input, _) = multispace0(input)?;
    let (input, not_kw) = opt(terminated(keyword("NOT"), multispace1))(input)?;
    let (input, _) = keyword("IN")(input)?;
    let (input, _) = multispace0(input)?;
    let (input, _) = char('(')(input)?;
    let (input, _) = multispace0(input)?;
    let op = if not_kw.is_some() {
        InOperator::NotIn
    } else {
        InOperator::In
    };
    if starts_with_keyword(input, "SELECT") || starts_with_keyword(input, "FROM") {
        let (input, subquery) = query(input)?;
        let (input, _) = multispace0(input)?;
        let (input, _) = char(')')(input)?;
        Ok((
            input,
            Expression::Comparison(ComparisonExpression::InSubquery {
                left,
                operator: op,
                subquery: Subquery {
                    query: Box::new(subquery),
                    alias: None,
                },
            }),
        ))
    } else {
        let (input, list) = separated_list1(comma, value_expression)(input)?;
        let (input, _) = multispace0(input)?;
        let (input, _) = char(')')(input)?;
        Ok((
            input,
            Expression::Comparison(ComparisonExpression::In {
                left,
                operator: op,
                list,
            }),
        ))
    }
}

fn includes_comparison(left: ValueExpression, input: &str) -> Res<'_, Expression> {
    let (input, _) = multispace0(input)?;
    let (input, operator) = alt((
        value(IncludesOperator::Includes, keyword("INCLUDES")),
        value(IncludesOperator::Excludes, keyword("EXCLUDES")),
    ))(input)?;
    let (input, _) = multispace0(input)?;
    let (input, _) = char('(')(input)?;
    let (input, _) = multispace0(input)?;
    let (input, list) = separated_list1(comma, value_expression)(input)?;
    let (input, _) = multispace0(input)?;
    let (input, _) = char(')')(input)?;
    Ok((
        input,
        Expression::Comparison(ComparisonExpression::Includes {
            left,
            operator,
            list,
        }),
    ))
}

fn value_expression(input: &str) -> Res<'_, ValueExpression> {
    preceded(
        multispace0,
        alt((
            map(bind_variable, ValueExpression::BindVariable),
            map(function_call, ValueExpression::Function),
            map(literal, ValueExpression::Literal),
            map(field_path, ValueExpression::Field),
        )),
    )(input)
}

fn function_call(input: &str) -> Res<'_, FunctionCall> {
    let (input, name) = identifier(input)?;
    let (input, _) = multispace0(input)?;
    let (input, _) = char('(')(input)?;
    let (input, _) = multispace0(input)?;
    let (input, distinct) = opt(terminated(keyword("DISTINCT"), multispace1))(input)?;
    let (input, args) = separated_list0(comma, value_expression)(input)?;
    if distinct.is_some() && args.is_empty() {
        return Err(NomErr::Failure(VerboseError {
            errors: vec![(
                input,
                VerboseErrorKind::Context("DISTINCT requires arguments"),
            )],
        }));
    }
    let (input, _) = multispace0(input)?;
    let (input, _) = char(')')(input)?;
    Ok((
        input,
        FunctionCall {
            name,
            distinct: distinct.is_some(),
            arguments: args,
        },
    ))
}

fn typeof_expression(input: &str) -> Res<'_, TypeofSelect> {
    let (input, _) = keyword("TYPEOF")(input)?;
    let (input, _) = multispace1(input)?;
    let (input, field) = field_path(input)?;
    let (input, branches) = many1(typeof_branch)(input)?;
    let (input, else_fields) = opt(typeof_else_clause)(input)?;
    let (input, _) = multispace0(input)?;
    let (input, _) = keyword("END")(input)?;
    Ok((
        input,
        TypeofSelect {
            field,
            branches,
            else_fields: else_fields.unwrap_or_default(),
        },
    ))
}

fn typeof_branch(input: &str) -> Res<'_, TypeofBranch> {
    let (input, _) = ws_keyword("WHEN")(input)?;
    let (input, _) = multispace1(input)?;
    let (input, first_type) = identifier(input)?;
    let (input, mut additional) = many0(preceded(comma, identifier))(input)?;
    let mut type_names = Vec::with_capacity(1 + additional.len());
    type_names.push(first_type);
    type_names.append(&mut additional);
    let (input, _) = multispace1(input)?;
    let (input, _) = keyword("THEN")(input)?;
    let (input, _) = multispace1(input)?;
    let (input, fields) = separated_list1(comma, field_path)(input)?;
    Ok((input, TypeofBranch { type_names, fields }))
}

fn typeof_else_clause(input: &str) -> Res<'_, Vec<FieldPath>> {
    let (input, _) = ws_keyword("ELSE")(input)?;
    separated_list1(comma, field_path)(input)
}

fn bind_variable(input: &str) -> Res<'_, BindVariable> {
    let (input, _) = char(':')(input)?;
    let (mut input, first) = identifier(input)?;
    let mut path = vec![first];
    loop {
        match preceded(multispace0, preceded(char('.'), identifier))(input) {
            Ok((next, ident)) => {
                path.push(ident);
                input = next;
            }
            Err(NomErr::Error(_)) => break,
            Err(e) => return Err(e),
        }
    }
    Ok((input, BindVariable { path }))
}

fn optional_alias(input: &str) -> Res<'_, Option<Identifier>> {
    if let Ok((input, alias)) = preceded(multispace1, alias)(input) {
        Ok((input, Some(alias)))
    } else {
        Ok((input, None))
    }
}

fn alias(input: &str) -> Res<'_, Identifier> {
    let (input, _) = opt(terminated(keyword("AS"), multispace1))(input)?;
    identifier_not_reserved(input)
}

fn identifier_not_reserved(input: &str) -> Res<'_, Identifier> {
    let (input, ident) = identifier(input)?;
    if is_reserved_word(ident.as_str()) {
        Err(NomErr::Failure(VerboseError {
            errors: vec![(
                input,
                VerboseErrorKind::Context("reserved keyword cannot be used as alias"),
            )],
        }))
    } else {
        Ok((input, ident))
    }
}

fn field_path(input: &str) -> Res<'_, FieldPath> {
    let (input, _) = multispace0(input)?;
    let (mut input, first) = identifier(input)?;
    let mut parts = vec![first];
    loop {
        match preceded(multispace0, preceded(char('.'), identifier))(input) {
            Ok((next, ident)) => {
                parts.push(ident);
                input = next;
            }
            Err(NomErr::Error(_)) => break,
            Err(e) => return Err(e),
        }
    }
    Ok((input, FieldPath(parts)))
}

fn identifier(input: &str) -> Res<'_, Identifier> {
    let (input, first) = take_while_m_n(1, 1, is_identifier_start)(input)?;
    let (input, rest) = take_while(is_identifier_continue)(input)?;
    let mut value = String::with_capacity(first.len() + rest.len());
    value.push_str(first);
    value.push_str(rest);
    Ok((input, Identifier::new(value)))
}

fn literal(input: &str) -> Res<'_, Literal> {
    alt((
        map(string_literal, Literal::String),
        map(boolean_literal, Literal::Boolean),
        value(Literal::Null, keyword("NULL")),
        map(relative_date_literal, Literal::RelativeDate),
        map(date_literal, Literal::Date),
        map(number_literal, Literal::Number),
    ))(input)
}

fn string_literal(input: &str) -> Res<'_, String> {
    let (mut input, _) = char('\'')(input)?;
    let mut result = String::new();
    loop {
        if input.is_empty() {
            return Err(NomErr::Failure(VerboseError {
                errors: vec![(
                    input,
                    VerboseErrorKind::Context("unterminated string literal"),
                )],
            }));
        }
        if let Some(rest) = input.strip_prefix("''") {
            result.push('\'');
            input = rest;
            continue;
        }
        if let Some(rest) = input.strip_prefix('\'') {
            return Ok((rest, result));
        }
        if let Some(pos) = input.find('\'') {
            result.push_str(&input[..pos]);
            input = &input[pos..];
        } else {
            result.push_str(input);
            return Err(NomErr::Failure(VerboseError {
                errors: vec![("", VerboseErrorKind::Context("unterminated string literal"))],
            }));
        }
    }
}

fn boolean_literal(input: &str) -> Res<'_, bool> {
    alt((value(true, keyword("TRUE")), value(false, keyword("FALSE"))))(input)
}

fn number_literal(input: &str) -> Res<'_, NumberLiteral> {
    let (input, number) = recognize(tuple((
        opt(char('-')),
        digit1,
        opt(tuple((char('.'), digit1))),
        opt(tuple((one_of("eE"), opt(one_of("+-")), digit1))),
    )))(input)?;
    if number.contains(['.', 'e', 'E']) {
        Ok((input, NumberLiteral::Decimal(number.to_string())))
    } else if let Ok(value) = number.parse::<i64>() {
        Ok((input, NumberLiteral::Integer(value)))
    } else {
        Ok((input, NumberLiteral::Decimal(number.to_string())))
    }
}

fn date_literal(input: &str) -> Res<'_, DateLiteral> {
    alt((date_time_literal, date_only_literal, time_literal))(input)
}

fn date_only_literal(input: &str) -> Res<'_, DateLiteral> {
    map(
        recognize(tuple((
            year_digits,
            char('-'),
            month_digits,
            char('-'),
            day_digits,
        ))),
        |s: &str| DateLiteral::Date(s.to_string()),
    )(input)
}

fn date_time_literal(input: &str) -> Res<'_, DateLiteral> {
    map(
        recognize(tuple((
            year_digits,
            char('-'),
            month_digits,
            char('-'),
            day_digits,
            one_of("Tt"),
            hour_digits,
            char(':'),
            minute_digits,
            char(':'),
            second_digits,
            opt(tuple((char('.'), digit1))),
            opt(timezone_part),
        ))),
        |s: &str| DateLiteral::DateTime(s.to_string()),
    )(input)
}

fn time_literal(input: &str) -> Res<'_, DateLiteral> {
    map(
        recognize(tuple((
            hour_digits,
            char(':'),
            minute_digits,
            char(':'),
            second_digits,
            opt(tuple((char('.'), digit1))),
        ))),
        |s: &str| DateLiteral::Time(s.to_string()),
    )(input)
}

fn relative_date_literal(input: &str) -> Res<'_, RelativeDateLiteral> {
    let (input, ident) = identifier(input)?;
    let upper = ident.as_str().to_ascii_uppercase();
    if let Some(requirement) = relative_date_requirement(&upper) {
        match requirement {
            RelativeDateRequirement::None => Ok((
                input,
                RelativeDateLiteral {
                    name: ident,
                    offset: None,
                },
            )),
            RelativeDateRequirement::Required => {
                let (input, _) = char(':')(input)?;
                let (input, offset) = recognize(tuple((opt(one_of("+-")), digit1)))(input)?;
                let value = offset.parse::<i32>().map_err(|_| {
                    NomErr::Failure(VerboseError {
                        errors: vec![(
                            input,
                            VerboseErrorKind::Context("invalid relative date offset"),
                        )],
                    })
                })?;
                Ok((
                    input,
                    RelativeDateLiteral {
                        name: ident,
                        offset: Some(value),
                    },
                ))
            }
        }
    } else {
        Err(NomErr::Error(VerboseError {
            errors: vec![(input, VerboseErrorKind::Context("relative date literal"))],
        }))
    }
}

fn unsigned_integer(input: &str) -> Res<'_, u32> {
    map_res(preceded(multispace0, digit1), |s: &str| s.parse::<u32>())(input)
}

fn ws_keyword<'a>(kw: &'static str) -> impl FnMut(&'a str) -> Res<'a, &'a str> {
    move |input| preceded(multispace0, keyword(kw))(input)
}

fn keyword<'a>(kw: &'static str) -> impl FnMut(&'a str) -> Res<'a, &'a str> {
    move |input| {
        let (input, matched) = tag_no_case(kw)(input)?;
        if input.chars().next().map_or(false, is_identifier_continue) {
            Err(NomErr::Error(VerboseError {
                errors: vec![(input, VerboseErrorKind::Context("keyword boundary"))],
            }))
        } else {
            Ok((input, matched))
        }
    }
}

fn comma(input: &str) -> Res<'_, char> {
    delimited(multispace0, char(','), multispace0)(input)
}

fn parenthesized_list<'a, F, T>(mut parser: F) -> impl FnMut(&'a str) -> Res<'a, Vec<T>>
where
    F: FnMut(&'a str) -> Res<'a, T>,
{
    move |input| {
        let (input, _) = preceded(multispace0, char('('))(input)?;
        let (input, items) = separated_list1(comma, |inner| parser(inner))(input)?;
        let (input, _) = preceded(multispace0, char(')'))(input)?;
        Ok((input, items))
    }
}

fn starts_with_keyword(input: &str, keyword: &str) -> bool {
    let trimmed = input.trim_start();
    trimmed
        .get(..keyword.len())
        .map(|candidate| candidate.eq_ignore_ascii_case(keyword))
        .unwrap_or(false)
}

fn year_digits(input: &str) -> Res<'_, &str> {
    take_while_m_n(4, 4, |c: char| c.is_ascii_digit())(input)
}

fn month_digits(input: &str) -> Res<'_, &str> {
    take_while_m_n(2, 2, |c: char| c.is_ascii_digit())(input)
}

fn day_digits(input: &str) -> Res<'_, &str> {
    take_while_m_n(2, 2, |c: char| c.is_ascii_digit())(input)
}

fn hour_digits(input: &str) -> Res<'_, &str> {
    take_while_m_n(2, 2, |c: char| c.is_ascii_digit())(input)
}

fn minute_digits(input: &str) -> Res<'_, &str> {
    take_while_m_n(2, 2, |c: char| c.is_ascii_digit())(input)
}

fn second_digits(input: &str) -> Res<'_, &str> {
    take_while_m_n(2, 2, |c: char| c.is_ascii_digit())(input)
}

fn timezone_part(input: &str) -> Res<'_, &str> {
    recognize(alt((
        value((), one_of("Zz")),
        map(
            tuple((
                one_of("+-"),
                hour_digits,
                opt(tuple((char(':'), minute_digits))),
            )),
            |_| (),
        ),
    )))(input)
}

#[derive(Clone, Copy)]
enum RelativeDateRequirement {
    None,
    Required,
}

fn relative_date_requirement(name: &str) -> Option<RelativeDateRequirement> {
    RELATIVE_DATE_KEYWORDS
        .iter()
        .find(|(keyword, _)| keyword.eq(&name))
        .map(|(_, requirement)| *requirement)
}

const RELATIVE_DATE_KEYWORDS: &[(&str, RelativeDateRequirement)] = &[
    ("YESTERDAY", RelativeDateRequirement::None),
    ("TODAY", RelativeDateRequirement::None),
    ("TOMORROW", RelativeDateRequirement::None),
    ("LAST_WEEK", RelativeDateRequirement::None),
    ("THIS_WEEK", RelativeDateRequirement::None),
    ("NEXT_WEEK", RelativeDateRequirement::None),
    ("LAST_MONTH", RelativeDateRequirement::None),
    ("THIS_MONTH", RelativeDateRequirement::None),
    ("NEXT_MONTH", RelativeDateRequirement::None),
    ("LAST_QUARTER", RelativeDateRequirement::None),
    ("THIS_QUARTER", RelativeDateRequirement::None),
    ("NEXT_QUARTER", RelativeDateRequirement::None),
    ("LAST_YEAR", RelativeDateRequirement::None),
    ("THIS_YEAR", RelativeDateRequirement::None),
    ("NEXT_YEAR", RelativeDateRequirement::None),
    ("LAST_FISCAL_QUARTER", RelativeDateRequirement::None),
    ("THIS_FISCAL_QUARTER", RelativeDateRequirement::None),
    ("NEXT_FISCAL_QUARTER", RelativeDateRequirement::None),
    ("LAST_FISCAL_YEAR", RelativeDateRequirement::None),
    ("THIS_FISCAL_YEAR", RelativeDateRequirement::None),
    ("NEXT_FISCAL_YEAR", RelativeDateRequirement::None),
    ("LAST_7_DAYS", RelativeDateRequirement::None),
    ("LAST_30_DAYS", RelativeDateRequirement::None),
    ("LAST_60_DAYS", RelativeDateRequirement::None),
    ("LAST_90_DAYS", RelativeDateRequirement::None),
    ("NEXT_7_DAYS", RelativeDateRequirement::None),
    ("NEXT_30_DAYS", RelativeDateRequirement::None),
    ("NEXT_60_DAYS", RelativeDateRequirement::None),
    ("NEXT_90_DAYS", RelativeDateRequirement::None),
    ("LAST_N_DAYS", RelativeDateRequirement::Required),
    ("NEXT_N_DAYS", RelativeDateRequirement::Required),
    ("LAST_N_WEEKS", RelativeDateRequirement::Required),
    ("NEXT_N_WEEKS", RelativeDateRequirement::Required),
    ("LAST_N_MONTHS", RelativeDateRequirement::Required),
    ("NEXT_N_MONTHS", RelativeDateRequirement::Required),
    ("LAST_N_QUARTERS", RelativeDateRequirement::Required),
    ("NEXT_N_QUARTERS", RelativeDateRequirement::Required),
    ("LAST_N_YEARS", RelativeDateRequirement::Required),
    ("NEXT_N_YEARS", RelativeDateRequirement::Required),
    ("LAST_N_FISCAL_QUARTERS", RelativeDateRequirement::Required),
    ("NEXT_N_FISCAL_QUARTERS", RelativeDateRequirement::Required),
    ("LAST_N_FISCAL_YEARS", RelativeDateRequirement::Required),
    ("NEXT_N_FISCAL_YEARS", RelativeDateRequirement::Required),
    ("THIS_FISCAL_PERIOD", RelativeDateRequirement::None),
    ("LAST_FISCAL_PERIOD", RelativeDateRequirement::None),
    ("NEXT_FISCAL_PERIOD", RelativeDateRequirement::None),
    ("LAST_N_FISCAL_PERIODS", RelativeDateRequirement::Required),
    ("NEXT_N_FISCAL_PERIODS", RelativeDateRequirement::Required),
];

fn is_identifier_start(c: char) -> bool {
    c.is_ascii_alphabetic() || c == '_' || c == '$'
}

fn is_identifier_continue(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_' || c == '$'
}

fn is_reserved_word(candidate: &str) -> bool {
    matches!(
        candidate.to_ascii_uppercase().as_str(),
        "SELECT"
            | "FROM"
            | "WHERE"
            | "GROUP"
            | "BY"
            | "HAVING"
            | "ORDER"
            | "LIMIT"
            | "OFFSET"
            | "FOR"
            | "WITH"
            | "USING"
            | "SCOPE"
            | "AND"
            | "OR"
            | "NOT"
            | "ASC"
            | "DESC"
            | "NULLS"
            | "FIRST"
            | "LAST"
            | "VIEW"
            | "REFERENCE"
            | "UPDATE"
            | "SHARE"
            | "TYPEOF"
            | "WHEN"
            | "THEN"
            | "ELSE"
            | "END"
            | "IN"
            | "INCLUDES"
            | "EXCLUDES"
            | "LIKE"
            | "BETWEEN"
            | "NULL"
            | "TRUE"
            | "FALSE"
            | "DATA"
            | "CATEGORY"
            | "ROLLUP"
            | "CUBE"
            | "GROUPING"
            | "SETS"
            | "DISTINCT"
            | "ALL"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ident(value: &str) -> Identifier {
        Identifier::new(value)
    }

    fn field_path(parts: &[&str]) -> FieldPath {
        FieldPath::new(parts.iter().cloned().map(Identifier::new).collect())
    }

    #[test]
    fn parses_basic_query() {
        let query = parse_query("SELECT Name FROM Account").expect("parse");
        assert!(query.where_clause.is_none());
        assert_eq!(query.select.modifier, None);
        assert_eq!(query.select.items.len(), 1);
        match &query.select.items[0] {
            SelectItem::Field(field) => {
                assert_eq!(field.field, field_path(&["Name"]));
                assert!(field.alias.is_none());
            }
            item => panic!("unexpected select item: {item:?}"),
        }
        assert_eq!(query.from.sources.len(), 1);
        assert_eq!(query.from.sources[0].name, field_path(&["Account"]));
    }

    #[test]
    fn parses_where_and_order_by() {
        let text = "SELECT DISTINCT Name, BillingCity FROM Account a WHERE BillingState = 'CA' AND NumberOfEmployees > 10 ORDER BY Name ASC NULLS LAST LIMIT 50 OFFSET 5";
        let query = parse_query(text).expect("parse");
        assert_eq!(query.select.modifier, Some(SelectModifier::Distinct));
        assert_eq!(query.select.items.len(), 2);
        assert_eq!(
            query.from.sources[0].alias.as_ref().map(Identifier::as_str),
            Some("a")
        );
        let where_clause = query.where_clause.expect("where clause");
        match where_clause {
            Expression::Logical(logical) => {
                assert_eq!(logical.operator, LogicalOperator::And);
                match logical.left {
                    Expression::Comparison(ComparisonExpression::Binary {
                        left,
                        operator,
                        right,
                    }) => {
                        assert_eq!(operator, ComparisonOperator::Eq);
                        assert_eq!(left, ValueExpression::Field(field_path(&["BillingState"])));
                        assert_eq!(
                            right,
                            ValueExpression::Literal(Literal::String("CA".into()))
                        );
                    }
                    other => panic!("unexpected left clause: {other:?}"),
                }
            }
            other => panic!("unexpected where clause: {other:?}"),
        }
        let order = query.order_by.expect("order by");
        assert_eq!(order.fields.len(), 1);
        let field = &order.fields[0];
        assert_eq!(
            field.expression,
            ValueExpression::Field(field_path(&["Name"]))
        );
        assert_eq!(field.direction, Some(OrderDirection::Asc));
        assert_eq!(field.nulls, Some(NullsOrder::Last));
        assert_eq!(query.limit, Some(50));
        assert_eq!(query.offset, Some(5));
    }

    #[test]
    fn parses_subquery_and_relative_date() {
        let text = "SELECT Name, (SELECT LastName FROM Contacts WHERE CreatedDate >= LAST_N_DAYS:30) FROM Account";
        let query = parse_query(text).expect("parse");
        assert_eq!(query.select.items.len(), 2);
        match &query.select.items[1] {
            SelectItem::Subquery(subquery) => {
                let inner = &subquery.query;
                assert_eq!(inner.select.items.len(), 1);
                assert_eq!(inner.from.sources[0].name, field_path(&["Contacts"]));
                let where_clause = inner.where_clause.as_ref().expect("inner where");
                match where_clause {
                    Expression::Comparison(ComparisonExpression::Binary {
                        left,
                        operator,
                        right,
                    }) => {
                        assert_eq!(left, &ValueExpression::Field(field_path(&["CreatedDate"])));
                        assert_eq!(*operator, ComparisonOperator::Gte);
                        assert_eq!(
                            right,
                            &ValueExpression::Literal(Literal::RelativeDate(RelativeDateLiteral {
                                name: ident("LAST_N_DAYS"),
                                offset: Some(30),
                            }))
                        );
                    }
                    other => panic!("unexpected inner where: {other:?}"),
                }
            }
            other => panic!("unexpected select item: {other:?}"),
        }
    }

    #[test]
    fn parses_with_security_enforced() {
        let query = parse_query("SELECT Name FROM Account WITH SECURITY_ENFORCED").expect("parse");
        assert!(matches!(query.with, Some(WithClause::SecurityEnforced)));
    }

    #[test]
    fn parses_rollup_and_having() {
        let text = "SELECT COUNT(Id) total FROM Opportunity WHERE StageName IN ('Closed Won','Closed Lost') GROUP BY ROLLUP(StageName) HAVING COUNT(Id) > 5 ORDER BY COUNT(Id) DESC";
        let query = parse_query(text).expect("parse");
        assert_eq!(
            query.group_by,
            Some(GroupByClause::Rollup(vec![GroupByExpression::Field(
                field_path(&["StageName"])
            )]))
        );
        let having = query.having.expect("having");
        match having {
            Expression::Comparison(ComparisonExpression::Binary {
                left,
                operator,
                right,
            }) => {
                assert_eq!(operator, ComparisonOperator::Gt);
                assert_eq!(
                    right,
                    ValueExpression::Literal(Literal::Number(NumberLiteral::Integer(5)))
                );
                match left {
                    ValueExpression::Function(function) => {
                        assert_eq!(function.name.as_str(), "COUNT");
                        assert_eq!(function.arguments.len(), 1);
                    }
                    other => panic!("unexpected having left: {other:?}"),
                }
            }
            other => panic!("unexpected having: {other:?}"),
        }
    }

    #[test]
    fn parses_leading_from() {
        let query =
            parse_query("FROM Account SELECT Name, BillingCity ORDER BY Name").expect("parse");
        assert_eq!(query.from.sources[0].name, field_path(&["Account"]));
        assert_eq!(query.select.items.len(), 2);
    }

    #[test]
    fn parses_typeof_expression() {
        let text = "SELECT TYPEOF What WHEN Account THEN Name, BillingCity WHEN Opportunity THEN Amount ELSE Subject END FROM Event";
        let query = parse_query(text).expect("parse");
        match &query.select.items[0] {
            SelectItem::Typeof(typeof_expr) => {
                assert_eq!(typeof_expr.field, field_path(&["What"]));
                assert_eq!(typeof_expr.branches.len(), 2);
                assert_eq!(typeof_expr.else_fields, vec![field_path(&["Subject"])]);
            }
            other => panic!("expected TYPEOF select, found {other:?}"),
        }
    }

    #[test]
    fn parses_with_data_category() {
        let query = parse_query(
            "SELECT Name FROM Knowledge__kav WITH DATA CATEGORY Geography__c ABOVE usa__c",
        )
        .expect("parse");
        match query.with.expect("with") {
            WithClause::DataCategory(filter) => {
                assert_eq!(filter.conditions.len(), 1);
                let condition = &filter.conditions[0];
                assert_eq!(condition.category, ident("Geography__c"));
                assert_eq!(condition.op, DataCategoryOperator::Above);
                assert_eq!(condition.value, ident("usa__c"));
            }
            other => panic!("unexpected with clause: {other:?}"),
        }
    }

    #[test]
    fn parses_for_update_tracking() {
        let query = parse_query("SELECT Id FROM Contact FOR UPDATE TRACKING").expect("parse");
        assert_eq!(
            query.for_clause,
            Some(ForClause::Update(Some(UpdateType::Tracking)))
        );
    }

    #[test]
    fn parses_bind_variable() {
        let query = parse_query("SELECT Id FROM Case WHERE CreatedDate > :myParams.created")
            .expect("parse");
        let where_clause = query.where_clause.expect("where");
        match where_clause {
            Expression::Comparison(ComparisonExpression::Binary { right, .. }) => {
                assert_eq!(
                    right,
                    ValueExpression::BindVariable(BindVariable {
                        path: vec![ident("myParams"), ident("created")],
                    })
                );
            }
            other => panic!("unexpected where clause: {other:?}"),
        }
    }
}
