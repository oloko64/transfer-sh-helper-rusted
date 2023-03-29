use clap::Parser;

/// A simple way to use Transfer.sh from the CLI.
#[derive(Parser)]
#[command(version)]
pub struct Args {
    /// List all links on database
    #[arg(short, long)]
    pub list: bool,

    /// List all delete links on database
    #[arg(long, short = 'L')]
    pub list_del: bool,

    /// Deletes a entry locally as well as from Transfer.sh servers
    #[arg(short, long)]
    pub delete: bool,

    /// Delete the database from your system
    #[arg(long)]
    pub drop: bool,

    /// Upload a file to Transfer.sh servers
    #[arg(short, long, value_parser = validate_path)]
    pub upload_file: Option<String>,
}

fn validate_path(path: &str) -> Result<String, String> {
    if std::path::Path::new(path).exists() {
        Ok(path.to_string())
    } else {
        Err(format!(r#"Provided path does not exist: "{path}""#))
    }
}

pub enum AppOptions {
    List { list_del: bool },
    Delete,
    Drop,
    Transfer(String),
}

impl AppOptions {
    pub fn init() -> AppOptions {
        let args = Args::parse();

        if args.list {
            AppOptions::List {
                list_del: args.list_del,
            }
        } else if args.delete {
            AppOptions::Delete
        } else if args.drop {
            AppOptions::Drop
        } else if let Some(path) = args.upload_file {
            AppOptions::Transfer(path)
        } else {
            AppOptions::List {
                list_del: args.list_del,
            }
        }
    }
}
