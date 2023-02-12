use anyhow::{bail, ensure, Result};
use chrono::prelude::{DateTime, NaiveDateTime, Utc};
use dirs::config_dir;
use owo_colors::OwoColorize;
use prettytable::{row, Table};
use reqwest::Response;
use serde::{Deserialize, Serialize};
use sqlite::Row;
use std::{
    fs::{self, create_dir_all, read_to_string, write},
    io::{self, Write},
    process::exit,
    time::{SystemTime, UNIX_EPOCH},
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

    pub fn get_database_file(&self) -> String {
        self.database_file.clone()
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
    pub fn new(row: &Row) -> Link {
        Link {
            id: row.read::<i64, _>("id"),
            name: row.read::<&str, _>("name").to_owned(),
            link: row.read::<&str, _>("link").to_owned(),
            delete_link: row.read::<&str, _>("deleteLink").to_owned(),
            unix_time: row
                .read::<i64, _>("unixTime")
                .try_into()
                .expect("unixTime cannot be converted to u64"),
            is_available: Link::is_link_available(
                row.read::<i64, _>("unixTime")
                    .try_into()
                    .expect("unixTime cannot be converted to u64"),
            ),
        }
    }

    fn is_link_available(upload_time: u64) -> bool {
        current_time().expect("Failed to get current time.") - upload_time < UNIX_WEEK
    }

    pub fn get_delete_link(&self) -> String {
        self.delete_link.clone()
    }
}

pub fn get_file_size(path: &str) -> Result<String> {
    ensure!(fs::metadata(path)?.is_file(), "Path is not a file.");

    let size = fs::metadata(path)?.len();
    let float_size = size as f64;
    let kb = f64::from(1024);
    let mb = f64::from(1024 * 1024);
    let gb = f64::from(1024 * 1024 * 1024);

    match size {
        0 => bail!("File is empty"),
        1..=1023 => Ok(format!("{float_size}B")),
        1024..=1_048_575 => Ok(format!("{:.2}KB", float_size / kb)),
        1_048_576..=1_073_741_823 => Ok(format!("{:.2}MB", float_size / mb)),
        1_073_741_824..=1_610_612_735 => Ok(format!("{:.2}GB", float_size / gb)),
        _ => bail!("File over the 1.5GB limit"),
    }
}

pub fn get_config() -> Result<Config> {
    let config_path = config_app_folder() + "transfer-helper-config.json";
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

pub fn current_time() -> Result<u64> {
    Ok(SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs())
}

pub fn create_config_app_folder() -> Result<()> {
    create_dir_all(config_app_folder())?;
    Ok(())
}

pub fn config_app_folder() -> String {
    let config_path = match config_dir() {
        Some(path) => path.display().to_string(),
        None => panic!("Could not get config directory"),
    };
    config_path + "/transfer-sh-helper/"
}

pub fn ask_confirmation(text: &str) -> bool {
    let mut confirmation = String::new();
    print!("\n{} (y/N): ", text.yellow());
    io::stdout().flush().unwrap();
    io::stdin().read_line(&mut confirmation).unwrap();
    println!();
    confirmation.trim().to_lowercase().starts_with('y')
}

pub async fn upload_file(file_path: &str) -> Result<TransferResponse> {
    let file = tokio::fs::File::open(&file_path)
        .await
        .expect("Cannot open file.");
    let total_size = file
        .metadata()
        .await
        .expect("Cannot determine input file size")
        .len();
    let mut reader_stream = ReaderStream::new(file);
    let mut total_uploaded = 0_f64;

    let async_stream = async_stream::stream! {
        while let Some(chunk) = reader_stream.next().await {
            if let Ok(chunk) = &chunk {
                total_uploaded += chunk.len() as f64;
                let progress = (total_uploaded / total_size as f64) * 100.0;
                print!("\rUploading... {:.2}%", progress.green());
                io::stdout().flush().unwrap();
            }
            yield chunk;
        }
    };

    let response = reqwest::Client::new()
        .put(&format!(
            "https://transfer.sh/{}",
            file_path.split('/').last().expect("Cannot get file name.")
        ))
        .body(reqwest::Body::wrap_stream(async_stream))
        .send()
        .await?;

    println!("\n");

    let delete_link = response
        .headers()
        .get("x-url-delete")
        .expect("No delete link found.")
        .to_str()?
        .to_owned();

    Ok(TransferResponse {
        transfer_link: response.text().await?,
        delete_link,
    })
}

pub fn output_data(list_del: bool) -> usize {
    let data = DATABASE
        .try_lock()
        .expect("Failed to acquire lock of database.")
        .get_all_entries()
        .expect("Failed to get data from database.");

    if data.is_empty() {
        println!("No entries found.");
        println!("Run \"transferhelper -h\" to see all available commands.\n");
        exit(0);
    }
    transfer_table!(&data, list_del);

    data.len()
}

fn readable_date(unix_time: u64) -> String {
    let date = DateTime::<Utc>::from_utc(
        NaiveDateTime::from_timestamp_opt(
            (unix_time + UNIX_WEEK)
                .try_into()
                .expect("Failed to convert u64 to i64"),
            0,
        )
        .expect("Invalid date"),
        Utc,
    );
    date.format("%d-%m-%Y").to_string()
}

pub async fn delete_entry_server(delete_link: &str) -> Result<Response, reqwest::Error> {
    reqwest::Client::new().delete(delete_link).send().await
}
