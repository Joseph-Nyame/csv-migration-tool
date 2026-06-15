use config::read_config;
use migrator::migrate_csv;
use anyhow::Result;
mod config;
mod db;
mod migrator;




#[tokio::main]

async fn main()->Result<()>
 {
    println!("migration tool starting...");
    let db_pool = create_pool().await?;
    println!("connected to database");
    let config = read_config("transform.toml")?;
    db::ensure_table_exists(&db_pool, &config.migration.table,&config.schema).await?;
    db::sync_columns(&db_pool,&config.migration.table,config.schema.clone()).await?;
    db::ensure_migrations_table(&db_pool).await?;

    db::list_migrations(&db_pool).await?;

    let csv_file="test_large1.csv";
    if db::check_already_migrated(&db_pool,csv_file).await?{
        println!("Already migrated: {}",csv_file);

        println!("Do you want to rollback or remove the migration? (y/n): ");
        let mut answer = String::new();
        std::io::stdin().read_line(&mut answer).expect("failed to read line");
        let answer = answer.trim().to_lowercase();
        if answer == "y" || answer == "yes"{
            println!("Rolling back...");
            db::rollback_migration(&db_pool,csv_file).await?;
            println!("successfullly rolled back migration");
            return Ok(());
        }else{
            println!("Exiting...");
            return Ok(())
        }
    }
    migrate_csv(&db_pool,csv_file,&config.migration.table,&config.columns.keep,config.migration.batch_size,&config.schema).await?;
    
    
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