use home::home_dir;
use sqlite::open;
use std::{
    fs::{create_dir_all, remove_file},
    io::stdin,
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};
use chrono::prelude::{DateTime, Utc, NaiveDateTime};

#[derive(Debug)]
pub struct TransferResponse {
    pub transfer_link: String,
    pub delete_link: String,
}

#[derive(Debug, Clone)]
pub struct Link {
    pub id: i64,
    pub name: String,
    pub link: String,
    pub delete_link: String,
    pub unix_time: i64,
    pub is_expired: bool,
}

pub fn create_table() {
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

pub fn get_all_entries() -> Vec<Link> {
    let connection = open_connection();
    let mut cursor = connection
        .prepare("SELECT * FROM transfer_data")
        .unwrap()
        .into_cursor();

    let mut result: Vec<Link> = vec![];

    while let Some(row) = cursor.next().unwrap() {
        result.append(&mut vec![Link {
            id: row[0].as_integer().unwrap(),
            name: String::from(row[1].as_string().unwrap()),
            link: String::from(row[2].as_string().unwrap()),
            delete_link: String::from(row[3].as_string().unwrap()),
            unix_time: row[4].as_integer().unwrap(),
            is_expired: is_link_expired(row[4].as_integer().unwrap()),
        }]);
    }
    result
}

fn is_link_expired(upload_time: i64) -> bool {
    current_time() - upload_time > unix_week()
}

pub fn get_single_entry(entry_id: i64) -> Option<Link> {
    let connection = open_connection();
    let mut cursor = connection
        .prepare(format!(
            "SELECT * FROM transfer_data WHERE id = {}",
            entry_id
        ))
        .unwrap()
        .into_cursor();

    if let Some(row) = cursor.next().unwrap() {
        return Some(Link {
            id: row[0].as_integer().unwrap(),
            name: String::from(row[1].as_string().unwrap()),
            link: String::from(row[2].as_string().unwrap()),
            delete_link: String::from(row[3].as_string().unwrap()),
            unix_time: row[4].as_integer().unwrap(),
            is_expired: is_link_expired(row[4].as_integer().unwrap()),
        });
    }
    None
}

pub fn insert_entry(name: &str, link: &str, delete_link: &str) {
    let connection = open_connection();
    connection
        .execute(
            format!(
                "INSERT INTO transfer_data (name, link, deleteLink, unixTime) VALUES ('{}', '{}', '{}', {})",
                name,
                link,
                delete_link,
                current_time()
            ),
        )
        .unwrap();
}

fn current_time() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
        .try_into()
        .unwrap()
}

fn open_connection() -> sqlite::Connection {
    create_config_app_folder();
    open(database_path()).unwrap()
}

fn create_config_app_folder() {
    create_dir_all(config_app_folder()).unwrap()
}

fn config_app_folder() -> String {
    let path = match home_dir() {
        Some(path) => path.display().to_string(),
        None => panic!("Could not get home directory"),
    };
    path + "/.config/transfer-sh-helper-database/"
}

fn database_path() -> String {
    [&config_app_folder(), "transfer-sh-helper.db"].join("")
}

fn unix_week() -> i64 {
    1209600
}

fn ask_confirmation(text: &str) -> bool {
    let mut confirmation = String::new();
    println!("\n{} (y/N)", text);
    stdin().read_line(&mut confirmation).unwrap();
    confirmation.trim().to_ascii_lowercase().starts_with('y')
}

pub fn upload_file(file_path: &str) -> TransferResponse {
    let output = Command::new("curl")
        .arg("-v")
        .arg("--upload-file")
        .arg(file_path)
        .arg(format!(
            "https://transfer.sh/{}",
            file_path.split('/').last().unwrap()
        ))
        .output()
        .expect("Failed to execute command");

    let mut delete_link = String::new();
    for line in String::from_utf8_lossy(&output.stderr)
        .split('\n')
        .collect::<Vec<&str>>()
    {
        if line.starts_with("< x-url-delete:") {
            delete_link = line.split("< x-url-delete:").collect::<Vec<&str>>()[1].trim().to_string();
        }
    }
    TransferResponse {
        transfer_link: String::from_utf8_lossy(&output.stdout).to_string(),
        delete_link,
    }
}

pub fn transfer_file(entry_name: &str, file_path: &str) {
    let transfer_response = upload_file(file_path);
    insert_entry(
        entry_name,
        &transfer_response.transfer_link,
        &transfer_response.delete_link,
    );
}

pub fn output_data(data: Vec<Link>) {
    for entry in data {
        if entry.link.is_empty() && entry.delete_link.is_empty() {
            continue;
        }
        println!(
            "Id: {} | Name: {} | Link: {} | Delete link: {} | Expire Date: {} | Expired: {}",
            entry.id,
            entry.name,
            entry.link,
            entry.delete_link,
            readable_date(entry.unix_time),
            entry.is_expired
        );
    }
}

fn readable_date(unix_time: i64) -> String {
    let date = DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(unix_time + unix_week(), 0), Utc);
    date.format("%Y-%m-%d").to_string()
}

fn delete_entry_server(delete_link: &str) {
    Command::new("curl")
        .arg("-v")
        .arg("-X")
        .arg("DELETE")
        .arg(delete_link)
        .output()
        .expect("Failed to delete from transfer sh servers");
}

pub fn delete_entry(entry_id: i64) {
    let delete_link = match get_single_entry(entry_id) {
        Some (link) => link.delete_link,
        None => String::new(),
    };
    if delete_link.is_empty() {
        println!("\nNo delete link found with id: {}\n", entry_id);
        return;
    }
    if !ask_confirmation(format!("Are you sure you want to delete the entry {}?", entry_id).as_str()) {
        return;
    }
    delete_entry_server(delete_link.as_str());
    let connection = open_connection();
    connection
        .execute(
            format!(
                "DELETE FROM transfer_data WHERE id = {}",
                entry_id
            ),
        )
        .unwrap();
}

pub fn delete_database_file() {
    if !ask_confirmation("Are you sure you want to delete the database file?") {
        return;
    }
    remove_file(database_path()).unwrap();
}
