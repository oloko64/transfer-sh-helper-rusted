use clap::{Parser, Subcommand};
use comprexor::CompressionLevel;

/// A simple way to use Transfer.sh from the CLI.
#[derive(Parser)]
#[command(version)]
pub struct AppArguments {
    #[command(subcommand)]
    pub app_subcommands: Option<AppOptions>,
}

#[derive(Subcommand)]
pub enum AppOptions {
    /// List all uploaded files
    List {
        /// Show delete links
        #[arg(short, long)]
        delete_link: bool,
    },

    /// Delete a file by id, deleting the file from Transfer.sh servers and the local database
    Delete,

    /// Delete the local database but not the files on Transfer.sh servers
    Drop,

    /// Upload files to Transfer.sh servers
    Upload {
        /// Upload a file to Transfer.sh servers
        #[arg(short, long, value_parser = validate_path)]
        path: String,

        /// Compress a file or directory and upload it to Transfer.sh servers
        #[arg(short, long, group = "compress_flag")]
        compress: bool,

        /// Compress level to use when compressing a file or directory
        #[arg(short, long, default_value = "6", requires = "compress_flag", value_parser = validate_compression_level)]
        level: CompressionLevel,
    },
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
