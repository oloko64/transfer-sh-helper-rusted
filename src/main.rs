mod utils;

fn main() {
    utils::create_table();
    // utils::transfer_file("Test first file", "./README.md");
    utils::output_data(utils::get_all_entries());
    utils::output_data([utils::get_single_entry(3)].to_vec());
    // utils::delete_entry(1);
    utils::delete_database_file();
}
