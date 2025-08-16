use serde_json::{Map, Value};
use sqlx::{Postgres, QueryBuilder, query_builder::Separated};

pub fn handle_operators(
    builder: &mut QueryBuilder<'_, Postgres>,
    object: &Map<String, Value>,
    conn: Option<&str>,
) {
    let conn = format!(" {} ", conn.unwrap_or("AND"));

    let mut seperated = builder.separated(" AND ");

    let mut and_operator_builder = QueryBuilder::<Postgres>::new("");
    let mut or_operator_builder = QueryBuilder::<Postgres>::new("");
    let mut not_operator_builder = QueryBuilder::<Postgres>::new("");

    let mut field_operator_builder = QueryBuilder::<Postgres>::new("(");
    let mut field_seperated = field_operator_builder.separated(conn.as_str());

    for (key, value) in object {
        match key.as_str() {
            "_and" => {
                if let Value::Object(o) = value {
                    handle_operators(&mut and_operator_builder, o, Some("AND"));
                    if and_operator_builder.sql() != "()" {
                        seperated.push(and_operator_builder.sql());
                    }
                }
            }
            "_not" => {
                if let Value::Object(o) = value {
                    handle_operators(&mut not_operator_builder, o, Some("AND NOT"));
                    if not_operator_builder.sql() != "()" {
                        seperated.push(not_operator_builder.sql());
                    }
                }
            }
            "_or" => {
                if let Value::Object(o) = value {
                    handle_operators(&mut or_operator_builder, o, Some("OR"));
                    if or_operator_builder.sql() != "()" {
                        seperated.push(or_operator_builder.sql());
                    }
                }
            }
            _ => {
                if let Value::Object(o) = value {
                    handle_field_operators(&mut field_seperated, key, o);
                }
            }
        }
    }

    field_seperated.push_unseparated(")");

    if field_operator_builder.sql() != "()" {
        seperated.push(field_operator_builder.sql());
    }
}

pub fn handle_field_operators(
    seperated: &mut Separated<'_, '_, Postgres, &str>,
    field: &str,
    operators: &Map<String, Value>,
) {
    for (op, value) in operators {
        match op.as_str() {
            "_eq" => handle_eq_operator(seperated, field, value),
            "_in" => handle_in_operator(seperated, field, value),
            "_gt" => handle_gt_operator(seperated, field, value),
            "_lt" => handle_lt_operator(seperated, field, value),
            "_gte" => handle_gte_operator(seperated, field, value),
            "_lte" => handle_lte_operator(seperated, field, value),
            "_nin" => handle_nin_operator(seperated, field, value),
            "_neq" => handle_neq_operator(seperated, field, value),
            "_is_null" => handle_is_null_operator(seperated, field, value),
            _ => {
                // Not implemented
            }
        }
    }
}

pub fn handle_eq_operator(
    seperated: &mut Separated<'_, '_, Postgres, &str>,
    field: &str,
    value: &Value,
) {
    match value {
        Value::String(s) => {
            seperated.push(format!("a.{} = '{}'", field, s));
        }
        Value::Number(n) => {
            seperated.push(format!("a.{} = {}", field, n));
        }
        Value::Bool(b) => {
            seperated.push(format!("a.{} = {}", field, b));
        }
        _ => {
            // Not implemented
        }
    }
}

pub fn handle_neq_operator(
    seperated: &mut Separated<'_, '_, Postgres, &str>,
    field: &str,
    value: &Value,
) {
    match value {
        Value::String(s) => {
            seperated.push(format!("a.{} != '{}'", field, s));
        }
        Value::Number(n) => {
            seperated.push(format!("a.{} != {}", field, n));
        }
        Value::Bool(b) => {
            seperated.push(format!("a.{} != {}", field, b));
        }
        _ => {
            // Not implemented
        }
    }
}

pub fn handle_in_operator(
    seperated: &mut Separated<'_, '_, Postgres, &str>,
    field: &str,
    value: &Value,
) {
    match value {
        Value::Array(a) => {
            let mut builder = QueryBuilder::<Postgres>::new("(");
            let mut s = builder.separated(" OR ");

            for value in a {
                match value {
                    Value::String(st) => {
                        s.push(format!("a.{} = '{}'", field, st));
                    }
                    Value::Number(n) => {
                        s.push(format!("a.{} = {}", field, n));
                    }
                    Value::Bool(b) => {
                        s.push(format!("a.{} = {}", field, b));
                    }
                    _ => {
                        // Not implemented
                    }
                }
            }

            s.push_unseparated(")");
            seperated.push(builder.sql());
        }
        _ => {
            // Not implemented
        }
    }
}

pub fn handle_nin_operator(
    seperated: &mut Separated<'_, '_, Postgres, &str>,
    field: &str,
    value: &Value,
) {
    match value {
        Value::Array(a) => {
            let mut builder = QueryBuilder::<Postgres>::new("(");
            let mut s = builder.separated(" AND ");
            for value in a {
                match value {
                    Value::String(st) => {
                        s.push(format!("a.{} != '{}'", field, st));
                    }
                    Value::Number(n) => {
                        s.push(format!("a.{} != {}", field, n));
                    }
                    Value::Bool(b) => {
                        s.push(format!("a.{} != {}", field, b));
                    }
                    _ => {
                        // Not implemented
                    }
                }
            }

            s.push_unseparated(")");
            seperated.push(builder.sql());
        }
        _ => {
            // handle_neq_operator(builder, field, value, conn);
        }
    }
}

pub fn handle_gt_operator(
    seperated: &mut Separated<'_, '_, Postgres, &str>,
    field: &str,
    value: &Value,
) {
    match value {
        Value::String(s) => {
            seperated.push(format!("a.{} > '{}'", field, s));
        }
        Value::Number(n) => {
            seperated.push(format!("a.{} > {}", field, n));
        }
        Value::Bool(b) => {
            seperated.push(format!("a.{} > {}", field, b));
        }
        _ => {
            // Not implemented
        }
    }
}

pub fn handle_gte_operator(
    seperated: &mut Separated<'_, '_, Postgres, &str>,
    field: &str,
    value: &Value,
) {
    match value {
        Value::String(s) => {
            seperated.push(format!("a.{} >= '{}'", field, s));
        }
        Value::Number(n) => {
            seperated.push(format!("a.{} >= {}", field, n));
        }
        Value::Bool(b) => {
            seperated.push(format!("a.{} >= {}", field, b));
        }
        _ => {
            // Not implemented
        }
    }
}

pub fn handle_lt_operator(
    seperated: &mut Separated<'_, '_, Postgres, &str>,
    field: &str,
    value: &Value,
) {
    match value {
        Value::String(s) => {
            seperated.push(format!("a.{} < '{}'", field, s));
        }
        Value::Number(n) => {
            seperated.push(format!("a.{} < {}", field, n));
        }
        Value::Bool(b) => {
            seperated.push(format!("a.{} < {}", field, b));
        }
        _ => {
            // Not implemented
        }
    }
}

pub fn handle_lte_operator(
    seperated: &mut Separated<'_, '_, Postgres, &str>,
    field: &str,
    value: &Value,
) {
    match value {
        Value::String(s) => {
            seperated.push(format!("a.{} <= '{}'", field, s));
        }
        Value::Number(n) => {
            seperated.push(format!("a.{} <= {}", field, n));
        }
        Value::Bool(b) => {
            seperated.push(format!("a.{} <= {}", field, b));
        }
        _ => {
            // Not implemented
        }
    }
}

pub fn handle_is_null_operator(
    seperated: &mut Separated<'_, '_, Postgres, &str>,
    field: &str,
    value: &Value,
) {
    if let Value::Bool(b) = value {
        if *b {
            seperated.push(format!("a.{} IS NULL", field));
        } else {
            seperated.push(format!("a.{} IS NOT NULL", field));
        }
    }
}
