use std::process::Command;
use std::error::Error;
use colored::*;

fn main() -> Result<(), Box<dyn Error>> {
    let model = "deepseek-r1";
    let prompt = "Tell me a joke.";

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