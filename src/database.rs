use std::{fs::remove_file, path::PathBuf, sync::Arc};
use tokio::try_join;

use crate::{
    errors::TransferError,
    utils::{
        ask_confirmation, calculate_sha25sum, config_app_folder, create_config_app_folder,
        current_time, delete_entry_server, get_config, upload_file, Link,
    },
};

pub struct Database {
    connection: rusqlite::Connection,
    database_path: PathBuf,
}

impl Database {
    pub fn new() -> Result<Database, TransferError> {
        create_config_app_folder()?;

        let binding = get_config()?;
        let database_file = binding.get_database_file();

        let database_path = config_app_folder()?.join(database_file);
        Ok(Database {
            connection: rusqlite::Connection::open(&database_path)?,
            database_path,
        })
    }

    pub fn create_table(&self) -> Result<(), TransferError> {
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
            (),
        )?;

        let count = self.connection.query_row(
            "SELECT COUNT(*) FROM pragma_table_info('transfer_data') WHERE name = 'sha256sum'",
            [],
            |row| row.get::<_, i64>(0),
        )?;

        if count == 0 {
            self.connection.execute(
                "
                    ALTER TABLE transfer_data ADD COLUMN 'sha256sum' TEXT;
                    ",
                (),
            )?;
        }

        Ok(())
    }

    pub fn get_all_entries(&self) -> Result<Vec<Link>, TransferError> {
        let mut stmt = self.connection.prepare("SELECT * FROM transfer_data")?;
        let mut rows = stmt.query([])?;

        let mut result: Vec<Link> = vec![];
        while let Some(row) = rows.next()? {
            result.push(Link::new(row)?);
        }
        Ok(result)
    }

    pub async fn transfer_file(
        &self,
        entry_name: &str,
        file_path: &str,
    ) -> Result<(), TransferError> {
        let arc_file_path = Arc::new(file_path.to_string());
        let upload_handle = tokio::spawn(upload_file(Arc::clone(&arc_file_path)));
        let sha256sum_handle = tokio::spawn(calculate_sha25sum(Arc::clone(&arc_file_path)));
        let (transfer_response, file_hash) =
            try_join!(upload_handle, sha256sum_handle).map_err(|err| err.to_string())?;
        let transfer_response = transfer_response?;
        let file_hash = file_hash?;
        self.insert_entry(
            entry_name,
            &transfer_response.transfer_link,
            &transfer_response.delete_link,
            &file_hash,
        )?;

        Ok(())
    }

    pub fn insert_entry(
        &self,
        name: &str,
        link: &str,
        delete_link: &str,
        sha256sum: &str,
    ) -> Result<(), TransferError> {
        let current_time = &current_time()?.to_string();
        let query = "INSERT INTO transfer_data (name, link, deleteLink, unixTime, sha256sum) VALUES (:name, :link, :deleteLink, :unixTime, :sha256sum)";
        let query_params = &[
            (":name", name),
            (":link", link),
            (":deleteLink", delete_link),
            (":unixTime", current_time),
            (":sha256sum", sha256sum),
        ];

        let mut stmt = self.connection.prepare(query)?;
        stmt.execute(query_params)?;

        Ok(())
    }

    pub fn delete_database_file(&self) -> Result<(), TransferError> {
        if !ask_confirmation("Are you sure you want to delete the database file?")? {
            return Ok(());
        }
        remove_file(&self.database_path)?;
        println!("Database file deleted.\n");
        Ok(())
    }

    pub async fn delete_entry(&mut self, entry_id: i64) -> Result<(), TransferError> {
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

        let query = "DELETE FROM transfer_data WHERE id = ?";
        let transaction = self.connection.transaction()?;
        transaction.prepare(query)?.execute([&entry_id])?;

        match delete_entry_server(&delete_link).await {
            Ok(_) => {
                transaction.commit()?;
                println!("Entry with id {entry_id} deleted.\n");
            }
            Err(err) => {
                eprintln!("Error while deleting entry from server: {err}");
                if ask_confirmation("Do you want to delete the entry from the database anyway? (It will still be accessible from the link)")? {
                    transaction.commit()?;
                    println!("Entry with id {entry_id} deleted.\n");
                } else {
                    transaction.rollback()?;
                    println!("Entry with id {entry_id} not deleted.\n");
                }
            }
        }

        Ok(())
    }

    pub fn get_single_entry(&self, entry_id: i64) -> Result<Option<Link>, TransferError> {
        let mut stmt = self
            .connection
            .prepare("SELECT * FROM transfer_data WHERE id = ?")?;

        let params = &[&entry_id];
        let mut rows = stmt.query(params)?;

        if let Some(row) = (rows.next()?).into_iter().next() {
            return Ok(Some(Link::new(row)?));
        }
        Ok(None)
    }
}
