use crate::types::Table;

/// Compare source (reference) and target schemas, generate SQL to make target match source.
/// The output SQL can be applied to the target database.
pub fn generate_migration_sql(source: &[Table], target: &[Table]) -> String {
    let source_map: std::collections::HashMap<_, _> =
        source.iter().map(|t| (t.name.as_str(), t)).collect();
    let target_map: std::collections::HashMap<_, _> =
        target.iter().map(|t| (t.name.as_str(), t)).collect();

    let mut sql_parts = Vec::new();

    // Tables only in source -> CREATE TABLE
    for table in source {
        if !target_map.contains_key(table.name.as_str()) {
            if let Some(ref create_sql) = table.create_sql {
                sql_parts.push(format!("{}\n;", create_sql));
            } else {
                // Fallback: build CREATE TABLE from columns
                sql_parts.push(build_create_table(table));
            }
        }
    }

    // Tables in both -> diff columns
    for table in source {
        if let Some(target_table) = target_map.get(table.name.as_str()) {
            let alters = diff_columns(table, target_table);
            if !alters.is_empty() {
                let table_name = escape_ident(&table.name);
                for stmt in alters {
                    sql_parts.push(format!("ALTER TABLE {} {};\n", table_name, stmt));
                }
            }
        }
    }

    // Tables only in target -> DROP TABLE
    for table in target {
        if !source_map.contains_key(table.name.as_str()) {
            sql_parts.push(format!(
                "DROP TABLE IF EXISTS `{}`;\n",
                table.name.replace('`', "``")
            ));
        }
    }

    sql_parts.join("\n")
}

fn escape_ident(s: &str) -> String {
    format!("`{}`", s.replace('`', "``"))
}

fn build_create_table(table: &Table) -> String {
    let cols: Vec<String> = table.columns.iter().map(|c| c.to_sql()).collect();
    let table_name = escape_ident(&table.name);
    format!(
        "CREATE TABLE {} (\n  {}\n);",
        table_name,
        cols.join(",\n  ")
    )
}

fn diff_columns(source: &Table, target: &Table) -> Vec<String> {
    let source_cols: std::collections::HashMap<_, _> = source
        .columns
        .iter()
        .map(|c| (c.name.as_str(), c))
        .collect();
    let target_cols: std::collections::HashMap<_, _> = target
        .columns
        .iter()
        .map(|c| (c.name.as_str(), c))
        .collect();

    let mut alters = Vec::new();

    // Columns only in source -> ADD COLUMN
    for col in &source.columns {
        if !target_cols.contains_key(col.name.as_str()) {
            alters.push(format!("ADD COLUMN {}", col.to_sql()));
        }
    }

    // Columns in both but different -> MODIFY COLUMN
    for col in &source.columns {
        if let Some(tcol) = target_cols.get(col.name.as_str()) {
            if col != *tcol {
                alters.push(format!("MODIFY COLUMN {}", col.to_sql()));
            }
        }
    }

    // Columns only in target -> DROP COLUMN (reverse order to avoid position issues)
    let cols_to_drop: Vec<_> = target
        .columns
        .iter()
        .filter(|c| !source_cols.contains_key(c.name.as_str()))
        .map(|c| c.name.clone())
        .collect();
    for col_name in cols_to_drop.iter().rev() {
        alters.push(format!("DROP COLUMN `{}`", col_name.replace('`', "``")));
    }

    alters
}
