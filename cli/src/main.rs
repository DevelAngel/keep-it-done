use clap::Parser;

/// Family task management with assistant-friendly CLI
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
enum Commands {
    /// Add a new task
    Add,
    /// Delete a task
    Del,
    /// Modify a task
    Mod,
}

fn main() {
    match Commands::parse() {
        Commands::Add => {
            todo!("I will add a new task");
        }
        Commands::Del => {
            todo!("I will delete a task");
        }
        Commands::Mod => {
            todo!("I will modify a task");
        }
    }
}
