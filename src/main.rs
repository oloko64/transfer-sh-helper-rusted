mod utils;
use std::{env, io::{self, Write}, process::exit, path::Path};
#[macro_use] extern crate prettytable;

fn execute_version() {
    println!("\nVersion => v0.1.4\n");
}

fn execute_help() {
    println!("
    Usage:
    -h, --help                 Prints help
    -v, --version              Prints version
    -u, --upload [FILE_PATH]   Upload a new link
    -l, --list                 Lists all links
    -L, --listdel              Lists all delete links
    -d, --delete               Deletes a specific link
    -D, --drop                 Deletes database file
    ");
}

fn execute_delete_by_id() {
    println!();
    if utils::output_data(utils::get_all_entries(), false) <= 0 {
        println!("No data to delete");
        exit(0);
    }
    println!();
    let mut id = String::new();
    print!("Enter the id of the entry you want to remove: ");
    io::stdout().flush().unwrap();
    io::stdin().read_line(&mut id).expect("Failed to read line");
    utils::delete_entry(id.trim().parse::<i64>().expect("Failed to parse id"))
}

fn execute_list(del_links: bool) {
    println!();
    utils::output_data(utils::get_all_entries(), del_links);
    println!();
}

fn execute_drop () {
    utils::delete_database_file();
}

fn execute_transfer(path: &str) {
    if !Path::new(&path).exists() {
        println!("\nFile not found. Exiting...\n{}\n", &path);
        exit(1);
    }
    let default_name = path.split('/').last().unwrap_or("Default Name");
    {
        let mut entry_name = String::new();
        print!("\nEnter the name of the entry (Default name: {}): ", default_name);
        io::stdout().flush().unwrap();
        io::stdin().read_line(&mut entry_name).expect("Failed to read line");
        if entry_name.trim().is_empty() {
            entry_name = default_name.to_string();
        }
        println!("\nUploading... please wait\n");
        utils::transfer_file(entry_name.trim(), path);
    }
    utils::output_data(utils::get_all_entries(), false);
    println!();
}

fn execute_warn_upload() {
    println!("
    You need to inform a path for the file to upload.
    
    Usage:
    -u, --upload [FILE_PATH]   Upload a new link
    ");
}

fn main() {
    utils::create_config_app_folder();
    utils::create_table();
    let args: Vec<String> = env::args().collect();
    match args.len() {
        2 => match args[1].as_str() {
            "-v"  | "--version" => execute_version(),
            "-l"  | "--list" => execute_list(false),
            "-L"  | "--listdel" => execute_list(true),
            "-d"  | "--delete" => execute_delete_by_id(),
            "-D"  | "--drop" => execute_drop(),
            "-u"  | "--upload" => execute_warn_upload(),
            _ => execute_help(),
        },
        3 => match args[1].as_str() {
            "-u" | "--upload" => execute_transfer(&args[2]),
            _ => execute_help(),
        },
        _ => execute_help()
    }
}
