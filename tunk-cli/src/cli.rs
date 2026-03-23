use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "tunk", about = "A tunnel management daemon")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    Start { name: String },
    Stop { name: String },
    Restart { name: String },
    Status { name: String },
    List,
    Reload,
}
