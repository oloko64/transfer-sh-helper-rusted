use std::{io::stdin, process::Command};
mod sqls;

#[derive(Debug)]
struct TransferResponse {
    transfer_link: String,
    delete_link: String,
}

fn main() {
    sqls::create_table();
    ask_confirmation("Do you want to insert a new record?");
    println!("Unix week -> {}", unix_week());
    println!("{:?}", sqls::get_single_entry(1));
    println!("{:?}", sqls::get_all_entries());
    // sqls::insert_entry("test", "test", "test");
    println!("{:?}", upload_file("./README.md"));
}

fn unix_week() -> i32 {
    1209600
}

fn ask_confirmation(text: &str) -> bool {
    let mut confirmation = String::new();
    println!("{} (y/N)", text);
    stdin().read_line(&mut confirmation).unwrap();
    confirmation.trim().to_ascii_lowercase().starts_with('y')
}

fn upload_file(file_path: &str) -> TransferResponse {
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
            delete_link = line.split("< x-url-delete:").collect::<Vec<&str>>()[1].to_string();
        }
    }
    TransferResponse {
        transfer_link: String::from_utf8_lossy(&output.stdout).to_string(),
        delete_link: delete_link.split_at(delete_link.len() - 1).0.to_string(),
    }
}
