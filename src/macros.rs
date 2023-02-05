#[macro_export]
macro_rules! transfer_table {
    ($data:expr, $del_links:expr) => {
        let mut table = Table::new();
        if $del_links {
            table.add_row(row![c->"ID", "Name", "Delete Link", c->"Expire Date", c->"Expired"]);
            for entry in $data {
                if entry.is_expired {
                    table.add_row(row![
                        c->entry.id,
                        entry.name,
                        entry.delete_link,
                        c->readable_date(entry.unix_time),
                        cFr->entry.is_expired
                    ]);
                } else {
                    table.add_row(row![
                        c->entry.id,
                        entry.name,
                        entry.delete_link,
                        c->readable_date(entry.unix_time),
                        cFg->entry.is_expired
                    ]);
                }
            }
        } else {
            table.add_row(row![c->"ID", "Name", "Link", c->"Expire Date", c->"Expired"]);
            for entry in $data {
                if entry.is_expired {
                    table.add_row(row![
                        c->entry.id,
                        entry.name,
                        entry.link,
                        c->readable_date(entry.unix_time),
                        cFr->entry.is_expired
                    ]);
                } else {
                    table.add_row(row![
                        c->entry.id,
                        entry.name,
                        entry.link,
                        c->readable_date(entry.unix_time),
                        cFg->entry.is_expired
                    ]);
                }
            }
        }
        table.printstd();
    };
}
