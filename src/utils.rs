use chrono::prelude::{DateTime, NaiveDateTime, Utc};
use dirs::config_dir;
use prettytable::Table;
use serde::{Deserialize, Serialize};
use sqlite::{open, Row};
use std::{
    error::Error,
    fs::{self, create_dir_all, read_to_string, remove_file, write},
    io::{self, Write},
    process::{exit, Command},
    time::{SystemTime, UNIX_EPOCH},
};

pub struct TransferResponse {
    pub transfer_link: String,
    pub delete_link: String,
}

#[derive(Serialize, Deserialize)]
pub struct Config {
    #[serde(rename = "database_file")]
    database_file: String,
}

impl Config {
    fn new() -> Config {
        Config {
            database_file: String::from("transfer-sh-helper.db"),
        }
    }
}

pub struct Link {
    pub id: i64,
    pub name: String,
    pub link: String,
    pub delete_link: String,
    pub unix_time: i64,
    pub is_expired: bool,
}

pub fn get_file_size(path: &str) -> Result<String, Box<dyn Error>> {
    if !fs::metadata(path)?.is_file() {
        return Err(Box::new(io::Error::new(
            io::ErrorKind::NotFound,
            format!("{} is not a file", path),
        )));
    }
    let size = fs::metadata(path)?.len();
    let float_size = size as f64;
    let kb = 1024_f64;
    let mb = (1024 * 1024) as f64;
    let gb = (1024 * 1024 * 1024) as f64;

    match size {
        0 => Err(Box::new(io::Error::new(
            io::ErrorKind::Other,
            "File is empty",
        ))),
        1..=1023 => Ok(format!("{}B", float_size)),
        1024..=1048575 => Ok(format!("{:.2}KB", float_size / kb)),
        1048576..=1073741823 => Ok(format!("{:.2}MB", float_size / mb)),
        1073741824..=1610612735 => Ok(format!("{:.2}GB", float_size / gb)),
        _ => Err(Box::new(io::Error::new(
            io::ErrorKind::Other,
            "File over the 1.5GB limit",
        ))),
    }
}

pub fn get_config() -> Result<Config, Box<dyn Error>> {
    let config_path = config_app_folder() + "transfer-helper-config.json";
    let default_config = Config::new();

    Ok(match read_to_string(&config_path) {
        Ok(config) => match serde_json::from_str(&config) {
            Ok(config) => config,
            Err(_) => {
                write(config_path, serde_json::to_string_pretty(&default_config)?)?;
                default_config
            }
        },
        Err(_) => {
            write(config_path, serde_json::to_string_pretty(&default_config)?)?;
            default_config
        }
    })
}

fn is_link_expired(upload_time: i64) -> bool {
    current_time().expect("Failed to get current time.") - upload_time > unix_week()
}

fn current_time() -> Result<i64, Box<dyn Error>> {
    Ok(SystemTime::now()
        .duration_since(UNIX_EPOCH)?
        .as_secs()
        .try_into()?)
}

pub fn create_config_app_folder() -> Result<(), Box<dyn Error>> {
    create_dir_all(config_app_folder())?;
    Ok(())
}

fn config_app_folder() -> String {
    let config_path = match config_dir() {
        Some(path) => path.display().to_string(),
        None => panic!("Could not get config directory"),
    };
    config_path + "/transfer-sh-helper/"
}

fn database_path() -> String {
    config_app_folder()
        + &get_config()
            .expect("Failed to read config file.")
            .database_file
}

fn unix_week() -> i64 {
    1209600
}

fn ask_confirmation(text: &str) -> bool {
    let mut confirmation = String::new();
    print!("\n{} (y/N): ", text);
    io::stdout().flush().unwrap();
    io::stdin().read_line(&mut confirmation).unwrap();
    println!();
    confirmation.trim().to_lowercase().starts_with('y')
}

pub fn upload_file(file_path: &str) -> Result<TransferResponse, Box<dyn Error>> {
    let output = Command::new("curl")
        .arg("-v")
        .arg("--upload-file")
        .arg(file_path)
        .arg(format!(
            "https://transfer.sh/{}",
            file_path.split('/').last().unwrap()
        ))
        .output()?;

    let mut delete_link = String::new();
    for line in String::from_utf8_lossy(&output.stderr)
        .split('\n')
        .collect::<Vec<&str>>()
    {
        if line.starts_with("< x-url-delete:") {
            delete_link = line.split("< x-url-delete:").collect::<Vec<&str>>()[1]
                .trim()
                .to_string();
            break;
        }
    }
    Ok(TransferResponse {
        transfer_link: String::from_utf8_lossy(&output.stdout).into_owned(),
        delete_link,
    })
}

pub fn transfer_file(entry_name: &str, file_path: &str) {
    let transfer_response = upload_file(file_path).unwrap_or_else(|err| {
        eprintln!("Error while uploading file: {}", err);
        exit(1);
    });
    insert_entry(
        entry_name,
        &transfer_response.transfer_link,
        &transfer_response.delete_link,
    )
    .unwrap_or_else(|err| {
        eprintln!("Error while inserting entry: {}", err);
        eprintln!("But the file was uploaded successfully");
        eprintln!("\nLink: {}", transfer_response.transfer_link);
        eprintln!("Delete link: {}\n", transfer_response.delete_link);
        exit(1);
    });
}

pub fn output_data(data: &Vec<Link>, del_links: bool) -> i32 {
    if data.is_empty() {
        println!("No entries found.");
        println!("Run \"transferhelper -h\" to see all available commands.\n");
        exit(0);
    }
    let mut table = Table::new();
    if del_links {
        table.add_row(row!["ID", "Name", "Delete Link", "Expire Date", "Expired"]);
        for entry in data {
            table.add_row(row![
                entry.id,
                entry.name,
                entry.delete_link,
                readable_date(entry.unix_time),
                entry.is_expired
            ]);
        }
    } else {
        table.add_row(row!["ID", "Name", "Link", "Expire Date", "Expired"]);
        for entry in data {
            table.add_row(row![
                entry.id,
                entry.name,
                entry.link,
                readable_date(entry.unix_time),
                entry.is_expired
            ]);
        }
    }

    table.printstd();
    data.len() as i32
}

fn readable_date(unix_time: i64) -> String {
    let date = DateTime::<Utc>::from_utc(
        NaiveDateTime::from_timestamp(unix_time + unix_week(), 0),
        Utc,
    );
    date.format("%d-%m-%Y").to_string()
}

pub fn delete_database_file() -> Result<(), Box<dyn Error>> {
    if !ask_confirmation("Are you sure you want to delete the database file?") {
        return Ok(());
    }
    remove_file(database_path())?;
    println!("Database file deleted.\n");
    Ok(())
}

// SQL functions

fn open_connection() -> Result<sqlite::Connection, Box<dyn Error>> {
    Ok(open(database_path())?)
}

pub fn delete_entry(entry_id: i64) {
    let delete_link =
        match get_single_entry(entry_id).expect("Failed to get this entry from the database") {
            Some(link) => link.delete_link,
            None => {
                println!("\nEntry with id {} not found\n", entry_id);
                return;
            }
        };
    if !ask_confirmation(&format!(
        "Are you sure you want to delete the entry {}? (It will also delete from the cloud)",
        entry_id
    )) {
        return;
    }
    delete_entry_server(delete_link.as_str());
    let connection = open_connection().expect("Failed to open connection");
    connection
        .execute(format!("DELETE FROM transfer_data WHERE id = {}", entry_id))
        .expect("Failed to delete entry from database");
    println!("Entry with id {} deleted.\n", entry_id);
}

fn delete_entry_server(delete_link: &str) {
    Command::new("curl")
        .arg("-v")
        .arg("-X")
        .arg("DELETE")
        .arg(delete_link)
        .output()
        .expect("Failed to delete entry from transfer.sh servers");
}

pub fn insert_entry(name: &str, link: &str, delete_link: &str) -> Result<(), Box<dyn Error>> {
    let connection = open_connection().expect("Failed to open connection");
    connection
        .execute(
            format!(
                "INSERT INTO transfer_data (name, link, deleteLink, unixTime) VALUES ('{}', '{}', '{}', {})",
                name,
                link,
                delete_link,
                current_time().expect("Failed to get current time.")
            ),
        )?;
    Ok(())
}

pub fn get_single_entry(entry_id: i64) -> Result<Option<Link>, Box<dyn Error>> {
    let connection = open_connection().expect("Failed to open connection");
    let cursor = connection
        .prepare(format!(
            "SELECT * FROM transfer_data WHERE id = {}",
            entry_id
        ))?
        .into_cursor()
        .bind(&[])?;

    if let Some(row) = (cursor.collect::<Result<Vec<Row>, _>>()?)
        .into_iter()
        .next()
    {
        return Ok(Some(Link {
            id: row.get::<i64, _>("id"),
            name: row.get::<String, _>("name"),
            link: row.get::<String, _>("link"),
            delete_link: row.get::<String, _>("deleteLink"),
            unix_time: row.get::<i64, _>("unixTime"),
            is_expired: is_link_expired(row.get::<i64, _>("unixTime")),
        }));
    }
    Ok(None)
}

pub fn create_table() -> Result<(), Box<dyn Error>> {
    let connection = open_connection().expect("Failed to open connection");
    connection.execute(
        "
        CREATE TABLE IF NOT EXISTS transfer_data (
        'id'	INTEGER,
        'name'	TEXT,
        'link'	TEXT,
        'deleteLink'	TEXT,
        'unixTime'	INTEGER,
        PRIMARY KEY('id' AUTOINCREMENT));
        ",
    )?;
    Ok(())
}

pub fn get_all_entries() -> Result<Vec<Link>, Box<dyn Error>> {
    let connection = open_connection().expect("Failed to open connection");
    let cursor = connection
        .prepare("SELECT * FROM transfer_data")?
        .into_cursor()
        .bind(&[])?;

    let mut result: Vec<Link> = vec![];
    for row in cursor.collect::<Result<Vec<Row>, _>>()? {
        result.append(&mut vec![Link {
            id: row.get::<i64, _>("id"),
            name: row.get::<String, _>("name"),
            link: row.get::<String, _>("link"),
            delete_link: row.get::<String, _>("deleteLink"),
            unix_time: row.get::<i64, _>("unixTime"),
            is_expired: is_link_expired(row.get::<i64, _>("unixTime")),
        }]);
    }
    Ok(result)
}
