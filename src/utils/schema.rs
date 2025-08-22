use serde_json::{Map, Value};
use sqlx::{Postgres, QueryBuilder, query_builder::Separated};

use crate::database::Schema;

pub fn handle_join(builder: &mut QueryBuilder<'_, Postgres>, object: &Map<String, Value>) {
    for (key, value) in object {
        if let Value::Object(o) = value {
            if !is_object_empty(o) {
                match key.as_str() {
                    "collection" => {
                        builder.push(" LEFT JOIN collections c ON c.id = collection_id ");
                    }
                    "nft" => {
                        builder.push(" LEFT JOIN nfts n ON n.id = nft_id ");
                    }
                    _ => {
                        // Not implemented
                    }
                }
            }
        }
    }
}

pub fn handle_query(
    builder: &mut QueryBuilder<'_, Postgres>,
    object: &Map<String, Value>,
    conn: &str,
    schema: Schema,
) {
    let conn = format!(" {} ", conn);
    let mut seperated = builder.separated(" AND ");

    let mut and_operator_builder = QueryBuilder::<Postgres>::new("");
    let mut or_operator_builder = QueryBuilder::<Postgres>::new("");
    let mut not_operator_builder = QueryBuilder::<Postgres>::new("");

    // Available for nft and collection schema only
    let mut activity_builder = match schema.to_owned() {
        Schema::Collections => QueryBuilder::<Postgres>::new(
            r#"
            id IN (
                SELECT collection_id from activities
                WHERE
            "#,
        ),
        Schema::Nfts => QueryBuilder::<Postgres>::new(
            r#"
            id IN (
                SELECT nft_id from activities
                WHERE
            "#,
        ),
        _ => QueryBuilder::<Postgres>::new(""),
    };

    // Available for nft and collection schema only
    let mut attribute_builder = match schema.to_owned() {
        Schema::Collections => QueryBuilder::<Postgres>::new(
            r#"
            id IN (
                SELECT collection_id from attributes
                WHERE
            "#,
        ),
        Schema::Nfts => QueryBuilder::<Postgres>::new(
            r#"
            id IN (
                SELECT nft_id from attributes
                WHERE
            "#,
        ),
        _ => QueryBuilder::<Postgres>::new(""),
    };

    // Available for nft and collection schema only
    let mut bid_builder = match schema.to_owned() {
        Schema::Collections => QueryBuilder::<Postgres>::new(
            r#"
            id IN (
                SELECT collection_id from bids
                WHERE
            "#,
        ),
        Schema::Nfts => QueryBuilder::<Postgres>::new(
            r#"
            id IN (
                SELECT nft_id from bids
                WHERE
            "#,
        ),
        _ => QueryBuilder::<Postgres>::new(""),
    };

    // Available for nft schema only
    let mut listing_builder = QueryBuilder::<Postgres>::new(
        r#"
        id IN (
            SELECT nft_id FROM listings
            WHERE
        "#,
    );

    // Available for attribute, activity, bid and listing schema
    let mut nft_builder = QueryBuilder::<Postgres>::new(
        r#"
        nft_id IN (
            SELECT id FROM (
                SELECT 
                    nfts.*,
                    CASE
                        WHEN rarity IS NOT NULL
                        THEN RANK () OVER (
                            PARTITION BY collection_id
                            ORDER BY rarity DESC
                        )
                        END                                 AS ranking
                FROM nfts
            )
            WHERE
        "#,
    );

    // Available for attribute, activity, bid, and nft schema
    let mut collection_builder = QueryBuilder::<Postgres>::new(
        r#"
        collection_id IN (
            SELECT id FROM collections
            WHERE
        "#,
    );

    let mut field_operator_builder = QueryBuilder::<Postgres>::new("(");
    let mut field_seperated = field_operator_builder.separated(conn.as_str());

    for (key, value) in object {
        match key.as_str() {
            "_and" => {
                if let Value::Object(o) = value {
                    handle_query(&mut and_operator_builder, o, "AND", schema.to_owned());
                    if and_operator_builder.sql() != "()" {
                        seperated.push(and_operator_builder.sql());
                    }
                }
            }
            "_not" => {
                if let Value::Object(o) = value {
                    handle_query(&mut not_operator_builder, o, "AND NOT", schema.to_owned());
                    if not_operator_builder.sql() != "()" {
                        seperated.push(not_operator_builder.sql());
                    }
                }
            }
            "_or" => {
                if let Value::Object(o) = value {
                    handle_query(&mut or_operator_builder, o, "OR", schema.to_owned());
                    if or_operator_builder.sql() != "()" {
                        seperated.push(or_operator_builder.sql());
                    }
                }
            }
            "nft" => {
                if let Value::Object(o) = value {
                    handle_query(&mut nft_builder, o, "AND", Schema::Nfts);
                    if !nft_builder.sql().trim().ends_with("WHERE") {
                        nft_builder.push(")");
                        seperated.push(nft_builder.sql());
                    }
                }
            }
            "activity" => {
                if !activity_builder.sql().trim().is_empty() {
                    if let Value::Object(o) = value {
                        handle_query(&mut activity_builder, o, "AND", Schema::Activities);
                        if !activity_builder.sql().trim().ends_with("WHERE") {
                            activity_builder.push(")");
                            seperated.push(activity_builder.sql());
                        }
                    }
                }
            }
            "bid" => {
                if !bid_builder.sql().trim().is_empty() {
                    if let Value::Object(o) = value {
                        handle_query(&mut bid_builder, o, "AND", Schema::Bids);
                        if !bid_builder.sql().trim().ends_with("WHERE") {
                            bid_builder.push(")");
                            seperated.push(bid_builder.sql());
                        }
                    }
                }
            }
            "attribute" => {
                if !attribute_builder.sql().trim().is_empty() {
                    if let Value::Object(o) = value {
                        handle_query(&mut attribute_builder, o, "AND", Schema::Attributes);
                        if !attribute_builder.sql().trim().ends_with("WHERE") {
                            attribute_builder.push(")");
                            seperated.push(attribute_builder.sql());
                        }
                    }
                }
            }
            "collection" => {
                if let Value::Object(o) = value {
                    handle_query(&mut collection_builder, o, "AND", Schema::Collections);
                    if !collection_builder.sql().trim().ends_with("WHERE") {
                        collection_builder.push(")");
                        seperated.push(collection_builder.sql());
                    }
                }
            }
            "listing" => {
                if let Value::Object(o) = value {
                    handle_query(&mut listing_builder, o, "AND", Schema::Listings);
                    if !listing_builder.sql().trim().ends_with("WHERE") {
                        listing_builder.push(")");
                        seperated.push(listing_builder.sql());
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

pub fn handle_order(builder: &mut QueryBuilder<'_, Postgres>, object: &Map<String, Value>) {
    let mut order_builder = QueryBuilder::<Postgres>::new(" ");
    let mut order_seperated_builder = order_builder.separated(",");

    for (key, value) in object {
        match value {
            Value::String(s) => {
                order_seperated_builder.push(format!("{} {}", key, s.replace("_", " ")));
            }
            Value::Object(o) => {
                for (sub_key, value) in o {
                    if let Value::String(s) = value {
                        let table = match key.as_str() {
                            "collection" => "c",
                            "nft" => "n",
                            _ => "",
                        };

                        if !table.is_empty() {
                            order_seperated_builder.push(format!(
                                "{}.{} {}",
                                table,
                                sub_key,
                                s.replace("_", " ")
                            ));
                        }
                    }
                }
            }
            _ => {
                // Not implemented
            }
        }
    }

    builder.push(order_builder.sql());
}

pub fn handle_nested_order(builder: &mut QueryBuilder<'_, Postgres>, object: &Map<String, Value>) {
    let mut nested_order = QueryBuilder::<Postgres>::new("");
    let mut nested_order_seperated_builder = nested_order.separated(",");

    for (key, value) in object {
        if let Value::Object(o) = value {
            if !is_object_empty(o) {
                match key.as_str() {
                    "collection" => {
                        nested_order_seperated_builder.push(
                            r#"
                            collections AS (SELECT * FROM collections)
                            "#,
                        );
                    }
                    "nft" => {
                        nested_order_seperated_builder.push(
                            r#"
                            nfts AS (
                                SELECT 
                                    nfts.*,
                                    CASE
                                        WHEN rarity IS NOT NULL
                                        THEN RANK () OVER (
                                            PARTITION BY collection_id
                                            ORDER BY rarity DESC
                                        )
                                        END                                 AS ranking
                                FROM nfts
                            )
                            "#,
                        );
                    }
                    _ => {
                        // Not implemented
                    }
                }
            }
        }
    }

    builder.push(nested_order.sql());
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
            seperated.push(format!("{} = '{}'", field, s));
        }
        Value::Number(n) => {
            seperated.push(format!("{} = {}", field, n));
        }
        Value::Bool(b) => {
            seperated.push(format!("{} = {}", field, b));
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
            seperated.push(format!("{} != '{}'", field, s));
        }
        Value::Number(n) => {
            seperated.push(format!("{} != {}", field, n));
        }
        Value::Bool(b) => {
            seperated.push(format!("{} != {}", field, b));
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
                        s.push(format!("{} = '{}'", field, st));
                    }
                    Value::Number(n) => {
                        s.push(format!("{} = {}", field, n));
                    }
                    Value::Bool(b) => {
                        s.push(format!("{} = {}", field, b));
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
                        s.push(format!("{} != '{}'", field, st));
                    }
                    Value::Number(n) => {
                        s.push(format!("{} != {}", field, n));
                    }
                    Value::Bool(b) => {
                        s.push(format!("{} != {}", field, b));
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
            seperated.push(format!("{} > '{}'", field, s));
        }
        Value::Number(n) => {
            seperated.push(format!("{} > {}", field, n));
        }
        Value::Bool(b) => {
            seperated.push(format!("{} > {}", field, b));
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
            seperated.push(format!("{} >= '{}'", field, s));
        }
        Value::Number(n) => {
            seperated.push(format!("{} >= {}", field, n));
        }
        Value::Bool(b) => {
            seperated.push(format!("{} >= {}", field, b));
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
            seperated.push(format!("{} < '{}'", field, s));
        }
        Value::Number(n) => {
            seperated.push(format!("{} < {}", field, n));
        }
        Value::Bool(b) => {
            seperated.push(format!("{} < {}", field, b));
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
            seperated.push(format!("{} <= '{}'", field, s));
        }
        Value::Number(n) => {
            seperated.push(format!("{} <= {}", field, n));
        }
        Value::Bool(b) => {
            seperated.push(format!("{} <= {}", field, b));
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
            seperated.push(format!("{} IS NULL", field));
        } else {
            seperated.push(format!("{} IS NOT NULL", field));
        }
    }
}

pub fn is_object_empty(o: &Map<String, Value>) -> bool {
    for (_, value) in o {
        match value {
            Value::Null => continue,
            _ => {
                return false;
            }
        }
    }

    true
}
