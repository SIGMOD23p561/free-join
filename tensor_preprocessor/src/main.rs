#![allow(warnings)]
use sqlparser::ast::*;
use sqlparser::dialect::GenericDialect;
use sqlparser::parser::Parser;

use std::env;
use std::fs;

use std::collections::HashMap;

fn main() {
    let args: Vec<String> = env::args().collect();
    let f = &args[1];
    let mode = &args[2];
    let name = &args[3];

    let sql = fs::read_to_string(f).expect("Unable to read file");

    let dialect = GenericDialect {};

    let mut ast = Parser::parse_sql(&dialect, &sql).unwrap();
    assert_eq!(ast.len(), 1, "File must contain exactly 1 statement");

    let mut stmt = ast.pop().unwrap();

    if let Statement::Query(q) = &mut stmt {
        if let SetExpr::Select(q) = &mut q.body {
            let mut filters: Vec<Expr> = vec![];
            let mut joins: Vec<Expr> = vec![];

            if let Some(sel) = &q.selection {
                get_joins_and_filters(sel, &mut joins, &mut filters);
            }

            let mut from_aliases: HashMap<String, TableWithJoins> = HashMap::new();
            map_from_aliases(&q.from, &mut from_aliases);

            let mut filter_aliases: HashMap<String, Vec<String>> = HashMap::new();
            map_filter_aliases(&filters, &mut filter_aliases);

            let mut join_aliases: HashMap<String, Vec<String>> = HashMap::new();
            map_join_aliases(&joins, &mut join_aliases);

            if mode == "counts" {
                println!("SELECT COUNT(*) FROM ");
                for table in &q.from {
                    println!("{}, ", table.to_string());
                }
                if let Some(sel) = &q.selection {
                    println!("WHERE {};", sel.to_string());
                }
            }

            if mode == "filters" {
                // constructs the filter queries for each table matching by alias
                for (filter_alias, parsed_filters) in &filter_aliases {
                    for (from_alias, parsed_from) in &from_aliases {
                        if filter_alias == from_alias {
                            if let TableFactor::Table {
                                name: _,
                                alias,
                                args: _,
                                with_hints: _,
                            } = &parsed_from.relation
                            {
                                if let Some(a) = &alias {
                                    let alias_string = a.to_string();
                                    if let Some(columns) = join_aliases.get(&alias_string) {
                                        println!("COPY (SELECT {} FROM {} WHERE {}) TO '../data/{}/{}.parquet' (FORMAT 'parquet');", columns.join(", "), parsed_from, parsed_filters.join(" AND "), name, alias_string);
                                    }
                                }
                            }
                        }
                    }
                }
            }

            if mode == "joins" {
                println!("SELECT COUNT(*)");

                print!(" FROM ");
                for (from_alias, parsed_from) in &from_aliases {
                    let mut was_filtered = false;
                    for (filter_alias, _) in &filter_aliases {
                        if filter_alias == from_alias {
                            if let TableFactor::Table {
                                name: _,
                                alias,
                                args: _,
                                with_hints: _,
                            } = &parsed_from.relation
                            {
                                if let Some(a) = &alias {
                                    print!("{}, ", a.to_string());
                                    was_filtered = true;
                                    break;
                                }
                            }
                        }
                    }
                    if !was_filtered {
                        print!("{}, ", parsed_from.to_string());
                    }
                }

                println!("");
                println!("WHERE {}", (&joins)[0]);
                for join in &joins[1..] {
                    println!("AND {}", join.to_string());
                }

                println!(";");
            }
        } else {
            panic!("Only SELECT-PROJECT-JOIN queries are supported");
        }
    } else {
        panic!("Only SELECT queries are supported");
    }
}

// maps table aliases (ie. cn) to their full from statements (ie. company_name AS cn)
fn map_from_aliases(froms: &Vec<TableWithJoins>, aliases: &mut HashMap<String, TableWithJoins>) {
    for from in froms {
        if let TableFactor::Table {
            name: _,
            alias,
            args: _,
            with_hints: _,
        } = &from.relation
        {
            if let Some(a) = &alias {
                let alias_string = a.to_string();
                aliases.entry(alias_string).or_insert(from.clone());
            }
        }
    }
}

// maps table aliases (ie. cn) to their filter statements (ie. cn.country_code = '[us]')
fn map_filter_aliases(filters: &Vec<Expr>, aliases: &mut HashMap<String, Vec<String>>) {
    for fil in filters {
        let alias_string = get_filter_alias(fil);
        aliases.entry(alias_string.clone()).or_insert(vec![]);
        if let Some(fs) = aliases.get_mut(&alias_string) {
            fs.push((*fil).to_string());
        }
    }
}

// gets the filter alias from the expression
fn get_filter_alias(filter: &Expr) -> String {
    match filter {
        Expr::CompoundIdentifier(identifier) => {
            let alias_string = (&identifier[0]).to_string();
            return alias_string;
        }
        Expr::Nested(nest) => {
            return get_filter_alias(nest);
        }
        Expr::IsNull(is_null) => {
            return get_filter_alias(is_null);
        }
        Expr::IsNotNull(is_not_null) => {
            return get_filter_alias(is_not_null);
        }
        Expr::InList {
            expr,
            list: _,
            negated: _,
        } => {
            return get_filter_alias(expr);
        }
        Expr::Between {
            expr,
            negated: _,
            low: _,
            high: _,
        } => {
            return get_filter_alias(expr);
        }
        Expr::UnaryOp { op: _, expr } => {
            return get_filter_alias(expr);
        }
        Expr::BinaryOp {
            left: l,
            op: o,
            right: r,
        } => match (&**l, o, &**r) {
            (Expr::CompoundIdentifier(l), _, _) => {
                let alias_string = (&l[0]).to_string();
                return alias_string;
            }
            (_, _, Expr::CompoundIdentifier(r)) => {
                let alias_string = (&r[0]).to_string();
                return alias_string;
            }
            (e_1, _, e_2) => {
                let left_branch = get_filter_alias(e_1);
                if left_branch.is_empty() {
                    return get_filter_alias(e_2);
                }
                return left_branch;
            }
        },
        _ => return String::new(),
    }
}

// maps table aliases (ie. cn) to their joined columns (ie. movie_id)
fn map_join_aliases(joins: &Vec<Expr>, aliases: &mut HashMap<String, Vec<String>>) {
    for join in joins {
        let aliases_and_columns = get_join_aliases_and_columns(join);
        aliases
            .entry(aliases_and_columns[0].clone())
            .or_insert(vec![]);
        aliases
            .entry(aliases_and_columns[2].clone())
            .or_insert(vec![]);
        if let Some(js) = aliases.get_mut(&aliases_and_columns[0]) {
            if !js.contains(&aliases_and_columns[1].clone()) {
                js.push(aliases_and_columns[1].clone());
            }
        }
        if let Some(js) = aliases.get_mut(&aliases_and_columns[2]) {
            if !js.contains(&aliases_and_columns[3].clone()) {
                js.push(aliases_and_columns[3].clone());
            }
        }
    }
}

// gets a vector representing the aliases and columns of a given join in the form [left_alias, left_column, right_alias, right_column]
fn get_join_aliases_and_columns(join: &Expr) -> Vec<String> {
    if let Expr::BinaryOp {
        left: l,
        op: o,
        right: r,
    } = join
    {
        match (&**l, o, &**r) {
            (
                Expr::CompoundIdentifier(left_ident),
                BinaryOperator::Eq,
                Expr::CompoundIdentifier(right_ident),
            ) => {
                return vec![
                    (&left_ident[0]).to_string(),
                    (&left_ident[1]).to_string(),
                    (&right_ident[0]).to_string(),
                    (&right_ident[1]).to_string(),
                ];
            }
            _ => return vec![String::new(), String::new(), String::new(), String::new()],
        }
    }
    return vec![String::new(), String::new(), String::new(), String::new()];
}

// gets all of the joins and filters from the expression
fn get_joins_and_filters(e: &Expr, joins: &mut Vec<Expr>, filters: &mut Vec<Expr>) {
    if let Expr::BinaryOp {
        left: l,
        op: o,
        right: r,
    } = e
    {
        match (&**l, o, &**r) {
            (Expr::CompoundIdentifier(_), BinaryOperator::Eq, Expr::CompoundIdentifier(_)) => {
                joins.push(e.clone())
            }
            (e_l, BinaryOperator::And, e_r) => {
                get_joins_and_filters(e_l, joins, filters);
                get_joins_and_filters(e_r, joins, filters)
            }
            _ => filters.push(e.clone()),
        }
    } else {
        filters.push(e.clone());
    }
}
