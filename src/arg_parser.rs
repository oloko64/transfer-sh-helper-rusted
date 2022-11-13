use clap::Parser;

/// A simple way to use Transfer.sh from the CLI.
#[derive(Parser, Debug)]
#[clap(version)]
pub struct Args {
    /// List all links on database
    #[clap(short, long, action)]
    pub list: bool,

    /// List all delete links on database
    #[clap(long, action)]
    pub list_del: bool,

    /// Deletes a entry locally as well as from Transfer.sh servers
    #[clap(short, long, action)]
    pub delete: bool,

    /// Delete the database from your system
    #[clap(long, action)]
    pub drop: bool,

    /// Upload a file to Transfer.sh servers
    #[clap(short, long, value_parser)]
    pub upload_file: Option<String>,
}

pub enum AppOptions {
    List,
    ListDel,
    Delete,
    Drop,
    Transfer(String),
}

impl AppOptions {
    fn new(args: Args) -> AppOptions {
        if args.list {
            AppOptions::List
        } else if args.list_del {
            AppOptions::ListDel
        } else if args.delete {
            AppOptions::Delete
        } else if args.drop {
            AppOptions::Drop
        } else if let Some(path) = args.upload_file {
            AppOptions::Transfer(path)
        } else {
            AppOptions::List
        }
    }
}

pub fn prepare_args() -> AppOptions {
    let args = Args::parse();
    AppOptions::new(args)
}
