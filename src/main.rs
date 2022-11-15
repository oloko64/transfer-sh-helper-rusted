mod arg_parser;
mod database;
mod utils;
use std::{
    io::{self, Write},
    process::exit,
};

use database::Database;
use owo_colors::OwoColorize;

fn execute_delete_by_id(database: &Database) {
    println!();
    if utils::output_data(
        &database
            .get_all_entries()
            .expect("Failed while trying to read all entries."),
        false,
    ) == 0
    {
        println!("No data to delete");
        exit(0);
    }
    println!();
    let mut id = String::new();
    print!("Enter the id of the entry you want to remove: ");
    io::stdout().flush().unwrap();
    io::stdin().read_line(&mut id).expect("Failed to read line");
    database.delete_entry(id.trim().parse::<i64>().expect("Failed to parse id"));
}

fn execute_list(del_links: bool, database: &Database) {
    println!();
    utils::output_data(
        &database
            .get_all_entries()
            .expect("Failed while trying to read all entries."),
        del_links,
    );
    println!();
}

fn execute_drop(database: &Database) {
    database
        .delete_database_file()
        .expect("Failed to delete database file.");
}

fn execute_transfer(path: &str, database: &Database) {
    match utils::get_file_size(path) {
        Ok(size) => {
            println!("File size: {}", size);
        }
        Err(err) => {
            eprintln!("{}", err);
            exit(1);
        }
    };
    let default_name = path.split('/').last().unwrap_or("Default Name");
    {
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
        println!("\nUploading... please wait\n");
        database.transfer_file(entry_name.trim(), path);
    }
    utils::output_data(
        &database
            .get_all_entries()
            .expect("Failed while trying to read all entries."),
        false,
    );
    println!();
}

fn run_app() {
    let database = Database::new();
    database.create_table().expect("Failed to create table.");
    let args = arg_parser::prepare_args();

    match args {
        arg_parser::AppOptions::List => execute_list(false, &database),
        arg_parser::AppOptions::ListDel => execute_list(true, &database),
        arg_parser::AppOptions::Delete => execute_delete_by_id(&database),
        arg_parser::AppOptions::Drop => execute_drop(&database),
        arg_parser::AppOptions::Transfer(path) => execute_transfer(&path, &database),
    }
}

fn main() {
    run_app();
}
