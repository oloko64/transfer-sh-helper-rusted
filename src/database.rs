use anyhow::Result;
use sqlite::{open, Row};
use std::{fs::remove_file, process::exit};

use crate::utils::{
    ask_confirmation, config_app_folder, create_config_app_folder, current_time,
    delete_entry_server, get_config, upload_file, Link,
};

pub struct Database {
    connection: sqlite::Connection,
    database_path: String,
}

impl Database {
    pub fn new() -> Database {
        create_config_app_folder().expect("Failed to create config folder.");

        let database_path = config_app_folder()
            + &get_config()
                .expect("Failed to read config file.")
                .get_database_file();
        Database {
            connection: open(&database_path).expect("Failed to open database file."),
            database_path,
        }
    }

    pub fn create_table(&self) -> Result<()> {
        self.connection.execute(
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

    pub fn get_all_entries(&self) -> Result<Vec<Link>> {
        let cursor = self
            .connection
            .prepare("SELECT * FROM transfer_data")?
            .into_iter();

        let mut result: Vec<Link> = vec![];
        for row in cursor.collect::<Result<Vec<Row>, _>>()? {
            result.append(&mut vec![Link::new(&row)]);
        }
        Ok(result)
    }

    pub async fn transfer_file(&self, entry_name: &str, file_path: &str) {
        let transfer_response = upload_file(file_path).await.unwrap_or_else(|err| {
            eprintln!("Error while uploading file: {err}");
            exit(1)
        });
        self.insert_entry(
            entry_name,
            &transfer_response.transfer_link,
            &transfer_response.delete_link,
        )
        .unwrap_or_else(|err| {
            eprintln!("Error while inserting entry: {err}");
            eprintln!("But the file was uploaded successfully");
            eprintln!("\nLink: {}", transfer_response.transfer_link);
            eprintln!("Delete link: {}\n", transfer_response.delete_link);
            exit(1);
        });
    }

    pub fn insert_entry(&self, name: &str, link: &str, delete_link: &str) -> Result<()> {
        let current_time = &current_time()
            .expect("Failed to get current time.")
            .to_string();
        let query = "INSERT INTO transfer_data (name, link, deleteLink, unixTime) VALUES (:name, :link, :deleteLink, :unixTime)";
        let query_params = &[
            (":name", name),
            (":link", link),
            (":deleteLink", delete_link),
            (":unixTime", current_time),
        ][..];

        let mut statement = self.connection.prepare(query)?;
        statement.bind(query_params)?;
        statement.next()?;

        Ok(())
    }

    pub fn delete_database_file(&self) -> Result<()> {
        if !ask_confirmation("Are you sure you want to delete the database file?") {
            return Ok(());
        }
        remove_file(&self.database_path)?;
        println!("Database file deleted.\n");
        Ok(())
    }

    pub async fn delete_entry(&self, entry_id: i64) {
        let delete_link = if let Some(link) = self
            .get_single_entry(entry_id)
            .expect("Failed to get this entry from the database")
        {
            link.get_delete_link()
        } else {
            println!("\nEntry with id {entry_id} not found.\n");
            return;
        };
        if !ask_confirmation(&format!(
            "Are you sure you want to delete the entry {entry_id}? (It will also delete from the cloud)"
        )) {
            return;
        }

        let query = format!("DELETE FROM transfer_data WHERE id = :id");
        let mut statement = self.connection.prepare(&query).unwrap();
        statement
            .bind((":id", entry_id))
            .expect("Failed to bind id to query");

        match delete_entry_server(&delete_link).await {
            Ok(_) => {
                statement
                    .next()
                    .expect("Failed to delete entry from database");
                println!("Entry with id {entry_id} deleted.\n");
            }
            Err(err) => {
                eprintln!("Error while deleting entry from server: {err}");
                if ask_confirmation("Do you want to delete the entry from the database anyway? (It will still be accessible from the link)") {
                    statement.next().expect("Failed to delete entry from database");
                    println!("Entry with id {entry_id} deleted.\n");
                }
            }
        };
    }

    pub fn get_single_entry(&self, entry_id: i64) -> Result<Option<Link>> {
        let cursor = self
            .connection
            .prepare("SELECT * FROM transfer_data WHERE id = ?")?
            .into_iter()
            .bind((1, entry_id))?;

        if let Some(row) = (cursor.collect::<Result<Vec<Row>, _>>()?)
            .into_iter()
            .next()
        {
            return Ok(Some(Link::new(&row)));
        }
        Ok(None)
    }
}
