mod utils;
use std::env;

fn execute_version() {
    println!("\nVersion => v0.1.0\n");
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
    utils::output_data(utils::get_all_entries());
    println!();
    let mut id = String::new();
    println!("Enter the id of the entry you want to remove:");
    std::io::stdin().read_line(&mut id).expect("Failed to read line");
    utils::delete_entry(id.trim().parse::<i64>().unwrap())
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
        println!("Enter the name of the entry");
        std::io::stdin().read_line(&mut entry_name).expect("Failed to read line");
        println!("\nUploading... please wait\n");
        utils::transfer_file(entry_name.split_at(entry_name.len() - 1).0, path);
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
