from schema import Table, Column


def generate_migration_sql(source: list[Table], target: list[Table]) -> str:
    source_map = {t.name: t for t in source}
    target_map = {t.name: t for t in target}

    sql_parts = []

    # Tables only in source -> CREATE TABLE
    for table in source:
        if table.name not in target_map:
            if table.create_sql:
                sql_parts.append(f"{table.create_sql}\n;")
            else:
                sql_parts.append(_build_create_table(table))

    # Tables in both -> diff columns -> ALTER TABLE
    for table in source:
        if table.name in target_map:
            alters = _diff_columns(table, target_map[table.name])
            if alters:
                table_name = _escape_ident(table.name)
                for stmt in alters:
                    sql_parts.append(f"ALTER TABLE {table_name} {stmt};\n")

    # Tables only in target -> DROP TABLE
    for table in target:
        if table.name not in source_map:
            escaped = table.name.replace("`", "``")
            sql_parts.append(f"DROP TABLE IF EXISTS `{escaped}`;\n")

    return "\n".join(sql_parts)


def _escape_ident(s: str) -> str:
    return f"`{s.replace('`', '``')}`"


def _build_create_table(table: Table) -> str:
    cols = ",\n  ".join(c.to_sql() for c in table.columns)
    return f"CREATE TABLE {_escape_ident(table.name)} (\n  {cols}\n);"


def _diff_columns(source: Table, target: Table) -> list[str]:
    source_cols = {c.name: c for c in source.columns}
    target_cols = {c.name: c for c in target.columns}

    alters = []

    # Columns only in source -> ADD COLUMN
    for col in source.columns:
        if col.name not in target_cols:
            alters.append(f"ADD COLUMN {col.to_sql()}")

    # Columns in both but different -> MODIFY COLUMN
    for col in source.columns:
        if col.name in target_cols and col != target_cols[col.name]:
            alters.append(f"MODIFY COLUMN {col.to_sql()}")

    # Columns only in target -> DROP COLUMN
    to_drop = [c for c in reversed(target.columns) if c.name not in source_cols]
    for col in to_drop:
        escaped = col.name.replace("`", "``")
        alters.append(f"DROP COLUMN `{escaped}`")

    return alters
