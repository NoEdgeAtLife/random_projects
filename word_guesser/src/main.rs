use colored::*;
use rand::seq::SliceRandom;
use std::collections::HashSet;
use std::io::{self, Write};

fn main() {
    println!("{}", "WORD GUESSER".bright_green().bold());
    println!("{}", "Try to guess the secret word!".bright_blue());
    println!();

    let words = vec![
        "rust", "code", "programming", "computer", "algorithm",
        "function", "variable", "memory", "compiler", "debug",
        "error", "syntax", "logic", "binary", "integer",
        "string", "array", "vector", "struct", "trait",
        "module", "crate", "macro", "async", "await",
    ];

    let mut rng = rand::thread_rng();
    let secret_word = words.choose(&mut rng).unwrap().to_string();
    let word_length = secret_word.len();
    
    println!("The secret word has {} letters.", word_length);
    
    let max_attempts = 6;
    let mut attempts = 0;
    let mut guessed_letters = HashSet::new();
    let mut word_progress: Vec<char> = vec!['_'; word_length];
    
    while attempts < max_attempts {
        // Display current progress
        print!("Word: ");
        for c in &word_progress {
            print!("{} ", c);
        }
        println!();
        
        // Show guessed letters
        print!("Guessed letters: ");
        for letter in &guessed_letters {
            print!("{} ", letter);
        }
        println!();
        
        // Show remaining attempts
        println!("Attempts remaining: {}", max_attempts - attempts);
        
        // Get user input
        print!("Enter a letter or guess the whole word: ");
        io::stdout().flush().unwrap();
        
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let input = input.trim().to_lowercase();
        
        if input.is_empty() {
            println!("{}", "Please enter something!".red());
            continue;
        }
        
        // Check if the input is a single letter or a word
        if input.len() == 1 {
            let guess_char = input.chars().next().unwrap();
            
            if !guess_char.is_alphabetic() {
                println!("{}", "Please enter a letter!".red());
                continue;
            }
            
            if guessed_letters.contains(&guess_char) {
                println!("{}", "You already guessed that letter!".yellow());
                continue;
            }
            
            guessed_letters.insert(guess_char);
            
            let mut found = false;
            for (i, c) in secret_word.chars().enumerate() {
                if c == guess_char {
                    word_progress[i] = c;
                    found = true;
                }
            }
            
            if found {
                println!("{}", "Good guess!".green());
            } else {
                println!("{}", "Letter not in the word!".red());
                attempts += 1;
            }
        } else {
            // The user is guessing the entire word
            if input == secret_word {
                for (i, c) in secret_word.chars().enumerate() {
                    word_progress[i] = c;
                }
                println!("{}", "You guessed the word!".bright_green().bold());
                break;
            } else {
                println!("{}", "That's not the word!".red());
                attempts += 1;
            }
        }
        
        // Check if the word has been completely guessed
        if !word_progress.contains(&'_') {
            println!("{}", "You've revealed the entire word!".bright_green().bold());
            break;
        }
        
        println!(); // Add an empty line for readability
    }
    
    if attempts >= max_attempts {
        println!("{}. The word was: {}", "Game over! You've used all your attempts".red().bold(), secret_word.bright_yellow());
    } else {
        println!("{}! The word was: {}", "Congratulations! You won".bright_green().bold(), secret_word.bright_yellow());
    }
    
    println!("\nThanks for playing Word Guesser!");
}
