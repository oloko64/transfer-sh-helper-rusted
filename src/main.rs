mod arg_parser;
mod utils;
use std::{
    io::{self, Write},
    process::exit,
};

use utils::Database;
#[macro_use]
extern crate prettytable;

fn execute_delete_by_id(database: &Database) {
    println!();
    if utils::output_data(
        &utils::get_all_entries(database).expect("Failed while trying to read all entries."),
        false,
    ) <= 0
    {
        println!("No data to delete");
        exit(0);
    }
    println!();
    let mut id = String::new();
    print!("Enter the id of the entry you want to remove: ");
    io::stdout().flush().unwrap();
    io::stdin().read_line(&mut id).expect("Failed to read line");
    utils::delete_entry(
        id.trim().parse::<i64>().expect("Failed to parse id"),
        database,
    );
}

fn execute_list(del_links: bool, database: &Database) {
    println!();
    utils::output_data(
        &utils::get_all_entries(database).expect("Failed while trying to read all entries."),
        del_links,
    );
    println!();
}

fn execute_drop(database: &Database) {
    utils::delete_database_file(database).expect("Failed to delete database file.");
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
            default_name
        );
        io::stdout().flush().unwrap();
        io::stdin()
            .read_line(&mut entry_name)
            .expect("Failed to read line");
        if entry_name.trim().is_empty() {
            entry_name = default_name.to_string();
        }
        println!("\nUploading... please wait\n");
        utils::transfer_file(entry_name.trim(), path, database);
    }
    utils::output_data(
        &utils::get_all_entries(database).expect("Failed while trying to read all entries."),
        false,
    );
    println!();
}

fn run_app() {
    let database = Database::new();
    utils::create_config_app_folder().expect("Failed to create config folder.");
    utils::create_table(&database).expect("Failed to create table.");
    let args = arg_parser::prepare_args();
    if let Some(path) = args.upload_file {
        execute_transfer(&path, &database);
    } else if args.list {
        execute_list(false, &database);
    } else if args.list_del {
        execute_list(true, &database);
    } else if args.delete {
        execute_delete_by_id(&database);
    } else if args.drop {
        execute_drop(&database);
    } else {
        execute_list(false, &database);
    }
}

fn main() {
    run_app();
}
