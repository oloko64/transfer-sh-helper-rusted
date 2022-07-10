mod utils;

fn main() {
    utils::create_table();
    println!("{:?}", utils::get_single_entry(1));
    println!("{:?}", utils::get_all_entries());
    // sqls::insert_entry("test", "test", "test");
    println!("{:?}", utils::upload_file("./README.md"));
    utils::transfer_file("Test first file", "./README.md");
}
