use clap::Parser;
use comprexor::CompressionLevel;

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

    /// Compress a file or directory and upload it to Transfer.sh servers
    #[arg(short, long, group = "upload_type", group = "compress_type")]
    pub compress_upload: Option<String>,

    /// Compress level to use when compressing a file or directory
    #[arg(long, default_value = "6", requires = "compress_type", value_parser = validate_compression_level)]
    pub level: CompressionLevel,

    /// Upload a file to Transfer.sh servers without compressing it
    #[arg(short, long, group = "upload_type", value_parser = validate_path)]
    pub upload_file: Option<String>,
}

fn validate_compression_level(level: &str) -> Result<CompressionLevel, String> {
    match level.parse::<u32>() {
        Ok(level) if (level <= 9) => Ok(CompressionLevel::Custom(level)),
        _ => Err(format!(
            "Invalid compression level: `{level}`, must be between 0 and 9"
        )),
    }
}

fn validate_path(path: &str) -> Result<String, String> {
    if std::path::Path::new(path).exists() {
        Ok(path.to_string())
    } else {
        Err(format!("Provided path does not exist: `{path}`"))
    }
}

pub enum AppOptions {
    List { list_del: bool },
    Delete,
    Drop,
    TransferFile(String),
    TransferCompressed(String, CompressionLevel),
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
            AppOptions::TransferFile(path)
        } else if let Some(path) = args.compress_upload {
            AppOptions::TransferCompressed(path, args.level)
        } else {
            AppOptions::List {
                list_del: args.list_del,
            }
        }
    }
}
