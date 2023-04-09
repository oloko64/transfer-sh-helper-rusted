mod arg_parser;
mod database;
mod macros;
mod utils;
use std::{
    io::{self, Write},
    path::Path,
    process::exit,
};

use arg_parser::AppOptions;
use comprexor::{CompressionLevel, Compressor};
use database::Database;
use once_cell::sync::{Lazy, OnceCell};
use owo_colors::OwoColorize;
use tokio::sync::Mutex;

static DATABASE: Lazy<Mutex<Database>> = Lazy::new(|| Mutex::new(Database::new()));

static ARGS: OnceCell<AppOptions> = OnceCell::new();

async fn execute_delete_by_id() {
    println!();
    if utils::output_data(false) == 0 {
        println!("No data to delete");
        exit(0);
    }
    println!();
    let mut id = String::new();
    print!("Enter the id of the entry you want to remove: ");
    io::stdout().flush().unwrap();
    io::stdin().read_line(&mut id).expect("Failed to read line");

    let database = DATABASE
        .try_lock()
        .expect("Failed to acquire lock of database.");
    database
        .delete_entry(id.trim().parse::<i64>().expect("Failed to parse id"))
        .await;
}

fn execute_list(delete_links: bool) {
    println!();
    utils::output_data(delete_links);
    println!();
}

fn execute_drop() {
    let database = DATABASE
        .try_lock()
        .expect("Failed to acquire lock of database.");
    database
        .delete_database_file()
        .expect("Failed to delete database file.");
}

async fn execute_transfer_file<T>(path: T) -> Result<(), io::Error>
where
    T: AsRef<str>,
{
    match utils::get_file_size(path.as_ref()).await {
        Ok(size) => {
            println!("File size: {}", size.green());
        }
        Err(err) => {
            eprintln!("{err}");
            exit(1);
        }
    };

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
        let database = DATABASE
            .try_lock()
            .expect("Failed to acquire lock of database.");
        database
            .transfer_file(entry_name.trim(), path.as_ref())
            .await;
    }

    utils::output_data(false);
    println!();

    Ok(())
}

async fn execute_transfer_compressed<T>(
    path: T,
    compression_level: &CompressionLevel,
) -> Result<(), io::Error>
where
    T: AsRef<str>,
{
    let compressed_path = format!("{}.tar.gz", path.as_ref());
    let compressor = Compressor::new(path.as_ref(), &compressed_path);
    println!(
        "Compressing {} with level {}...",
        path.as_ref().green(),
        u32::from(compression_level)
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
        let database = DATABASE
            .try_lock()
            .expect("Failed to acquire lock of database.");
        database
            .transfer_file(entry_name.trim(), &compressed_path)
            .await;
    }

    utils::output_data(false);
    println!();

    Ok(())
}

async fn run_app() {
    {
        let database = DATABASE
            .try_lock()
            .expect("Failed to acquire lock of database.");
        database.create_table().expect("Failed to create table.");
    }

    let args = ARGS.get().expect("Failed to get ARGS static variable.");

    match args {
        AppOptions::List { list_del } => execute_list(*list_del),
        AppOptions::Delete => execute_delete_by_id().await,
        AppOptions::Drop => execute_drop(),
        AppOptions::TransferFile(path) => execute_transfer_file(path).await.unwrap(),
        AppOptions::TransferCompressed(path, compression_level) => {
            execute_transfer_compressed(path, compression_level)
                .await
                .unwrap();
        }
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let Ok(_) = ARGS.set(AppOptions::init()) else {
        panic!("Failed to set ARGS static variable.")
    };

    run_app().await;
}
