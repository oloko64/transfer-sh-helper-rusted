mod utils;

fn main() {
    utils::create_table();
    // println!("{:?}", utils::get_single_entry(1));
    // println!("{:?}", utils::get_all_entries());
    // sqls::insert_entry("test", "test", "test");
    // utils::transfer_file("Test first file", "./README.md");
    utils::output_data(utils::get_all_entries());
    utils::output_data([utils::get_single_entry(3)].to_vec());
    utils::delete_entry(2);
}
