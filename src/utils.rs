use chrono::prelude::{DateTime, NaiveDateTime, Utc};
use dirs::config_dir;
use owo_colors::OwoColorize;
use reqwest::{Response, StatusCode};
use rusqlite::Row;
use serde::{Deserialize, Serialize};
use std::{
    fs::{create_dir_all, read_to_string, write},
    io::{self, Write},
    path::PathBuf,
    process::exit,
    time::{SystemTime, SystemTimeError, UNIX_EPOCH},
};
use tokio_stream::StreamExt;
use tokio_util::io::ReaderStream;

use crate::{transfer_table, DATABASE};
const UNIX_WEEK: u64 = 1_209_600;

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

    pub fn get_database_file(&self) -> &str {
        &self.database_file
    }
}

pub struct Link {
    id: i64,
    name: String,
    link: String,
    delete_link: String,
    unix_time: u64,
    is_available: bool,
}

impl Link {
    pub fn new(row: &Row) -> Result<Link, Box<dyn std::error::Error>> {
        Ok(Link {
            id: row.get(0)?,
            name: row.get(1)?,
            link: row.get(2)?,
            delete_link: row.get(3)?,
            unix_time: row.get(4)?,
            is_available: Link::is_link_available(row.get(4)?)?,
        })
    }

    fn is_link_available(upload_time: u64) -> Result<bool, SystemTimeError> {
        Ok(current_time()? - upload_time < UNIX_WEEK)
    }

    pub fn get_delete_link(&self) -> &str {
        &self.delete_link
    }
}

pub async fn get_file_size(path: &str) -> Result<String, Box<dyn std::error::Error>> {
    if !tokio::fs::metadata(path).await?.is_file() {
        return Err(
            "Path is not a file. You can use the compress mode `-c <path>` to upload a folder"
                .into(),
        );
    }

    let size = tokio::fs::metadata(path).await?.len();
    let float_size = size as f64;
    let kb = f64::from(1024);
    let mb = f64::from(1024 * 1024);
    let gb = f64::from(1024 * 1024 * 1024);

    match size {
        0 => Err("File is empty".into()),
        1..=1023 => Ok(format!("{float_size} B")),
        1024..=1_048_575 => Ok(format!("{:.2} KB", float_size / kb)),
        1_048_576..=1_073_741_823 => Ok(format!("{:.2} MB", float_size / mb)),
        1_073_741_824..=1_610_612_735 => Ok(format!("{:.2} GB", float_size / gb)),
        _ => Err("File is over the 1.5GB limit".into()),
    }
}

pub fn get_config() -> Result<Config, Box<dyn std::error::Error>> {
    let config_path = config_app_folder()?.join("transfer-helper-config.json");
    let default_config = Config::new();

    Ok(if let Ok(config) = read_to_string(&config_path) {
        if let Ok(config) = serde_json::from_str(&config) {
            config
        } else {
            write(config_path, serde_json::to_string_pretty(&default_config)?)?;
            default_config
        }
    } else {
        write(config_path, serde_json::to_string_pretty(&default_config)?)?;
        default_config
    })
}

pub fn current_time() -> Result<u64, SystemTimeError> {
    Ok(SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs())
}

pub fn create_config_app_folder() -> Result<(), io::Error> {
    create_dir_all(config_app_folder()?)?;
    Ok(())
}

pub fn config_app_folder() -> Result<PathBuf, io::Error> {
    let config_path = config_dir().ok_or(io::Error::new(
        io::ErrorKind::NotFound,
        "Config directory not found",
    ))?;
    Ok(config_path.join("transfer-sh-helper"))
}

pub fn ask_confirmation(text: &str) -> Result<bool, io::Error> {
    let mut confirmation = String::new();
    print!("\n{} (y/N): ", text.yellow());
    io::stdout().flush()?;
    io::stdin().read_line(&mut confirmation)?;
    println!();
    Ok(confirmation.trim().to_lowercase().starts_with('y'))
}

pub async fn upload_file(file_path: &str) -> Result<TransferResponse, Box<dyn std::error::Error>> {
    let file = tokio::fs::File::open(&file_path).await?;
    let total_size = file.metadata().await?.len();
    let mut reader_stream = ReaderStream::new(file);
    let mut total_uploaded = 0_f64;

    let async_stream = async_stream::stream! {
        while let Some(chunk) = reader_stream.next().await {
            if let Ok(chunk) = &chunk {
                total_uploaded += chunk.len() as f64;
                let progress = (total_uploaded / total_size as f64) * 100.0;
                print!("\rUploading... {:.2}%", progress.green());
                io::stdout().flush()?;
            }
            yield chunk;
        }
    };

    let response = reqwest::Client::new()
        .put(&format!(
            "https://transfer.sh/{}",
            file_path
                .split('/')
                .last()
                .ok_or("Failed to get file name from upload URL.")?
        ))
        .body(reqwest::Body::wrap_stream(async_stream))
        .send()
        .await?;

    if response.status() != StatusCode::OK {
        return Err(format!("Failed to upload file. Status code: {}", response.status()).into());
    }

    println!("\n");

    let delete_link = response
        .headers()
        .get("x-url-delete")
        .ok_or("No delete link found.")?
        .to_str()?
        .to_owned();

    Ok(TransferResponse {
        transfer_link: response.text().await?,
        delete_link,
    })
}

pub fn output_data(list_del: bool) -> Result<usize, Box<dyn std::error::Error>> {
    let data = DATABASE.try_lock()?.get_all_entries()?;

    if data.is_empty() {
        println!("No entries found.");
        println!("Run `transferhelper -h` to see all available commands.\n");
        exit(0);
    }
    let data_len = data.len();
    transfer_table!(data, list_del);

    Ok(data_len)
}

fn readable_date(unix_time: u64) -> Result<String, Box<dyn std::error::Error>> {
    let date = DateTime::<Utc>::from_utc(
        NaiveDateTime::from_timestamp_opt((unix_time + UNIX_WEEK).try_into()?, 0)
            .ok_or("Invalid date")?,
        Utc,
    );
    Ok(date.format("%d-%m-%Y").to_string())
}

pub async fn delete_entry_server(
    delete_link: &str,
) -> Result<Response, Box<dyn std::error::Error>> {
    let response = reqwest::Client::new().delete(delete_link).send().await?;

    match response.status() {
        StatusCode::OK | StatusCode::NOT_FOUND => Ok(response),
        _ => Err(format!("Failed to delete entry. Status code: {}", response.status()).into()),
    }
}

pub async fn transfer_response_code() -> Result<StatusCode, reqwest::Error> {
    let transfer_link = "https://transfer.sh/";
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()?;

    client
        .get(transfer_link)
        .send()
        .await
        .map(|response| response.status())
}
