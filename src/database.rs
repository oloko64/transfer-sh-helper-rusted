use anyhow::Result;
use std::{fs::remove_file, process::exit};

use sqlite::{open, Row};

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

    pub fn transfer_file(&self, entry_name: &str, file_path: &str) {
        let transfer_response = upload_file(file_path).unwrap_or_else(|err| {
            eprintln!("Error while uploading file: {}", err);
            exit(1)
        });
        self.insert_entry(
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

    pub fn insert_entry(&self, name: &str, link: &str, delete_link: &str) -> Result<()> {
        self.connection
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

    pub fn delete_database_file(&self) -> Result<()> {
        if !ask_confirmation("Are you sure you want to delete the database file?") {
            return Ok(());
        }
        remove_file(&self.database_path)?;
        println!("Database file deleted.\n");
        Ok(())
    }

    pub fn delete_entry(&self, entry_id: i64) {
        let delete_link = if let Some(link) = self
            .get_single_entry(entry_id)
            .expect("Failed to get this entry from the database")
        {
            link.get_delete_link()
        } else {
            println!("\nEntry with id {} not found\n", entry_id);
            return;
        };
        if !ask_confirmation(&format!(
            "Are you sure you want to delete the entry {}? (It will also delete from the cloud)",
            entry_id
        )) {
            return;
        }
        delete_entry_server(&delete_link);
        self.connection
            .execute(format!("DELETE FROM transfer_data WHERE id = {}", entry_id))
            .expect("Failed to delete entry from database");
        println!("Entry with id {} deleted.\n", entry_id);
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
