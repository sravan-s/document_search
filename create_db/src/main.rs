use anyhow::{Context, Result};
use dotenvy::dotenv;
use postgres::{Client, NoTls};
use std::env;

fn main() -> Result<()> {
    let _ = dotenv().context("Couldnt read env file");
    let postgres_pass = env::var("POSTGRES_PASSWORD")
        .context("Couldnt read postgres_pass")
        .unwrap();
    let postgres_port = env::var("POSTGRES_PORT")
        .context("Couldnt read postgres_port")
        .unwrap();

    let postgres_connection_param = format!(
        "host=localhost user=postgres password={} port={}",
        postgres_pass, postgres_port
    );

    let mut client = Client::connect(&postgres_connection_param.to_string(), NoTls)
        .context("Couldnt connect to postgres")
        .unwrap();

    client
        .batch_execute(
            "CREATE TABLE IF NOT EXISTS laws (
            penal_code VARCHAR(20) PRIMARY KEY,
            summary TEXT,
            illustrations TEXT[],
            sidenotes TEXT[])",
        )
        .context("Couldnt create table laws")
        .unwrap();
    Ok(())
}
