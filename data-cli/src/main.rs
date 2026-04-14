mod build_admin1;
mod build_data;
mod regen_sqlite;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "data-cli")]
#[command(about = "Data management CLI for zmanim project")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Build admin1 codes mapping for used country/admin pairs
    BuildAdmin1 {
        /// Path to SQLite database
        #[arg(short, long, default_value = "data/cities.db")]
        db: String,

        /// Output path for admin1 codes file
        #[arg(short, long, default_value = "public/data/admin1.json.br")]
        output: String,
    },

    /// Build client data files from SQLite database
    BuildData {
        /// Path to SQLite database
        #[arg(short, long, default_value = "data/cities.db")]
        db: String,

        /// Output path for client data file
        #[arg(short, long, default_value = "public/data/cities.jsonl.br")]
        output: String,
    },
    /// Regenerate the SQLite database from source data files
    RegenSqlite {
        /// Output path for the SQLite database file
        #[arg(short, long, default_value = "data/cities.db")]
        output: String,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::BuildAdmin1 { db, output } => {
            build_admin1::build_admin1(&db, &output);
        }
        Commands::BuildData { db, output } => {
            build_data::build_data(&db, &output);
        }
        Commands::RegenSqlite { output } => {
            regen_sqlite::regenerate_db(&output);
        }
    }
}
