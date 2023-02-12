mod arg_parser;
mod database;
mod macros;
mod utils;
use std::{
    io::{self, Write},
    process::exit,
};

use arg_parser::AppOptions;
use database::Database;
use once_cell::sync::{Lazy, OnceCell};
use owo_colors::OwoColorize;
use tokio::sync::Mutex;

static DATABASE: Lazy<Mutex<Database>> = Lazy::new(|| {
    let database = Database::new();
    Mutex::new(database)
});

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

async fn execute_transfer(path: &str) {
    match utils::get_file_size(path) {
        Ok(size) => {
            println!("File size: {size}");
        }
        Err(err) => {
            eprintln!("{err}");
            exit(1);
        }
    };

    {
        let default_name = path.split('/').last().unwrap_or("Default Name");
        let mut entry_name = String::new();
        print!(
            "\nEnter the name of the entry (Default name: {}): ",
            default_name.green()
        );
        io::stdout().flush().unwrap();
        io::stdin()
            .read_line(&mut entry_name)
            .expect("Failed to read line");
        if entry_name.trim().is_empty() {
            entry_name = default_name.to_string();
        }
        println!();
        let database = DATABASE
            .try_lock()
            .expect("Failed to acquire lock of database.");
        database.transfer_file(entry_name.trim(), path).await;
    }

    utils::output_data(false);
    println!();
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
        AppOptions::List => execute_list(false),
        AppOptions::ListDel => execute_list(true),
        AppOptions::Delete => execute_delete_by_id().await,
        AppOptions::Drop => execute_drop(),
        AppOptions::Transfer(path) => execute_transfer(path).await,
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let Ok(_) = ARGS.set(AppOptions::init()) else {
        panic!("Failed to set ARGS static variable.")
    };

    run_app().await;
}
