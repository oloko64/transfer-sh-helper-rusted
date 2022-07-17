mod utils;
use std::{env, io::{self, Write}};

fn execute_version() {
    println!("\nVersion => v0.1.3\n");
}

fn execute_help() {
    println!("
    Usage:
    -h, --help                 Prints help
    -v, --version              Prints version
    -u, --upload [FILE_PATH]   Upload a new link
    -l, --list                 Lists all links
    -d, --delete               Deletes a specific link
    -DD, --drop                Deletes database file
    ");
}

fn execute_delete_by_id() {
    println!();
    utils::output_data(utils::get_all_entries());
    println!();
    let mut id = String::new();
    print!("Enter the id of the entry you want to remove: ");
    io::stdout().flush().unwrap();
    io::stdin().read_line(&mut id).expect("Failed to read line");
    utils::delete_entry(id.trim().parse::<i64>().expect("Failed to parse id"))
}

fn execute_list() {
    println!();
    utils::output_data(utils::get_all_entries());
    println!();
}

fn execute_drop () {
    utils::delete_database_file();
}

fn execute_transfer(path: &str) {
    {
        let mut entry_name = String::new();
        print!("Enter the name of the entry: ");
        io::stdout().flush().unwrap();
        io::stdin().read_line(&mut entry_name).expect("Failed to read line");
        entry_name = entry_name.trim().to_string();
        println!("\nUploading... please wait\n");
        utils::transfer_file(&entry_name, path);
    }
    utils::output_data(utils::get_all_entries());
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
            "-l"  | "--list" => execute_list(),
            "-d"  | "--delete" => execute_delete_by_id(),
            "-DD" | "--drop" => execute_drop(),
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
