//! # Wordle Bot
//!
//! A multithreaded optimal Wordle solver using entropy-based information theory.
//!
//! The solver uses the maximum entropy strategy to find the optimal guess at each step,
//! maximizing the expected information gain from the feedback.

pub mod feedback;
pub mod solver;

pub use feedback::{Feedback, FeedbackPattern};
pub use solver::WordleSolver;

/// Word length for Wordle
pub const WORD_LENGTH: usize = 5;

/// Load the dictionary from the embedded file
pub fn load_dictionary() -> Vec<String> {
    include_str!("../dictionary/dictionary.txt")
        .lines()
        .filter(|line| !line.is_empty())
        .map(|s| s.to_lowercase())
        .collect()
}
