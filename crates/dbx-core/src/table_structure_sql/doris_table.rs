//! Apache Doris table model support for `CREATE TABLE`.
//!
//! Doris has no `PRIMARY KEY` clause. The columns flagged as primary key in the
//! editor are the table's *key columns*; they must lead the column list, and the
//! table model (`DUPLICATE` / `UNIQUE` / `AGGREGATE KEY`) decides how rows that
//! share a key are stored or merged. Every rule violation is reported as a hard
//! error so the user fixes the draft instead of the builder silently reordering
//! columns.

use super::types::{EditableStructureColumn, TableStructureSqlOptions};
use super::util::{clean, quote_string};

pub(super) const DORIS_KEY_MODELS: &[&str] = &["DUPLICATE", "UNIQUE", "AGGREGATE"];
const DORIS_DEFAULT_KEY_MODEL: &str = "DUPLICATE";
/// Aggregation types a value column may carry in an AGGREGATE KEY table.
pub(super) const DORIS_AGGREGATION_TYPES: &[&str] =
    &["SUM", "MAX", "MIN", "REPLACE", "REPLACE_IF_NOT_NULL", "HLL_UNION", "BITMAP_UNION", "QUANTILE_UNION"];
/// Doris refuses floating point, unbounded string and semi-structured types as keys.
const DORIS_NON_KEY_TYPES: &[&str] =
    &["float", "double", "string", "text", "json", "jsonb", "variant", "array", "map", "struct"];
/// Aggregate-state types can only be value columns of an AGGREGATE KEY table.
const DORIS_STATE_TYPES: &[&str] = &["bitmap", "hll", "quantile_state", "agg_state"];

/// The requested table model, upper-cased; defaults to DUPLICATE.
pub(super) fn doris_key_model(options: &TableStructureSqlOptions) -> String {
    let model =
        options.doris_table.as_ref().map(|table| table.key_model.trim().to_ascii_uppercase()).unwrap_or_default();
    if model.is_empty() {
        return DORIS_DEFAULT_KEY_MODEL.to_string();
    }
    model
}

fn base_type(data_type: &str) -> String {
    data_type
        .trim()
        .split(|ch: char| ch == '(' || ch == '<' || ch.is_whitespace())
        .next()
        .unwrap_or("")
        .to_ascii_lowercase()
}

fn doris_aggregation_type(column: &EditableStructureColumn) -> Option<String> {
    column
        .extra
        .as_ref()?
        .doris_aggregation_type
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_ascii_uppercase)
}

pub(super) fn validate_doris_table(
    options: &TableStructureSqlOptions,
    active_columns: &[&EditableStructureColumn],
    warnings: &mut Vec<String>,
) {
    let key_model = doris_key_model(options);
    if !DORIS_KEY_MODELS.contains(&key_model.as_str()) {
        warnings.push(format!("Unknown Doris table model \"{key_model}\"; use DUPLICATE, UNIQUE or AGGREGATE."));
        return;
    }
    if let Some(0) = options.doris_table.as_ref().and_then(|table| table.replication_num) {
        warnings.push("Doris replication_num must be at least 1.".to_string());
    }
    let mut seen_value_column = false;
    for column in active_columns {
        if column.is_primary_key {
            if seen_value_column {
                warnings.push(format!(
                    "Doris key columns must be the leading columns; move key column \"{}\" above every non-key column.",
                    column.name
                ));
            }
            let base = base_type(&column.data_type);
            if DORIS_NON_KEY_TYPES.contains(&base.as_str()) || DORIS_STATE_TYPES.contains(&base.as_str()) {
                warnings.push(format!(
                    "Column \"{}\" of type {} cannot be a Doris key column.",
                    column.name,
                    column.data_type.trim()
                ));
            }
            continue;
        }
        seen_value_column = true;
        if key_model == "AGGREGATE" {
            match doris_aggregation_type(column) {
                Some(aggregation) if DORIS_AGGREGATION_TYPES.contains(&aggregation.as_str()) => {}
                Some(aggregation) => warnings
                    .push(format!("Unknown Doris aggregation type \"{aggregation}\" on column \"{}\".", column.name)),
                None => warnings.push(format!(
                    "Doris AGGREGATE KEY tables need an aggregation type for value column \"{}\".",
                    column.name
                )),
            }
        }
    }
    if key_model != DORIS_DEFAULT_KEY_MODEL && !active_columns.iter().any(|column| column.is_primary_key) {
        warnings.push(format!("Doris {key_model} KEY tables need at least one key column."));
    }
}

/// Column-level aggregation clause (`SUM`, `REPLACE`, ...) of a value column in
/// an AGGREGATE KEY table; `None` for key columns and the other models.
pub(super) fn doris_column_aggregation_clause(key_model: &str, column: &EditableStructureColumn) -> Option<String> {
    if key_model != "AGGREGATE" || column.is_primary_key {
        return None;
    }
    doris_aggregation_type(column)
}

/// Everything after the column list: engine, key model, comment, distribution
/// and properties. `key_columns` are already quoted and in column order.
pub(super) fn doris_table_tail(options: &TableStructureSqlOptions, key_model: &str, key_columns: &[String]) -> String {
    let mut lines = vec!["ENGINE=OLAP".to_string()];
    let keys = key_columns.join(", ");
    if !key_columns.is_empty() {
        lines.push(format!("{key_model} KEY({keys})"));
    }
    let table_comment = clean(options.table_comment.as_deref().unwrap_or(""));
    if !table_comment.is_empty() {
        lines.push(format!("COMMENT {}", quote_string(&table_comment)));
    }
    if key_columns.is_empty() {
        // Only the DUPLICATE model may be distributed randomly; the other
        // models are rejected by `validate_doris_table` without key columns.
        lines.push("DISTRIBUTED BY RANDOM BUCKETS AUTO".to_string());
    } else {
        lines.push(format!("DISTRIBUTED BY HASH({keys}) BUCKETS AUTO"));
    }
    let replication = options.doris_table.as_ref().and_then(|table| table.replication_num).filter(|value| *value > 0);
    if let Some(replication) = replication {
        lines.push(format!("PROPERTIES (\"replication_num\" = \"{replication}\")"));
    }
    lines.join("\n")
}
