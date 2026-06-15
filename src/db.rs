
pub async fn ensure_table_exists(
    pool:&sqlx::PgPool,
    table:&str,
    schema:&std::collections::HashMap<String,String>,
)->anyhow::Result<()>
{
    let column_defs=schema.iter().map(|(name,dtype)| {
        format!("\"{}\" {}",name,dtype)
    }).collect::<Vec<String>>().join(",");




    let sql = format!("
    CREATE TABLE IF NOT EXISTS \"{table}\" (
        id SERIAL PRIMARY KEY,
        source_file TEXT NOT NULL,
        {column_defs}
    )");

    println!("SQL:{}",sql);

    sqlx::raw_sql(sqlx::AssertSqlSafe(sql)).execute(pool).await?;
    Ok(())
}



pub async fn sync_columns(
    pool:&sqlx::PgPool,
    table:&str,
    schema:std::collections::HashMap<String,String>
) ->anyhow::Result<()>{
    //get current columns from db 
    let existing_columns:Vec<String> = sqlx::query_scalar( "SELECT column_name  FROM information_schema.columns 
    WHERE table_name = $1 ORDER by ordinal_position"
    )
    .bind(table)
    .fetch_all(pool)
    .await?;
    println!("existing columns: {:?}",existing_columns);

    
    for(col, dtype) in &schema{
        if !existing_columns.contains(col){
            //this column is new run alt migration
            let sql = format!("ALTER TABLE \"{}\" ADD COLUMN \"{}\" {};",table,col,dtype);
            println!("Adding column: {}",sql);
            sqlx::raw_sql(sqlx::AssertSqlSafe(sql)).execute(pool).await?;
        }
    }

    Ok(())

}


pub async fn ensure_migrations_table(pool: &sqlx::PgPool) -> anyhow::Result<()> {
    let sql = "
        CREATE TABLE IF NOT EXISTS migrations (
            id SERIAL PRIMARY KEY,
            migration_name TEXT NOT NULL,
            table_name TEXT NOT NULL,
            row_count INTEGER NOT NULL,
            status TEXT NOT NULL,
            duration_ms INTEGER NOT NULL,
            created_at TIMESTAMP DEFAULT NOW()
        )
    ";
    sqlx::raw_sql(sql).execute(pool).await?;
    Ok(())
}



pub async fn record_migration(
    pool:&sqlx::PgPool,
    migration_name: &str,
    row_count: usize,
    status: &str,
    table_name:&str,
    duration_ms:i64
)-> anyhow::Result<()>
{
    sqlx::query("INSERT INTO migrations (migration_name,table_name,row_count,status,duration_ms)
    VALUES($1,$2,$3,$4,$5)
    ")
    .bind(migration_name)
    .bind(table_name)
    .bind(row_count as i64)
    .bind(status)
    .bind(duration_ms)
    .execute(pool)
    .await?;

    Ok(())
}

pub  async fn check_already_migrated(pool:&sqlx::PgPool,filename:&str)->anyhow::Result<bool>
{
    let count:i64=sqlx::query_scalar("
        SELECT COUNT(*) 
        FROM migrations 
        WHERE migration_name = $1 
        AND status = 'success'")
        .bind(filename)
        .fetch_one(pool)
        .await?;
    
    Ok(count>0)

} 

pub async fn rollback_migration(pool:&sqlx::PgPool,migration_name:&str)->anyhow::Result<()>{
   
    //get table name from migration table
    let table_name:String = sqlx::query_scalar("SELECT table_name FROM migrations 
    WHERE migration_name = $1")
    .bind(migration_name)
    .fetch_one(pool)
    .await?;

    //delete rows that were inserted by this migration
    sqlx::raw_sql(sqlx::AssertSqlSafe(format!(
        "DELETE FROM \"{}\" WHERE source_file = '{}'", 
        table_name, migration_name
    ))).execute(pool).await?;
    

    //update status in migration table
    sqlx::query("UPDATE migrations SET status = 'rolled_back'
    WHERE migration_name = $1 AND status = 'success'")
    .bind(migration_name)
    .execute(pool)
    .await?;
    
   println!("successfullly rolled back migration {}",migration_name);
   Ok(())
}



//will use later
pub async fn list_migrations(pool: &sqlx::PgPool) -> anyhow::Result<()> {
    let migrations: Vec<(String, String, i32, String)> = sqlx::query_as(
        "SELECT migration_name, status, row_count, created_at::text 
         FROM migrations ORDER BY created_at DESC"
    )
    .fetch_all(pool)
    .await?;
    
    println!("Recent migrations:");
    for (name, status, rows, created_at) in migrations {
        println!("  {} - {} ({} rows) - {}", name, status, rows, created_at);
    }
    
    Ok(())
}