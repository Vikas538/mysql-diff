use sqlx::mysql::MySqlPool;
use sqlx::Row;

use crate::types::{Column, Table};

/// Fetch full schema (tables and columns) from a database
pub async fn fetch_schema(pool: &MySqlPool) -> Result<Vec<Table>, sqlx::Error> {
    let rows = sqlx::query(
        r#"
        SELECT table_name, column_name, column_type, is_nullable, column_default, extra, column_key
        FROM information_schema.columns
        WHERE table_schema = DATABASE()
        ORDER BY table_name, ordinal_position
        "#,
    )
    .fetch_all(pool)
    .await?;

    // Group rows by table
    let mut tables: std::collections::HashMap<String, Vec<Column>> =
        std::collections::HashMap::new();

    for row in rows {
        let table_name: String = row.try_get("table_name")?;
        let column_name: String = row.try_get("column_name")?;
        let column_type: String = row.try_get("column_type")?;
        let is_nullable: String = row.try_get("is_nullable")?;
        let column_default: Option<String> = row.try_get("column_default").ok();
        let extra: Option<String> = row.try_get("extra").ok();
        let column_key: Option<String> = row.try_get("column_key").ok();

        let column = Column {
            name: column_name,
            column_type,
            is_nullable: is_nullable == "YES",
            column_default: column_default.filter(|s| !s.is_empty()),
            extra: extra.filter(|s| !s.is_empty()),
            column_key: column_key.filter(|s| !s.is_empty()),
        };

        tables.entry(table_name).or_default().push(column);
    }

    let result: Vec<Table> = tables
        .into_iter()
        .map(|(name, columns)| Table {
            name,
            columns,
            create_sql: None,
        })
        .collect();

    Ok(result)
}

/// Fetch SHOW CREATE TABLE for a table - returns the full DDL
pub async fn fetch_create_table(pool: &MySqlPool, table_name: &str) -> Result<String, sqlx::Error> {
    let escaped = table_name.replace('`', "``");
    let rows = sqlx::query(&format!("SHOW CREATE TABLE `{}`", escaped))
        .fetch_one(pool)
        .await?;
    // Second column is the Create Table DDL
    let create_sql: String = rows.try_get(1)?;
    Ok(create_sql)
}

/// Fetch schema with full CREATE TABLE for each table (for new tables)
pub async fn fetch_schema_with_create(pool: &MySqlPool) -> Result<Vec<Table>, sqlx::Error> {
    let mut tables = fetch_schema(pool).await?;
    for table in &mut tables {
        if let Ok(create_sql) = fetch_create_table(pool, &table.name).await {
            table.create_sql = Some(create_sql);
        }
    }
    Ok(tables)
}
