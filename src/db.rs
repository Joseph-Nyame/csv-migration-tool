
pub async fn ensure_table_exists(
    pool:&sqlx::PgPool,
    table:&str,
    schema:&std::collections::HashMap<String,String>
)->anyhow::Result<()>
{
    let column_defs=schema.iter().map(|(name,dtype)| {
        format!("\"{}\" {}",name,dtype)
    }).collect::<Vec<String>>().join(",");

    let sql = format!("
    CREATE TABLE IF NOT EXISTS \"{table}\" (
        id SERIAL PRIMARY KEY,
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