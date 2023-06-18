mod arg_parser;
mod database;
mod macros;
mod utils;
use std::{
    io::{self, Write},
    path::Path,
    process::exit,
};

use arg_parser::{AppArguments, AppOptions};
use clap::Parser;
use comprexor::{CompressionLevel, Compressor};
use database::Database;
use once_cell::sync::Lazy;
use owo_colors::OwoColorize;
use reqwest::StatusCode;
use tokio::sync::Mutex;
use utils::transfer_response_code;

static DATABASE: Lazy<Mutex<Database>> = Lazy::new(|| Mutex::new(Database::new().unwrap()));

async fn execute_delete_by_id() -> Result<(), Box<dyn std::error::Error>> {
    verify_transfer_connection().await;
    println!();
    if utils::output_data(false)? == 0 {
        println!("No data to delete");
        exit(0);
    }
    println!();
    let mut id = String::new();
    print!("Enter the id of the entry you want to remove: ");
    io::stdout().flush()?;
    io::stdin().read_line(&mut id)?;

    let database = DATABASE.try_lock()?;
    database.delete_entry(id.trim().parse::<i64>()?).await?;

    Ok(())
}

fn execute_list(delete_links: bool) -> Result<(), Box<dyn std::error::Error>> {
    println!();
    utils::output_data(delete_links)?;
    println!();

    Ok(())
}

fn execute_drop() -> Result<(), Box<dyn std::error::Error>> {
    let database = DATABASE.try_lock()?;
    database.delete_database_file()?;

    Ok(())
}

async fn execute_transfer_file<T>(path: T) -> Result<(), Box<dyn std::error::Error>>
where
    T: AsRef<str>,
{
    match utils::get_file_size(path.as_ref()).await {
        Ok(size) => {
            println!("File size: {}", size.green());
        }
        Err(err) => {
            return Err(err);
        }
    };

    verify_transfer_connection().await;

    {
        let default_name = Path::new(path.as_ref())
            .file_name()
            .ok_or(io::Error::new(
                io::ErrorKind::Other,
                "Failed to get file name",
            ))?
            .to_str()
            .unwrap_or("default-name");
        let mut entry_name = String::new();
        print!(
            "\nEnter the name of the entry (Default name: {}): ",
            default_name.green()
        );
        io::stdout().flush()?;
        io::stdin().read_line(&mut entry_name)?;
        if entry_name.trim().is_empty() {
            entry_name = default_name.to_string();
        }
        println!();
        let database = DATABASE.try_lock()?;
        database
            .transfer_file(entry_name.trim(), path.as_ref())
            .await?;
    }

    utils::output_data(false)?;
    println!();

    Ok(())
}

async fn execute_transfer_compressed<T>(
    path: T,
    compression_level: &CompressionLevel,
) -> Result<(), Box<dyn std::error::Error>>
where
    T: AsRef<str>,
{
    let compressed_path = format!("{}.tar.gz", path.as_ref());
    let compressor = Compressor::new(path.as_ref(), &compressed_path);
    println!(
        "Compressing {} with compression level {}...\n",
        path.as_ref().green(),
        u32::from(compression_level).green()
    );
    let compress_info = compressor.compress(compression_level)?;

    println!(
        "Compressed {} to {}",
        path.as_ref().green(),
        compressed_path.green()
    );
    println!(
        "Compression ratio: {}",
        compress_info.ratio_formatted(2).green()
    );

    match utils::get_file_size(&compressed_path).await {
        Ok(size) => {
            println!("File size: {}", size.green());
        }
        Err(err) => {
            eprintln!("{err}");
            exit(1);
        }
    };

    verify_transfer_connection().await;

    {
        let default_name = Path::new(&compressed_path)
            .file_name()
            .ok_or(io::Error::new(
                io::ErrorKind::Other,
                "Failed to get file name",
            ))?
            .to_str()
            .unwrap_or("default-name");
        let mut entry_name = String::new();
        print!(
            "\nEnter the name of the entry (Default name: {}): ",
            default_name.green()
        );
        io::stdout().flush()?;
        io::stdin().read_line(&mut entry_name)?;
        if entry_name.trim().is_empty() {
            entry_name = default_name.to_string();
        }
        println!();
        let database = DATABASE.try_lock()?;
        database
            .transfer_file(entry_name.trim(), &compressed_path)
            .await?;
    }

    utils::output_data(false)?;
    println!();

    Ok(())
}

async fn verify_transfer_connection() {
    match transfer_response_code().await {
        Ok(StatusCode::OK) => {}
        Ok(code) => {
            eprintln!("Transfer.sh is not reachable, status code: {}", code.red());
            exit(1);
        }
        Err(err) => {
            eprintln!("Transfer.sh is not reachable: {}", err.red());
            exit(1);
        }
    }
}

async fn run_app(args: AppArguments) -> Result<(), Box<dyn std::error::Error>> {
    {
        let database = DATABASE.try_lock()?;
        database.create_table()?;
    }
    let Some(subcommands) = args.app_subcommands else {
        execute_list(false)?;
        exit(0);
    };

    match subcommands {
        AppOptions::List { delete_link } => execute_list(delete_link)?,
        AppOptions::Delete => execute_delete_by_id().await?,
        AppOptions::Drop => execute_drop()?,
        AppOptions::Upload {
            path,
            compress,
            level,
        } => {
            if compress {
                execute_transfer_compressed(path, &level).await?;
            } else {
                execute_transfer_file(path).await?;
            }
        }
    }

    Ok(())
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = arg_parser::AppArguments::parse();

    run_app(args).await?;

    Ok(())
}
