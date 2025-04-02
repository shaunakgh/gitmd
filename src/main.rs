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
use colored::*;

#[derive(Parser)]
#[command(author, version, about,
    group(
        ArgGroup::new("mode")
            .args(["readme", "blog", "writeup"])
            .required(true)
    )
)]
// CLI structure
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

// Progress bar
fn prog(percent: usize) {
    let width = 50;
    let filled = percent * width / 100;
    let empty = width - filled;

    print!("\r{:3}% ▕{}{} |", percent, "█".repeat(filled), " ".repeat(empty));
    io::stdout().flush().unwrap();
}

// TODO: Fix visit_dirs functionality - some files are omitted or not recorded
// Get all code for context in folder
fn visit_dirs(dir: &Path, file_dict: &mut HashMap<String, String>) -> Result<(), Box<dyn Error>> {
    let allowed: [&str; 34] = [
        "md","markdown","txt","rst","adoc",
        "rs","py","java","js","jsx","ts","tsx","go","c","cpp","h","hpp","swift","kt","kts","rb","php",
        "html","htm","xml","yaml","yml","json","toml","ini","cfg","sh","bash","zsh"
    ];

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.components().any(|c| c.as_os_str() == ".git") {
            continue;
        }

        if path.is_dir() {
            visit_dirs(&path, file_dict)?;
            continue;
        }
        if let Some(ext) = path.extension().and_then(|e| e.to_str()).map(|s| s.to_lowercase()) {
            if !allowed.contains(&ext.as_str()) {
                continue;
            }
        } else {
            continue;
        }
        let name = path.strip_prefix(dir)?.to_string_lossy().to_string();
        match fs::read_to_string(&path) {
            Ok(contents) => { file_dict.insert(name, contents); }
            Err(err) if err.kind() == std::io::ErrorKind::InvalidData => {
                eprintln!("Skipping non‑UTF8 file → {}", name);
            }
            Err(err) => return Err(err.into()),
        }
    }
    Ok(())
}

// GENERATION CODE
fn gen_md(path: &str, model: &str, _type: i32) -> Result<String, Box<dyn Error>> {
    let mut file_dict = HashMap::new();
    visit_dirs(Path::new(path), &mut file_dict)?;

    // Get type (README/Blog/Scholar)
    let all_files = serde_json::to_string(&file_dict)?;
    let prompt = match _type {
        1 => format!("Generate a README.md file with a but not limited to a brief overview and description of the project, a key rundown of features and a usage guide suitable for a GitHub repository using these files:\n\n{}.", all_files),
        2 => format!("Generate a blog post in Markdown with a but not limited to a brief overview and description of the project, a key rundown of features and a usage guide using these files:\n\n{}.", all_files),
        3 => format!("Compose a scholarly write‑up in Markdown with a but not limited to a brief overview and description of the project, a key rundown of features and a usage guide using these files:\n\n{}.", all_files),
        _ => format!("Invalid"),
    };

    for i in 0..=100 {
        prog(i);
        thread::sleep(Duration::from_millis(20));
    }

    // Init AI
    let output = Command::new("ollama")
        .args(&["run", model, &prompt])
        .output()?;
    let output_str = String::from_utf8_lossy(&output.stdout);
    if output.status.success() {
        // Cleanup (remove excess output)
        let cleaned = regex::Regex::new(r"(?si)<think>.*?</think>")
            .unwrap()
            .replace_all(&output_str, "")
            .trim()
            .to_string();
        Ok(cleaned)
    } else {
        Err(String::from_utf8_lossy(&output.stderr).into())
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();
    let mode = if cli.readme { 1 } else if cli.blog { 2 } else { 3 };
    let output = gen_md(&cli.path, &cli.model, mode)?;
    fs::write("output.md", &output)?;
    println!(
        "\n{} {}",
        "!".bright_yellow().bold(),
        "File has been saved to output.md. You might need to remove excess output and meta-comments.".blue().italic()
    );
    Ok(())
}
