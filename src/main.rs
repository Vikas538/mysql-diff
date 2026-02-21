mod db;
mod diff;
mod types;

use std::env;
use std::fs::File;
use std::io::Write;
use std::path::Path;

use sqlx::mysql::MySqlPool;

use db::{fetch_schema, fetch_schema_with_create};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() == 2 {
        match args[1].as_str() {
            "-h" | "--help" => {
                eprintln!("MySQL Schema Diff - Compare two databases and generate migration SQL");
                eprintln!();
                eprintln!("Usage: {} <source_url> <target_url> <output.sql>", args[0]);
                eprintln!();
                eprintln!("Arguments:");
                eprintln!("  source_url   MySQL connection URL for reference schema (e.g. mysql://user:pass@host/db)");
                eprintln!("  target_url   MySQL connection URL for database to migrate");
                eprintln!("  output.sql   Path for the generated SQL migration file");
                eprintln!();
                eprintln!("The generated SQL makes target match source: adds new tables/columns,");
                eprintln!("modifies changed columns, drops removed tables/columns.");
                return Ok(());
            }
            "-v" | "--version" => {
                println!("mysql-diff {}", env!("CARGO_PKG_VERSION"));
                return Ok(());
            }
            _ => {}
        }
    }
    if args.len() != 4 {
        eprintln!("Usage: {} <source_url> <target_url> <output.sql>", args[0]);
        eprintln!("Use --help for more information");
        std::process::exit(1);
    }

    let source_url = &args[1];
    let target_url = &args[2];
    let output_path = &args[3];

    let runtime = tokio::runtime::Runtime::new()?;
    runtime.block_on(run(source_url, target_url, output_path))
}

async fn run(
    source_url: &str,
    target_url: &str,
    output_path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Connecting to source database...");
    let source_pool = MySqlPool::connect(source_url).await?;

    println!("Connecting to target database...");
    let target_pool = MySqlPool::connect(target_url).await?;

    println!("Fetching source schema...");
    let source_schema = fetch_schema_with_create(&source_pool).await?;
    println!("  Found {} tables", source_schema.len());

    println!("Fetching target schema...");
    let target_schema = fetch_schema(&target_pool).await?;
    println!("  Found {} tables", target_schema.len());

    let sql = diff::generate_migration_sql(&source_schema, &target_schema);

    let path = Path::new(output_path);
    let mut file = File::create(path)?;
    file.write_all(sql.as_bytes())?;

    println!("Migration SQL written to {}", output_path);

    source_pool.close().await;
    target_pool.close().await;

    Ok(())
}
