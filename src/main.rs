//! Wordle Bot CLI
//!
//! Interactive command-line interface for the optimal Wordle solver.

use std::io::{self, BufRead, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::Duration;
use wordle_bot::{load_dictionary, FeedbackPattern, WordleSolver};

const BANNER_TEXT: &str = include_str!("text/banner.txt");
const USAGE_TEXT: &str = include_str!("text/usage.txt");

struct Spinner {
    running: Arc<AtomicBool>,
    handle: Option<JoinHandle<()>>,
}

impl Spinner {
    fn new(message: &'static str) -> Self {
        let running = Arc::new(AtomicBool::new(true));
        let running_clone = running.clone();
        let handle = thread::spawn(move || {
            let frames = ['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];
            let mut i = 0;
            while running_clone.load(Ordering::Relaxed) {
                print!("\r{} {}", frames[i % frames.len()], message);
                io::stdout().flush().unwrap();
                thread::sleep(Duration::from_millis(80));
                i += 1;
            }
            print!("\r{}\r", " ".repeat(message.len() + 3));
            io::stdout().flush().unwrap();
        });
        Self { running, handle: Some(handle) }
    }

    fn stop(mut self) {
        self.running.store(false, Ordering::Relaxed);
        if let Some(handle) = self.handle.take() {
            handle.join().unwrap();
        }
    }
}

impl Drop for Spinner {
    fn drop(&mut self) {
        self.running.store(false, Ordering::Relaxed);
    }
}

fn print_banner() {
    for line in BANNER_TEXT.lines().take(6) {
        println!("{}", line);
    }
}

fn print_help() {
    println!("{}", BANNER_TEXT);
}

fn run_interactive() {
    print_banner();

    println!("Loading dictionary...");
    let words = load_dictionary();
    println!("Loaded {} words.", words.len());
    println!();

    let mut solver = WordleSolver::new(words);
    println!("Type 'help' for commands or 'suggest' to get started.");
    println!();

    let stdin = io::stdin();
    let mut stdout = io::stdout();

    loop {
        print!("> ");
        stdout.flush().unwrap();

        let mut line = String::new();
        if stdin.lock().read_line(&mut line).unwrap() == 0 {
            break;
        }

        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.is_empty() {
            continue;
        }

        match parts[0].to_lowercase().as_str() {
            "help" | "h" | "?" => {
                print_help();
            }
            "quit" | "exit" | "q" => {
                println!("Goodbye!");
                break;
            }
            "suggest" | "s" | "best" => {
                match solver.find_best_guess() {
                    Some(analysis) => {
                        println!();
                        println!("Best guess: {} ", analysis.word.to_uppercase());
                        println!("  Entropy: {:.3} bits", analysis.entropy);
                        println!("  Expected remaining: {:.1} words", analysis.expected_remaining);
                        if analysis.is_possible_answer {
                            println!("  ✓ This word is a possible answer");
                        } else {
                            println!("  ✗ This word is NOT a possible answer");
                        }
                        println!();
                        println!("Remaining possibilities: {}", solver.remaining_count());
                        if solver.is_hard_mode() {
                            println!("Mode: HARD");
                        }
                        println!();
                    }
                    None => {
                        println!("No possible words remaining. Use 'reset' to start over.");
                    }
                }
            }
            "hard" | "hardmode" => {
                let new_mode = !solver.is_hard_mode();
                solver.set_hard_mode(new_mode);
                if new_mode {
                    println!("Hard mode: ON");
                    println!("Guesses must use all revealed hints.");
                } else {
                    println!("Hard mode: OFF");
                }
            }
            "top" | "t" => {
                let n: usize = parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(5);
                let top = solver.get_top_guesses(n);

                if top.is_empty() {
                    println!("No possible words remaining.");
                } else {
                    println!();
                    println!("Top {} guesses:", top.len());
                    println!("{:>4} {:>8} {:>8} {:>12} Possible?", "#", "Word", "Entropy", "Exp. Remain");
                    println!("{}", "-".repeat(50));
                    for (i, analysis) in top.iter().enumerate() {
                        println!(
                            "{:>4} {:>8} {:>8.3} {:>12.1} {}",
                            i + 1,
                            analysis.word.to_uppercase(),
                            analysis.entropy,
                            analysis.expected_remaining,
                            if analysis.is_possible_answer { "✓" } else { "" }
                        );
                    }
                    println!();
                }
            }
            "feedback" | "f" | "fb" => {
                if parts.len() < 3 {
                    println!("Usage: feedback <word> <pattern>");
                    println!("Example: feedback crane gybbb");
                    continue;
                }

                let word = parts[1].to_lowercase();
                let pattern_str = parts[2].to_lowercase();

                match FeedbackPattern::parse(&pattern_str) {
                    Some(pattern) => {
                        let prev_count = solver.remaining_count();
                        solver.apply_feedback(&word, pattern);
                        let new_count = solver.remaining_count();

                        println!();
                        println!("Guess: {}", word.to_uppercase());
                        println!("Feedback: {}", pattern);
                        println!(
                            "Eliminated {} words ({} → {})",
                            prev_count - new_count,
                            prev_count,
                            new_count
                        );

                        if pattern.is_win() {
                            println!();
                            println!("🎉 Congratulations! You solved it!");
                            println!();
                        } else if new_count == 0 {
                            println!();
                            println!("⚠️  No words match this feedback pattern!");
                            println!("This might indicate an error. Use 'reset' to start over.");
                            println!();
                        } else if new_count <= 10 {
                            println!();
                            println!("Remaining words: {:?}", 
                                solver.possible_answers().iter()
                                    .map(|s| s.to_uppercase())
                                    .collect::<Vec<_>>());
                            println!();
                        }
                        println!();
                    }
                    None => {
                        println!("Invalid pattern: {}", pattern_str);
                        println!("Use g=green, y=yellow, b=black (5 characters)");
                    }
                }
            }
            "remaining" | "r" | "left" => {
                let remaining = solver.possible_answers();
                println!();
                println!("Remaining possibilities: {}", remaining.len());
                if remaining.len() <= 20 {
                    for (i, word) in remaining.iter().enumerate() {
                        if i > 0 && i % 10 == 0 {
                            println!();
                        }
                        print!("{:>8}", word.to_uppercase());
                    }
                    println!();
                }
                println!();
            }
            "solve" => {
                if parts.len() < 2 {
                    println!("Usage: solve <target_word>");
                    continue;
                }

                let target = parts[1].to_lowercase();
                if target.len() != 5 {
                    println!("Word must be 5 letters.");
                    continue;
                }

                println!();
                println!("Solving for: {}", target.to_uppercase());
                println!();

                solver.reset();
                let guesses = solver.solve_for_target(&target);

                for (i, (guess, pattern)) in guesses.iter().enumerate() {
                    println!(
                        "Guess {}: {} → {}",
                        i + 1,
                        guess.to_uppercase(),
                        pattern
                    );
                }

                println!();
                if let Some((_, pattern)) = guesses.last() {
                    if pattern.is_win() {
                        println!("✓ Solved in {} guesses!", guesses.len());
                    } else {
                        println!("✗ Failed to solve within 6 guesses.");
                    }
                }
                println!();
                solver.reset();
            }
            "benchmark" | "bench" => {
                println!();
                println!("Running benchmark on all {} words...", solver.all_words().len());

                let spinner = Spinner::new("Computing...");
                let start = std::time::Instant::now();
                let distribution = solver.benchmark_guess_distribution();
                let elapsed = start.elapsed();
                spinner.stop();

                let total: usize = distribution.iter().map(|(_, c)| c).sum();
                let total_guesses: usize = distribution.iter().map(|(g, c)| g * c).sum();
                let average = total_guesses as f64 / total as f64;

                println!("Results:");
                println!("{}", "=".repeat(40));
                println!();
                println!("Guess distribution:");
                for (guesses, count) in &distribution {
                    let pct = *count as f64 / total as f64 * 100.0;
                    let bar = "█".repeat((*count * 40 / total).max(1));
                    println!("  {} guesses: {:>5} ({:>5.1}%) {}", guesses, count, pct, bar);
                }
                println!();
                println!("Average guesses: {:.3}", average);
                println!("Total words: {}", total);
                println!("Time elapsed: {:.2?}", elapsed);

                let failures = distribution.iter()
                    .filter(|(g, _)| *g > 6)
                    .map(|(_, c)| c)
                    .sum::<usize>();
                if failures > 0 {
                    println!("Words not solved in 6 guesses: {}", failures);
                } else {
                    println!("✓ All words solved within 6 guesses!");
                }
                println!();
            }
            "reset" => {
                solver.reset();
                println!("Reset to initial state. {} words available.", solver.remaining_count());
            }
            _ => {
                println!("Unknown command: {}", parts[0]);
                println!("Type 'help' for available commands.");
            }
        }
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() > 1 {
        match args[1].as_str() {
            "--help" | "-h" => {
                println!("{}", USAGE_TEXT);
            }
            "solve" => {
                if args.len() < 3 {
                    eprintln!("Usage: wordle-bot solve <word>");
                    std::process::exit(1);
                }

                let target = args[2].to_lowercase();
                if target.len() != 5 {
                    eprintln!("Word must be 5 letters.");
                    std::process::exit(1);
                }

                let words = load_dictionary();
                let mut solver = WordleSolver::new(words);

                println!("Solving for: {}", target.to_uppercase());
                println!();

                let guesses = solver.solve_for_target(&target);

                for (i, (guess, pattern)) in guesses.iter().enumerate() {
                    println!("Guess {}: {} → {}", i + 1, guess.to_uppercase(), pattern);
                }

                println!();
                if let Some((_, pattern)) = guesses.last() {
                    if pattern.is_win() {
                        println!("Solved in {} guesses.", guesses.len());
                    }
                }
            }
            "benchmark" | "bench" => {
                let words = load_dictionary();
                let solver = WordleSolver::new(words);

                let spinner = Spinner::new("Running benchmark...");
                let start = std::time::Instant::now();
                let avg = solver.benchmark_average_guesses();
                let elapsed = start.elapsed();
                spinner.stop();

                println!("Average guesses: {:.3}", avg);
                println!("Time: {:.2?}", elapsed);
            }
            "suggest" => {
                let words = load_dictionary();
                let solver = WordleSolver::new(words);

                match solver.find_best_guess() {
                    Some(analysis) => {
                        println!("Best opening guess: {}", analysis.word.to_uppercase());
                        println!("Entropy: {:.3} bits", analysis.entropy);
                    }
                    None => {
                        eprintln!("No words available.");
                    }
                }
            }
            _ => {
                eprintln!("Unknown command: {}", args[1]);
                eprintln!("Use --help for usage information.");
                std::process::exit(1);
            }
        }
    } else {
        run_interactive();
    }
}

