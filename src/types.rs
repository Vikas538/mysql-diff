/// Represents a single column in a table with full MySQL definition
#[derive(Debug, Clone, PartialEq)]
pub struct Column {
    pub name: String,
    /// Full type e.g. "int(11)", "varchar(255)"
    pub column_type: String,
    pub is_nullable: bool,
    pub column_default: Option<String>,
    /// e.g. "auto_increment", "on update CURRENT_TIMESTAMP"
    pub extra: Option<String>,
    /// PRI, UNI, MUL, or empty
    pub column_key: Option<String>,
}

impl Column {
    /// Format column definition for use in CREATE/ALTER TABLE
    pub fn to_sql(&self) -> String {
        let mut parts = vec![format!("`{}`", self.name), self.column_type.clone()];
        if self.is_nullable {
            parts.push("NULL".to_string());
        } else {
            parts.push("NOT NULL".to_string());
        }
        if let Some(ref default) = self.column_default {
            let upper = default.to_uppercase();
            if upper.starts_with("CURRENT_TIMESTAMP") || upper == "NULL" || !default.contains('\'')
            {
                parts.push(format!("DEFAULT {}", default));
            } else {
                parts.push(format!("DEFAULT '{}'", default.replace('\'', "\\'")));
            }
        }
        if let Some(ref extra) = self.extra {
            parts.push(extra.to_uppercase().replace(" ", " "));
        }
        parts.join(" ")
    }
}

#[derive(Debug, Clone)]
pub struct Table {
    pub name: String,
    pub columns: Vec<Column>,
    /// Full CREATE TABLE statement from source (for new tables)
    pub create_sql: Option<String>,
}
