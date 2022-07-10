use home::home_dir;
use sqlite::open;
use std::{fs::create_dir_all, io::Result};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug)]
pub struct Link {
    pub id: i64,
    pub name: String,
    pub link: String,
    pub delete_link: String,
    pub unix_time: i64,
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
        result.append(
            &mut vec![Link {
                id: row[0].as_integer().unwrap(),
                name: String::from(row[1].as_string().unwrap()),
                link: String::from(row[2].as_string().unwrap()),
                delete_link: String::from(row[3].as_string().unwrap()),
                unix_time: row[4].as_integer().unwrap(),
            }],
        );
    }
    result
}

pub fn get_single_entry(entry_id: u32) -> Link {
    let connection = open_connection();
    let mut cursor = connection
        .prepare(format!(
            "SELECT * FROM transfer_data WHERE id = {}",
            entry_id
        ))
        .unwrap()
        .into_cursor();

    let mut result: Link = Link {
        id: 0,
        name: String::new(),
        link: String::new(),
        delete_link: String::new(),
        unix_time: 0,
    };

    while let Some(row) = cursor.next().unwrap() {
        result = Link {
            id: row[0].as_integer().unwrap(),
            name: String::from(row[1].as_string().unwrap()),
            link: String::from(row[2].as_string().unwrap()),
            delete_link: String::from(row[3].as_string().unwrap()),
            unix_time: row[4].as_integer().unwrap(),
        };
    }
    result
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

fn current_time() -> u64 {
    
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

fn open_connection() -> sqlite::Connection {
    let folder = create_config_app_folder();
    if let Err(e) = folder {
        println!("{}", e);
    };
    open(database_path()).unwrap()
}

fn create_config_app_folder() -> Result<()> {
    create_dir_all(config_app_folder())?;
    Ok(())
}

fn config_app_folder() -> String {
    let path = match home_dir() {
        Some(path) => path.display().to_string(),
        None => "".to_string(),
    };
    path + "/.config/transfer-sh-helper-database/"
}

fn database_path() -> String {
    [&config_app_folder(), "test.db"].join("")
}
