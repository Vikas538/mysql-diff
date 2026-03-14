# mysql-diff

Compare two MySQL databases and generate migration SQL. Makes target schema match source.

## Requirements

- Python 3.10+
- [PyMySQL](https://pypi.org/project/PyMySQL/)

```bash
pip install -r requirements.txt
```

## Usage

```bash
python __main__.py <source_url> <target_url> <output.sql>
```

- **source_url** – reference schema (e.g. `mysql://user:pass@host/db`)
- **target_url** – database to migrate
- **output.sql** – path for generated migration SQL

### Flags

| Flag | Description |
|------|-------------|
| `-h`, `--help` | Show help message and exit |
| `-v`, `--version` | Print version and exit |

### Examples

```bash
# Generate migration SQL
python __main__.py mysql://root:pass@localhost/db_v2 mysql://root:pass@localhost/db_v1 migration.sql

# Help
python __main__.py --help

# Version
python __main__.py --version
```

## Using in CI/CD (Schema Automation)

### Option 1: Docker (recommended)

Build an image in this repo, push to your registry, then use in your project's CI:

```bash
docker build -t mysql-diff .
docker run --rm -v $PWD:/out mysql-diff \
  "mysql://user:pass@host/source_db" "mysql://user:pass@host/target_db" /out/migration.sql
```

In a GitHub Actions pipeline:

```yaml
- name: Generate schema migration
  run: |
    docker run --rm \
      -v $PWD/migrations:/out \
      your-registry/mysql-diff:latest \
      "$MYSQL_SOURCE_URL" "$MYSQL_TARGET_URL" /out/migration.sql
```

### Option 2: Run from source

```bash
git clone https://github.com/YOUR_ORG/mysql-diff.git
cd mysql-diff
pip install -r requirements.txt
python __main__.py "$SOURCE_URL" "$TARGET_URL" migration.sql
```

### Option 3: GitHub Actions (in your project)

```yaml
jobs:
  schema-diff:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
        with:
          repository: YOUR_ORG/mysql-diff
          path: mysql-diff

      - run: pip install -r mysql-diff/requirements.txt

      - name: Generate migration
        env:
          SOURCE_URL: ${{ secrets.SOURCE_DB_URL }}
          TARGET_URL: ${{ secrets.TARGET_DB_URL }}
        run: |
          python mysql-diff/__main__.py "$SOURCE_URL" "$TARGET_URL" migrations/auto.sql
```

### Typical workflow

1. **Source** = canonical schema (e.g. dev, migrations applied).
2. **Target** = environment to update (e.g. staging, prod).
3. **Output** = migration SQL. Run it against target to bring it in sync with source.

Exit codes: `0` = success, non-zero = error (useful for CI gates).
