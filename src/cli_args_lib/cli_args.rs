use clap::Parser;

/// SQLite Editor
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct MyArgs {
    /// Path to a file containing a SQLite schema
    #[clap(short, long)]
    pub schema: bool,

    /// Path to a SQLite database file
    // #[clap(short, long, last = true)]
    pub path: Option<String>,
}
