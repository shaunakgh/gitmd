use std::process::Command;
use std::error::Error;
use clap::{Parser, Subcommand};
use colored::*;

#[derive(Parser)]
#[command(author, version, about)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}


#[derive(Subcommand)]
enum Commands {
    Send {
        #[arg(short, long)]
        prompt: String,
    },
}

fn main() -> Result<(), Box<dyn Error>> {
    let model = "deepseek-r1";
    let cli = Cli::parse();

    match &cli.command {
        Commands::Send { prompt } => {
            let output = Command::new("ollama")
                .args(&["run", model, prompt])
                .output()?;
            if output.status.success() {
                println!("{} {}\n {}", "+".green(), "Output: ".bright_yellow().bold(), String::from_utf8_lossy(&output.stdout).blue().italic());
            } else {
                eprintln!("Error: {}", String::from_utf8_lossy(&output.stderr).red().bold());
            }
            Ok(())
        }
    }
}