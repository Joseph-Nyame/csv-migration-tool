# CSV to PostgresSQL Importer
A command-line tool that takes a CSV file and a TOML configuration file to perform boring, repetitive database inserts. Define your table schema, batch size, and let the tool handle the rest using your environment configuration.
## Features
- **Migration**: Create tables based on schema definition
- **Batch Inserts**: Insert data in configurable batch sizes for optimal performance
- **Rollback**: Complete rollback functionality (coming soon with CLI args)
## Configuration
Create a TOML configuration file (e.g., `config.toml`) with the following structure:
```toml
[schema]
name = "TEXT"
email = "TEXT"
age = "INTEGER"
status = "TEXT"
phone = "TEXT"
[migration]
table = "members"
batch_size = 10000

Current Usage
bash

```
cargo run
```

Note: Currently, the tool reads a hardcoded configuration file. CLI argument support is coming soon.
Planned Usage (Coming Soon)
bash

```
cargo run -- --csv data.csv --config config.toml
```

Environment Configuration
The tool uses your environment configuration for database connections. Make sure your environment variables are properly set before running.

eg: 
DATABASE_URL = "postgresql://csv_user:password@localhost:5432/csv_migrations"

Requirements

* Rust (latest stable)
* Environment variables configured for database access
* CSV file matching the schema structure
* TOML configuration file