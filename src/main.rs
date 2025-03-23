use std::{
    io::{self, Write},
    process::Command,
    thread,
    time::Duration,
};
use std::{error::Error, fs, path::Path};
use clap::{Parser, ArgGroup};
use regex::Regex;
use colored::*;

#[derive(Parser)]
#[command(author, version, about,
    group(
        ArgGroup::new("mode")
            .args(["readme", "blog", "writeup"])
            .required(true)
    )
)]
struct Cli {
    #[arg(short, long, default_value = ".")]
    path: String,
    #[arg(short, long, default_value = "llama3.2")]
    model: String,
    #[arg(short = 'r', long)]
    readme: bool,
    #[arg(short = 'b', long)]
    blog: bool,
    #[arg(short = 'w', long)]
    writeup: bool,
}

fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();
    let mode = if cli.readme { 1 } else if cli.blog { 2 } else { 3 };
    let output = gen_md(&cli.path, &cli.model, mode)?;

    std::fs::write("output.md", &output)?;
    println!(
        "{} {}",
        "!".bright_yellow().bold(),
        "File has been saved to the current directory as output.md".blue().italic()
    );
    Ok(())
}

fn prog(percent: usize) {
    let width = 50;
    let filled = percent * width / 100;
    let empty = width - filled;

    print!("\r{:3}% ▕{}{} |", percent, "█".repeat(filled), " ".repeat(empty));
    io::stdout().flush().unwrap();
}

fn gen_md(path: &str, model: &str, _type: i32) -> Result<String, Box<dyn Error>> {
    let mut file_dict = std::collections::HashMap::new();

    fn visit_dirs(dir: &Path, file_dict: &mut std::collections::HashMap<String, String>) -> Result<(), Box<dyn Error>> {
        if dir.is_dir() {
            for entry in fs::read_dir(dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_dir() {
                    visit_dirs(&path, file_dict)?;
                } else {
                    let file_name = path.file_name().unwrap().to_string_lossy().to_string();
                    let content = fs::read_to_string(&path)?;
                    file_dict.insert(file_name, content);
                }
            }
        }
        Ok(())
    }

    visit_dirs(Path::new(path), &mut file_dict)?;

    let all_files = serde_json::to_string(&file_dict)?
        .replace('"', "\\\"")
        .replace('\n', "\\n");

    let prompt = match _type {
        1 => format!("Generate a README.md file suitable for a GitHub repository using the context provided by these files {}. The README should clearly describe the project’s purpose, installation instructions, usage examples, and licensing information. Do not include any prefatory remarks or meta‑comments.", all_files),
        2 => format!("Generate a blog post with moderate emotion in Markdown using the context provided by these files {}. The post should balance informative content with narrative flow. Do not include any prefatory remarks or meta‑comments.", all_files),
        _ => format!("Compose a scholarly, professional write‑up in Markdown from the context provided by these files {}. Use academic tone, precise language, and clear structure. Do not include any prefatory remarks or meta‑comments.", all_files),
    };

    let mut child = Command::new("ollama")
        .args(&["run", model, &prompt])
        .spawn()?;

    for i in 0..=100 {
        prog(i);
        if let Ok(Some(_)) = child.try_wait() { break; }
        thread::sleep(Duration::from_millis(20));
    }
    println!();

    let output = child.wait_with_output()?;
    if output.status.success() {
        let raw = String::from_utf8_lossy(&output.stdout);
        let re = Regex::new(r"(?s)<think>.*?</think>")?;
        let cleaned = re.replace_all(&raw, "").trim().to_string();
        Ok(cleaned)
    } else {
        Err(String::from_utf8_lossy(&output.stderr).into())
    }
}