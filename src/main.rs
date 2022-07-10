use std::{
    io::stdin,
    time::{SystemTime, UNIX_EPOCH},
};
mod sqls;

fn main() {
    sqls::create_table();
    println!("Current Unix time -> {}", current_time());
    ask_confirmation("Do you want to insert a new record?");
    println!("Unix week -> {}", unix_week());
    println!("{:?}", sqls::get_single_entry(1));
    println!("{:?}", sqls::get_all_entries());
}

fn unix_week() -> i32 {
    return 1209600;
}

fn current_time() -> u64 {
    let time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    return time;
}

fn ask_confirmation(text: &str) -> bool {
    let mut confirmation = String::new();
    println!("{} (y/N)", text);
    stdin().read_line(&mut confirmation).unwrap();
    if confirmation.trim().to_ascii_lowercase().starts_with("y") {
        return true;
    } else {
        return false;
    }
}
