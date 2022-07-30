use clap::Parser;

/// A simple way to use Transfer.sh.
#[derive(Parser, Debug)]
#[clap(version)]
pub struct Args {
    /// List all links on database
    #[clap(short, long, action)]
    pub list: bool,

    /// List all delete links on database
    #[clap(long, action)]
    pub list_del: bool,

    /// Deletes a entry from the database as well as from Transfer.sh servers
    #[clap(short, long, action)]
    pub delete: bool,

    /// Delete the database from your system
    #[clap(long, action)]
    pub drop: bool,

    /// Upload a file to Transfer.sh servers
    #[clap(short, long, value_parser)]
    pub upload: Option<String>,
}

pub fn prepare_args() -> Args {
    Args::parse()
}
