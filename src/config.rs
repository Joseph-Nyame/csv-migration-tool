use std::collections::HashMap;
use serde::Deserialize;


pub fn read_config(file_path: &str)->anyhow::Result<Config>{
    let content = std::fs::read_to_string(file_path)?;
    let config:Config = toml::from_str(&content)?;
    Ok(config)
}

#[derive(Debug ,Deserialize)]

pub struct Config{

    pub columns:Columns,
    pub filters: HashMap<String,String>,
    pub schema: HashMap<String,String>,
    pub migration: Migration
}

#[derive(Debug,Deserialize)]
pub struct Columns{
    pub keep: Vec<String>
}

#[derive(Debug,Deserialize)]
pub struct Migration{
    pub table:String,
    pub batch_size: usize,
}