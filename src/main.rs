use std::path::{PathBuf};
use clap::{Parser, Subcommand};

mod webserver;
mod build;
mod utils;
mod structs;

#[derive(Parser, Debug)]
#[command(version, about)]
struct Cli {
    // 1. This directs clap to look for a subcommand next
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Run the server
    Run {
        /// The path to the folder you want to execute
        folder: PathBuf,

        // You can still mix in optional flags if you want
        /// Enable extra debugging output
        #[arg(short, long)]
        verbose: bool,
    },
    Build{
        // The path to the folder you want to build
        folder: PathBuf,
    },
    /// Another optional subcommand (like ./program stop)
    Stop,
}

fn main() {

    let cli = Cli::parse();

    match &cli.command {
        Commands::Run { folder, verbose } => {
            if *verbose {
                println!("Debug: Preparing to run process...");
            }
            webserver::start_server(PathBuf::from(folder));
        }
        Commands::Build { folder } => {
            println!("Building code in folder: {:?}", folder);
            build::build_code(PathBuf::from(folder));
        }
        Commands::Stop => {
            println!("Stopping all active operations.");
        }
    }

    // Need to create a build step that will open the provided directory and then compile each of the files
    // And then the run will be able to run the compiled files in the provided directory
}
