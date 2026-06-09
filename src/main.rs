use anyhow::Result;
mod config;
mod db;
use config::read_config;


#[tokio::main]

async fn main()->Result<()>
 {
    println!("migration tool starting...");
    let db_pool = create_pool().await?;
    println!("connected to database");
    let config = read_config("transform.toml")?;
    db::ensure_table_exists(&db_pool,&config.migration.table,&config.schema).await?;
    db::sync_columns(&db_pool,&config.migration.table,config.schema.clone()).await?;
    println!("config: {:#?}",config);
    
    Ok(())
}

pub async fn create_pool()->Result<sqlx::PgPool>
{

    dotenv::dotenv().ok();
    let database_url =std::env::var("DATABASE_URL")?;
    let pool = sqlx::postgres::PgPoolOptions::new()
    .max_connections(5)
    .connect(&database_url)
    .await?;
    Ok(pool)

}