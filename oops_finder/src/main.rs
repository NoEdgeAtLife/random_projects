use clap::{Arg, Command};
use std::fs;
use std::path::Path;

fn main() {
    let matches = Command::new("oops_finder")
        .version("1.0")
        .author("Your Name")
        .about("Searches for files in a directory")
        .arg(
            Arg::new("pattern")
                .short('p')
                .long("pattern")
                .value_name("PATTERN")
                .help("Pattern to search for")
                .takes_value(true)
                .required(true),
        )
        .arg(
            Arg::new("directory")
                .short('d')
                .long("directory")
                .value_name("DIRECTORY")
                .help("Directory to search in")
                .takes_value(true)
                .default_value("."),
        )
        .get_matches();

    let pattern = matches.value_of("pattern").unwrap();
    let directory = matches.value_of("directory").unwrap();

    println!("Searching for '{}' in '{}'", pattern, directory);

    if let Err(e) = search_files(directory, pattern) {
        eprintln!("Error: {}", e);
    }
}

fn search_files(directory: &str, pattern: &str) -> Result<(), Box<dyn std::error::Error>> {
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            search_files(path.to_str().unwrap(), pattern)?;
        } else if let Some(file_name) = path.file_name() {
            if file_name.to_string_lossy().contains(pattern) {
                println!("{}", path.display());
            }
        }
    }
    Ok(())
}
