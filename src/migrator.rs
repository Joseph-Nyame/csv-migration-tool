use crate::db;

pub async fn migrate_csv(
    pool: &sqlx::PgPool,
    file_path: &str,
    table:&str,
    columns:&[String],
    batch_size: usize,
    schema: &std::collections::HashMap<String,String>
)->anyhow::Result<()>
{
    let mut reader = csv::Reader::from_path(file_path)?;
    let headers = reader.headers()?.clone();
    println!("headers {:?}" ,headers);

   let mut batch:Vec<csv::StringRecord> = Vec::new();
   let mut batch_number =0;
   //reader.records gives you an iterator over rows
   let start= std::time::Instant::now();
   for result in reader.records(){
    let record = result?;
    batch.push(record);
    if batch.len() >= batch_size{
        batch_number +=1;
        println!("processing batch {} with {} rows" , batch_number , batch.len());
        insert_batch(pool,table,columns,&headers,&batch,schema,file_path).await?;
        batch.clear();
    }

   }

   //handle remaining rows
   if !batch.is_empty(){
    batch_number +=1;
    println!("Processing final batch {} with {} rows" ,batch_number , batch.len());
    insert_batch(pool,table,columns,&headers,&batch,schema,file_path).await?;
    batch.clear();
   }

   let duration = start.elapsed().as_millis() as i64;
   db::record_migration(pool,file_path,batch_number,"success",table,duration).await?;

    Ok(())

}





pub async fn insert_batch(
    pool:&sqlx::PgPool,
    table:&str,
    columns: &[String],
    headers: &csv::StringRecord,
    batch: &[csv::StringRecord],
    schema:  &std::collections::HashMap<String,String>,
    source_file: &str,
)->anyhow::Result<()>
{

    // build a map of header name → column index
    // so we can look up where each column lives in each row

    //eg 
    //     headers.iter()        → gives you each header string one by one
    // .enumerate()          → wraps each with its position: (0, "name"), (1, "email"), (2, "age")
    // .map(|(i, h)| (h, i)) → flips it to (header_string, index): ("name", 0), ("email", 1)
    // .collect()            → gathers into HashMap: {"name": 0, "email": 1, "age": 2}
    let header_map: std::collections::HashMap<&str, usize> = headers.iter()
    .enumerate()
    .map(|(i,h)| (h,i))
    .collect();

    // Include source_file in the column list
    let mut all_columns =vec!["source_file".to_string()];
    all_columns.extend(columns.iter().cloned());

    let column_list = all_columns.join(",");
    let sql_start = format!("INSERT INTO \"{}\" ({}) ", table , column_list);
    let mut builder = sqlx::QueryBuilder::new(sql_start);

    let mut tx = pool.begin().await?;
    builder.push_values(batch.iter(), |mut b,row|{

        //first bind source file
        b.push_bind(source_file.to_string());

        for col in columns{
            if let Some(&idx) = header_map.get(col.as_str()){
                let value = row.get(idx).unwrap_or("");
                match schema.get(col).map(|s| s.as_str()){
                    Some("INTEGER") =>{
                        let n: i64 =value.parse().unwrap_or(0);
                        b.push_bind(n);
                    }
                    _ =>{
                        b.push_bind(value.to_string());
                    }
                }
            }
        }
    });
    builder.build().execute(&mut *tx).await?;

    tx.commit().await?;
    Ok(())
}