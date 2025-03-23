use std::{
    collections::HashMap,
    error::Error,
    fs,
    io::{self, Write},
    path::Path,
    process::Command,
    thread,
    time::Duration,
};
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
    fs::write("output.md", &output)?;
    println!(
        "{} {}",
        "!".bright_yellow().bold(),
        "File has been saved to output.md".blue().italic()
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

fn visit_dirs(dir: &Path, file_dict: &mut HashMap<String, String>) -> Result<(), Box<dyn Error>> {
    if !dir.exists() {
        return Err(format!("Directory not found: {}", dir.display()).into());
    }

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            visit_dirs(&path, file_dict)?;
        } else {
            let name = path.strip_prefix(dir)?
                .to_string_lossy()
                .to_string();
            println!("Adding file → {}", name);
            let content = fs::read_to_string(&path)?;
            file_dict.insert(name, content);
        }
    }
    Ok(())
}

fn gen_md(path: &str, model: &str, _type: i32) -> Result<String, Box<dyn Error>> {
    let mut file_dict = HashMap::new();
    visit_dirs(Path::new(path), &mut file_dict)?;

    let all_files = serde_json::to_string(&file_dict)?;
    println!("{}", all_files);
    let prompt = match _type {
        1 => format!("Generate a README.md file suitable for a GitHub repository using these files:\n\n{}", all_files),
        2 => format!("Generate a blog post in Markdown using these files:\n\n{}", all_files),
        _ => format!("Compose a scholarly write‑up in Markdown using these files:\n\n{}", all_files),
    };

    let mut child = Command::new("ollama")
        .args(&["run", model, &prompt])
        .spawn()?;

    for i in 0..=100 {
        prog(i);
        if child.try_wait()?.is_some() { break; }
        thread::sleep(Duration::from_millis(20));
    }
    println!();

    let output = child.wait_with_output()?;
    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).into());
    }

    let raw = String::from_utf8_lossy(&output.stdout);
    let cleaned = Regex::new(r"(?si)<think>.*?</think>")?
        .replace_all(&raw, "")
        .trim()
        .to_string();

    Ok(cleaned)
}