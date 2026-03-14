import sys

from db import connect, fetch_schema, fetch_schema_with_create
from diff import generate_migration_sql

VERSION = "0.1.0"
HELP = """\
MySQL Schema Diff - Compare two databases and generate migration SQL

Usage: python __main__.py <source_url> <target_url> <output.sql>

Arguments:
  source_url   MySQL connection URL for reference schema (e.g. mysql://user:pass@host/db)
  target_url   MySQL connection URL for database to migrate
  output.sql   Path for the generated SQL migration file

The generated SQL makes target match source: adds new tables/columns,
modifies changed columns, drops removed tables/columns."""


def main():
    args = sys.argv[1:]

    if len(args) == 1 and args[0] in ("-h", "--help"):
        print(HELP)
        return
    if len(args) == 1 and args[0] in ("-v", "--version"):
        print(f"mysql-diff {VERSION}")
        return

    if len(args) != 3:
        print("Usage: python __main__.py <source_url> <target_url> <output.sql>", file=sys.stderr)
        print("Use --help for more information", file=sys.stderr)
        sys.exit(1)

    source_url, target_url, output_path = args

    print("Connecting to source database...")
    source_conn = connect(source_url)

    print("Connecting to target database...")
    target_conn = connect(target_url)

    print("Fetching source schema...")
    source_schema = fetch_schema_with_create(source_conn)
    print(f"  Found {len(source_schema)} tables")

    print("Fetching target schema...")
    target_schema = fetch_schema(target_conn)
    print(f"  Found {len(target_schema)} tables")

    sql = generate_migration_sql(source_schema, target_schema)

    with open(output_path, "w") as f:
        f.write(sql)

    print(f"Migration SQL written to {output_path}")

    source_conn.close()
    target_conn.close()


if __name__ == "__main__":
    main()
