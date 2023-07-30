#[macro_export]
macro_rules! transfer_table {
    ($data:expr, $del_links:expr) => {
        use comfy_table::modifiers::UTF8_ROUND_CORNERS;
        use comfy_table::presets::UTF8_FULL;
        use comfy_table::{Cell, CellAlignment, Color, ContentArrangement, Table};

        let mut table = Table::new();
        table
            .load_preset(UTF8_FULL)
            .apply_modifier(UTF8_ROUND_CORNERS)
            .set_content_arrangement(ContentArrangement::Dynamic);
        if $del_links {
            table.set_header([
                Cell::new("ID").set_alignment(CellAlignment::Center),
                Cell::new("Name"),
                Cell::new("Delete Link"),
                Cell::new("Expire Date").set_alignment(CellAlignment::Center),
                Cell::new("Available").set_alignment(CellAlignment::Center),
            ]);
            for entry in $data {
                table.add_row([
                    Cell::new(entry.id).set_alignment(CellAlignment::Center),
                    Cell::new(entry.name),
                    Cell::new(entry.delete_link),
                    Cell::new(readable_date(entry.unix_time)?).set_alignment(CellAlignment::Center),
                    if entry.is_available {
                        Cell::new(entry.is_available)
                            .fg(Color::Green)
                            .set_alignment(CellAlignment::Center)
                    } else {
                        Cell::new(entry.is_available)
                            .fg(Color::Red)
                            .set_alignment(CellAlignment::Center)
                    },
                ]);
            }
        } else {
            table.set_header([
                Cell::new("ID").set_alignment(CellAlignment::Center),
                Cell::new("Name"),
                Cell::new("Link"),
                Cell::new("Expire Date").set_alignment(CellAlignment::Center),
                Cell::new("Available").set_alignment(CellAlignment::Center),
            ]);
            for entry in $data {
                table.add_row([
                    Cell::new(entry.id).set_alignment(CellAlignment::Center),
                    Cell::new(entry.name),
                    Cell::new(entry.link),
                    Cell::new(readable_date(entry.unix_time)?).set_alignment(CellAlignment::Center),
                    if entry.is_available {
                        Cell::new(entry.is_available)
                            .fg(Color::Green)
                            .set_alignment(CellAlignment::Center)
                    } else {
                        Cell::new(entry.is_available)
                            .fg(Color::Red)
                            .set_alignment(CellAlignment::Center)
                    },
                ]);
            }
        }
        println!("{table}");
    };
}
