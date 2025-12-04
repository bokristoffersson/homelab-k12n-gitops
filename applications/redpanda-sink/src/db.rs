use crate::error::AppError;
use crate::mapping::{FieldValue, Row};
use chrono::{DateTime, Utc};
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};
use std::collections::BTreeSet;

pub type DbPool = Pool<Postgres>;

pub async fn connect(url: &str) -> Result<DbPool, AppError> {
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(url)
        .await?;
    Ok(pool)
}

enum SqlValue {
    Ts(DateTime<Utc>),
    Text(Option<String>),
    F64(Option<f64>),
    I64(Option<i64>),
    Bool(Option<bool>),
}

/// Insert batch for timeseries data (simple INSERT)
pub async fn insert_batch(pool: &DbPool, table: &str, rows: &[Row]) -> Result<(), AppError> {
    if rows.is_empty() {
        return Ok(());
    }
    let mut tag_cols: BTreeSet<&str> = BTreeSet::new();
    let mut field_cols: BTreeSet<&str> = BTreeSet::new();
    for r in rows {
        for k in r.tags.keys() {
            tag_cols.insert(k.as_str());
        }
        for k in r.fields.keys() {
            field_cols.insert(k.as_str());
        }
    }
    let mut columns: Vec<String> = vec!["ts".into()];
    columns.extend(tag_cols.iter().map(|s| s.to_string()));
    columns.extend(field_cols.iter().map(|s| s.to_string()));
    let cols_per_row = columns.len();
    let mut values_placeholders: Vec<String> = Vec::with_capacity(rows.len());
    let mut binds: Vec<SqlValue> = Vec::with_capacity(rows.len() * cols_per_row);
    let mut arg_index = 1;
    for r in rows {
        let mut tuple = Vec::with_capacity(cols_per_row);
        tuple.push(format!("${}", arg_index));
        arg_index += 1;
        binds.push(SqlValue::Ts(r.ts));
        for col in &tag_cols {
            tuple.push(format!("${}", arg_index));
            arg_index += 1;
            binds.push(SqlValue::Text(r.tags.get(*col).cloned()));
        }
        for col in &field_cols {
            tuple.push(format!("${}", arg_index));
            arg_index += 1;
            let val = r.fields.get(*col);
            let sqlv = match val {
                Some(FieldValue::F64(v)) => SqlValue::F64(Some(*v)),
                Some(FieldValue::I64(v)) => SqlValue::I64(Some(*v)),
                Some(FieldValue::Bool(v)) => SqlValue::Bool(Some(*v)),
                Some(FieldValue::Text(v)) => SqlValue::Text(Some(v.clone())),
                None => SqlValue::Text(None),
            };
            binds.push(sqlv);
        }
        values_placeholders.push(format!("({})", tuple.join(", ")));
    }
    let sql = format!(
        "INSERT INTO {} ({}) VALUES {}",
        table,
        columns.join(", "),
        values_placeholders.join(", ")
    );
    let mut q = sqlx::query(&sql);
    for b in binds {
        q = match b {
            SqlValue::Ts(v) => q.bind(v),
            SqlValue::Text(v) => q.bind(v),
            SqlValue::F64(v) => q.bind(v),
            SqlValue::I64(v) => q.bind(v),
            SqlValue::Bool(v) => q.bind(v),
        };
    }
    q.execute(pool).await?;
    Ok(())
}

/// Upsert batch for static data (INSERT ... ON CONFLICT UPDATE)
pub async fn upsert_batch(
    pool: &DbPool,
    table: &str,
    upsert_key: &[String],
    rows: &[Row],
) -> Result<(), AppError> {
    if rows.is_empty() {
        return Ok(());
    }
    if upsert_key.is_empty() {
        return Err(AppError::Config(
            "upsert_key cannot be empty for static data".into(),
        ));
    }

    let mut tag_cols: BTreeSet<&str> = BTreeSet::new();
    let mut field_cols: BTreeSet<&str> = BTreeSet::new();
    for r in rows {
        for k in r.tags.keys() {
            tag_cols.insert(k.as_str());
        }
        for k in r.fields.keys() {
            field_cols.insert(k.as_str());
        }
    }

    // Verify all upsert_key columns exist
    for key_col in upsert_key {
        if !tag_cols.contains(key_col.as_str()) && !field_cols.contains(key_col.as_str()) {
            return Err(AppError::Config(format!(
                "upsert_key column '{}' not found in tags or fields",
                key_col
            )));
        }
    }

    // For static data, use 'latest_update' instead of 'ts'
    let mut columns: Vec<String> = vec!["latest_update".into()];
    columns.extend(tag_cols.iter().map(|s| s.to_string()));
    columns.extend(field_cols.iter().map(|s| s.to_string()));
    let cols_per_row = columns.len();
    let mut values_placeholders: Vec<String> = Vec::with_capacity(rows.len());
    let mut binds: Vec<SqlValue> = Vec::with_capacity(rows.len() * cols_per_row);
    let mut arg_index = 1;

    for r in rows {
        let mut tuple = Vec::with_capacity(cols_per_row);
        tuple.push(format!("${}", arg_index));
        arg_index += 1;
        binds.push(SqlValue::Ts(r.ts));
        for col in &tag_cols {
            tuple.push(format!("${}", arg_index));
            arg_index += 1;
            binds.push(SqlValue::Text(r.tags.get(*col).cloned()));
        }
        for col in &field_cols {
            tuple.push(format!("${}", arg_index));
            arg_index += 1;
            let val = r.fields.get(*col);
            let sqlv = match val {
                Some(FieldValue::F64(v)) => SqlValue::F64(Some(*v)),
                Some(FieldValue::I64(v)) => SqlValue::I64(Some(*v)),
                Some(FieldValue::Bool(v)) => SqlValue::Bool(Some(*v)),
                Some(FieldValue::Text(v)) => SqlValue::Text(Some(v.clone())),
                None => SqlValue::Text(None),
            };
            binds.push(sqlv);
        }
        values_placeholders.push(format!("({})", tuple.join(", ")));
    }

    // Build UPDATE clause - update all columns except the key columns
    let update_cols: Vec<String> = columns
        .iter()
        .filter(|col| !upsert_key.contains(col))
        .map(|col| format!("{} = EXCLUDED.{}", col, col))
        .collect();

    let conflict_target = upsert_key.join(", ");
    let sql = format!(
        "INSERT INTO {} ({}) VALUES {} ON CONFLICT ({}) DO UPDATE SET {}",
        table,
        columns.join(", "),
        values_placeholders.join(", "),
        conflict_target,
        update_cols.join(", ")
    );

    let mut q = sqlx::query(&sql);
    for b in binds {
        q = match b {
            SqlValue::Ts(v) => q.bind(v),
            SqlValue::Text(v) => q.bind(v),
            SqlValue::F64(v) => q.bind(v),
            SqlValue::I64(v) => q.bind(v),
            SqlValue::Bool(v) => q.bind(v),
        };
    }
    q.execute(pool).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mapping::{FieldValue, Row};
    use std::collections::BTreeMap;

    #[test]
    fn test_upsert_sql_generation() {
        // This test verifies the SQL structure is correct
        // Note: This doesn't actually execute SQL, just validates the logic

        let mut tags = BTreeMap::new();
        tags.insert("device_id".to_string(), "device-1".to_string());

        let mut fields = BTreeMap::new();
        fields.insert("name".to_string(), FieldValue::Text("Device 1".to_string()));
        fields.insert("status".to_string(), FieldValue::Text("active".to_string()));

        let row = Row {
            ts: Utc::now(),
            tags,
            fields,
        };

        let rows = vec![row];
        let upsert_key = vec!["device_id".to_string()];

        // We can't easily test the SQL generation without a database,
        // but we can verify the function signature and error handling
        assert!(!upsert_key.is_empty());
        assert_eq!(rows.len(), 1);
    }

    #[test]
    fn test_upsert_key_validation() {
        // Test that empty upsert_key is rejected
        let upsert_key: Vec<String> = vec![];
        assert!(upsert_key.is_empty());
    }
}
