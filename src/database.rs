use sqlite::{open, Row};
use std::{fs::remove_file, io};

use crate::utils::{
    ask_confirmation, config_app_folder, create_config_app_folder, current_time,
    delete_entry_server, get_config, upload_file, Link,
};

pub struct Database {
    connection: sqlite::Connection,
    database_path: String,
}

impl Database {
    pub fn new() -> Result<Database, Box<dyn std::error::Error>> {
        create_config_app_folder()?;

        let binding = get_config()?;
        let database_file = binding.get_database_file();

        let database_path = config_app_folder() + database_file;
        Ok(Database {
            connection: open(&database_path)?,
            database_path,
        })
    }

    pub fn create_table(&self) -> Result<(), Box<dyn std::error::Error>> {
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

    pub fn get_all_entries(&self) -> Result<Vec<Link>, Box<dyn std::error::Error>> {
        let cursor = self
            .connection
            .prepare("SELECT * FROM transfer_data")?
            .into_iter();

        let mut result: Vec<Link> = vec![];
        for row in cursor.collect::<Result<Vec<Row>, _>>()? {
            result.append(&mut vec![Link::new(&row)?]);
        }
        Ok(result)
    }

    pub async fn transfer_file(
        &self,
        entry_name: &str,
        file_path: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let transfer_response = upload_file(file_path).await?;
        self.insert_entry(
            entry_name,
            &transfer_response.transfer_link,
            &transfer_response.delete_link,
        )?;

        Ok(())
    }

    pub fn insert_entry(
        &self,
        name: &str,
        link: &str,
        delete_link: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let current_time = &current_time()?.to_string();
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

    pub fn delete_database_file(&self) -> Result<(), io::Error> {
        if !ask_confirmation("Are you sure you want to delete the database file?")? {
            return Ok(());
        }
        remove_file(&self.database_path)?;
        println!("Database file deleted.\n");
        Ok(())
    }

    pub async fn delete_entry(&self, entry_id: i64) -> Result<(), Box<dyn std::error::Error>> {
        let delete_link = if let Some(link) = self.get_single_entry(entry_id)? {
            link.get_delete_link().to_string()
        } else {
            println!("\nEntry with id {entry_id} not found.\n");
            return Ok(());
        };
        if !ask_confirmation(&format!(
            "Are you sure you want to delete the entry {entry_id}? (It will also delete from the cloud)"
        ))? {
            return Ok(());
        }

        let query = "DELETE FROM transfer_data WHERE id = :id";
        let mut statement = self.connection.prepare(query)?;
        statement.bind((":id", entry_id))?;

        match delete_entry_server(&delete_link).await {
            Ok(_) => {
                statement.next()?;
                println!("Entry with id {entry_id} deleted.\n");
            }
            Err(err) => {
                eprintln!("Error while deleting entry from server: {err}");
                if ask_confirmation("Do you want to delete the entry from the database anyway? (It will still be accessible from the link)")? {
                    statement.next()?;
                    println!("Entry with id {entry_id} deleted.\n");
                }
            }
        };

        Ok(())
    }

    pub fn get_single_entry(
        &self,
        entry_id: i64,
    ) -> Result<Option<Link>, Box<dyn std::error::Error>> {
        let cursor = self
            .connection
            .prepare("SELECT * FROM transfer_data WHERE id = ?")?
            .into_iter()
            .bind((1, entry_id))?;

        if let Some(row) = (cursor.collect::<Result<Vec<Row>, _>>()?)
            .into_iter()
            .next()
        {
            return Ok(Some(Link::new(&row)?));
        }
        Ok(None)
    }
}
