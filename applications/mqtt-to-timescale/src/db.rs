use crate::error::AppError;
use crate::mapping::{FieldValue, Row};
use chrono::{DateTime, Utc};
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};
use std::collections::BTreeSet;

pub type DbPool = Pool<Postgres>;

pub async fn connect(url: &str) -> Result<DbPool, AppError> {
    let pool = PgPoolOptions::new().max_connections(10).connect(url).await?;
    Ok(pool)
}

enum SqlValue { Ts(DateTime<Utc>), Text(Option<String>), F64(Option<f64>), I64(Option<i64>), Bool(Option<bool>), }

pub async fn insert_batch(pool: &DbPool, table: &str, rows: &[Row]) -> Result<(), AppError> {
    if rows.is_empty() { return Ok(()); }
    let mut tag_cols: BTreeSet<&str> = BTreeSet::new();
    let mut field_cols: BTreeSet<&str> = BTreeSet::new();
    for r in rows { for k in r.tags.keys() { tag_cols.insert(k.as_str()); } for k in r.fields.keys() { field_cols.insert(k.as_str()); } }
    let mut columns: Vec<String> = vec!["ts".into()];
    columns.extend(tag_cols.iter().map(|s| s.to_string()));
    columns.extend(field_cols.iter().map(|s| s.to_string()));
    let cols_per_row = columns.len();
    let mut values_placeholders: Vec<String> = Vec::with_capacity(rows.len());
    let mut binds: Vec<SqlValue> = Vec::with_capacity(rows.len() * cols_per_row);
    let mut arg_index = 1;
    for r in rows {
        let mut tuple = Vec::with_capacity(cols_per_row);
        tuple.push(format!("${}", arg_index)); arg_index += 1; binds.push(SqlValue::Ts(r.ts));
        for col in &tag_cols { tuple.push(format!("${}", arg_index)); arg_index += 1; binds.push(SqlValue::Text(r.tags.get(*col).cloned())); }
        for col in &field_cols { tuple.push(format!("${}", arg_index)); arg_index += 1; let val = r.fields.get(*col); let sqlv = match val { Some(FieldValue::F64(v)) => SqlValue::F64(Some(*v)), Some(FieldValue::I64(v)) => SqlValue::I64(Some(*v)), Some(FieldValue::Bool(v)) => SqlValue::Bool(Some(*v)), Some(FieldValue::Text(v)) => SqlValue::Text(Some(v.clone())), None => SqlValue::Text(None), }; binds.push(sqlv); }
        values_placeholders.push(format!("({})", tuple.join(", ")));
    }
    let sql = format!("INSERT INTO {} ({}) VALUES {}", table, columns.join(", "), values_placeholders.join(", "));
    let mut q = sqlx::query(&sql);
    for b in binds { q = match b { SqlValue::Ts(v) => q.bind(v), SqlValue::Text(v) => q.bind(v), SqlValue::F64(v) => q.bind(v), SqlValue::I64(v) => q.bind(v), SqlValue::Bool(v) => q.bind(v), }; }
    q.execute(pool).await?; Ok(())
}
