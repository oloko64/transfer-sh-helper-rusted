use home::home_dir;
use sqlite::open;
use std::{
    fs::create_dir_all,
    io::{stdin, Result},
    time::{SystemTime, UNIX_EPOCH},
};

fn main() {
    create_table();
    println!("Current Unix time -> {}", current_time());
    ask_confirmation("Do you want to insert a new record?");
    println!("Config path -> {}", config_app_folder());
    database_path();
    println!("Unix week -> {}", unix_week());
}

fn unix_week() -> i32 {
    return 1209600;
}

fn current_time() -> u64 {
    let time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    return time;
}

fn ask_confirmation(text: &str) -> bool {
    let mut confirmation = String::new();
    println!("{} (y/N)", text);
    stdin().read_line(&mut confirmation).unwrap();
    if confirmation.trim().to_ascii_lowercase().starts_with("y") {
        return true;
    } else {
        return false;
    }
}

fn database_path() -> String {
    return [&config_app_folder(), "test.db"].join("");
}

fn create_config_app_folder() -> Result<()> {
    create_dir_all(config_app_folder())?;
    Ok(())
}

fn open_connection() -> sqlite::Connection {
    let folder = create_config_app_folder();
    if let Err(e) = folder {
        println!("{}", e);
    };
    return open(database_path()).unwrap();
}

fn config_app_folder() -> String {
    let path = match home_dir() {
        Some(path) => path.display().to_string(),
        None => "".to_string(),
    };
    return path + "/.config/transfer-sh-helper-database/";
}

fn create_table() {
    let connection = open_connection();
    connection
        .execute(
            "
        CREATE TABLE IF NOT EXISTS transfer_data (
        'id'	INTEGER,
        'name'	TEXT,
        'link'	TEXT,
        'deleteLink'	TEXT,
        'unixTime'	INTEGER,
        PRIMARY KEY('id' AUTOINCREMENT));
        ",
        )
        .unwrap();
}
