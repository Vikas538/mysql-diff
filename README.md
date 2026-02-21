# mysql-diff

Compare two MySQL databases and generate migration SQL. Makes target schema match source.

## Usage

```
mysql-diff <source_url> <target_url> <output.sql>
```

- **source_url** – reference schema (e.g. `mysql://user:pass@host/db`)
- **target_url** – database to migrate
- **output.sql** – path for generated migration SQL

## Using in CI/CD (Schema Automation)

Use mysql-diff in another project's pipeline to automate schema diffs and migration generation.

### Option 1: Docker (recommended)

Build an image in this repo, push to your registry, then use in your project's CI:

```yaml
# In your project's CI (e.g. GitHub Actions)
- name: Generate schema migration
  run: |
    docker run --rm \
      -e MYSQL_SOURCE_URL -e MYSQL_TARGET_URL \
      -v $PWD/migrations:/out \
      your-registry/mysql-diff:latest \
      "$MYSQL_SOURCE_URL" "$MYSQL_TARGET_URL" /out/migration.sql
```

Or use the image directly from this repo (once published):

```yaml
- name: Schema diff
  run: |
    docker build -t mysql-diff https://github.com/YOUR_ORG/mysql-diff.git
    docker run --rm -v $PWD:/out mysql-diff \
      "$SOURCE_DB_URL" "$TARGET_DB_URL" /out/migration.sql
```

### Option 2: Build from source

If your CI has Rust:

```bash
git clone https://github.com/YOUR_ORG/mysql-diff.git
cd mysql-diff && cargo build --release
./target/release/mysql-diff "$SOURCE_URL" "$TARGET_URL" migration.sql
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

      - uses: dtolnay/rust-toolchain@stable

      - run: cargo build --release --manifest-path mysql-diff/Cargo.toml

      - name: Generate migration
        env:
          SOURCE_URL: ${{ secrets.SOURCE_DB_URL }}
          TARGET_URL: ${{ secrets.TARGET_DB_URL }}
        run: |
          ./mysql-diff/target/release/mysql-diff "$SOURCE_URL" "$TARGET_URL" migrations/auto.sql
```

### Typical workflow

1. **Source** = canonical schema (e.g. dev, migrations applied).
2. **Target** = environment to update (e.g. staging, prod).
3. **Output** = migration SQL. Run it against target to bring it in sync with source.

Exit codes: `0` = success, non-zero = error (useful for CI gates).
