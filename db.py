from urllib.parse import urlparse
from typing import Optional

import pymysql
import pymysql.cursors

from schema import Column, Table


def connect(url: str):
    parsed = urlparse(url)
    return pymysql.connect(
        host=parsed.hostname,
        port=parsed.port or 3306,
        user=parsed.username,
        password=parsed.password,
        database=parsed.path.lstrip("/"),
        cursorclass=pymysql.cursors.DictCursor,
    )


def fetch_schema(conn) -> list[Table]:
    with conn.cursor() as cursor:
        cursor.execute(
            """
            SELECT table_name, column_name, column_type, is_nullable,
                   column_default, extra, column_key
            FROM information_schema.columns
            WHERE table_schema = DATABASE()
            ORDER BY table_name, ordinal_position
            """
        )
        rows = cursor.fetchall()

    tables: dict[str, list[Column]] = {}
    for row in rows:
        table_name = row["table_name"] if "table_name" in row else row["TABLE_NAME"]
        col = Column(
            name=row.get("column_name") or row.get("COLUMN_NAME"),
            column_type=row.get("column_type") or row.get("COLUMN_TYPE"),
            is_nullable=(row.get("is_nullable") or row.get("IS_NULLABLE")) == "YES",
            column_default=row.get("column_default") or row.get("COLUMN_DEFAULT"),
            extra=(row.get("extra") or row.get("EXTRA")) or None,
            column_key=(row.get("column_key") or row.get("COLUMN_KEY")) or None,
        )
        # Normalize empty strings to None
        if col.extra == "":
            col.extra = None
        if col.column_key == "":
            col.column_key = None
        tables.setdefault(table_name, []).append(col)

    return [Table(name=name, columns=cols) for name, cols in tables.items()]


def fetch_create_table(conn, table_name: str) -> str:
    escaped = table_name.replace("`", "``")
    with conn.cursor() as cursor:
        cursor.execute(f"SHOW CREATE TABLE `{escaped}`")
        row = cursor.fetchone()
    # Second value in the row is the DDL
    values = list(row.values())
    return values[1]


def fetch_schema_with_create(conn) -> list[Table]:
    tables = fetch_schema(conn)
    for table in tables:
        try:
            table.create_sql = fetch_create_table(conn, table.name)
        except Exception:
            pass
    return tables
