//! Optimal Wordle solver using entropy-based strategy.
//!
//! This module implements an information-theoretic approach to solving Wordle.
//! The key insight is that we want to maximize the expected information gain
//! (entropy) from each guess, which minimizes the expected number of remaining
//! possible words.

use crate::feedback::{Feedback, FeedbackPattern};
use crate::WORD_LENGTH;
use rayon::prelude::*;

/// Result of analyzing a potential guess
#[derive(Debug, Clone)]
pub struct GuessAnalysis {
    pub word: String,
    pub entropy: f64,
    pub expected_remaining: f64,
    pub is_possible_answer: bool,
}

/// Hard mode constraints from previous guesses
#[derive(Debug, Clone, Default)]
pub struct HardModeConstraints {
    /// Letters that must be in specific positions (green)
    pub required_positions: [Option<char>; WORD_LENGTH],
    /// Letters that must appear somewhere in the word (yellow)
    pub required_letters: Vec<char>,
}

impl HardModeConstraints {
    pub fn new() -> Self {
        Self::default()
    }

    /// Update constraints based on a guess and its feedback
    pub fn update(&mut self, guess: &str, pattern: FeedbackPattern) {
        let feedbacks = pattern.to_feedbacks();
        let guess_chars: Vec<char> = guess.chars().collect();

        for (i, &fb) in feedbacks.iter().enumerate() {
            match fb {
                Feedback::Correct => {
                    self.required_positions[i] = Some(guess_chars[i]);
                }
                Feedback::Present => {
                    if !self.required_letters.contains(&guess_chars[i]) {
                        self.required_letters.push(guess_chars[i]);
                    }
                }
                Feedback::Absent => {}
            }
        }
    }

    /// Check if a word satisfies all hard mode constraints
    pub fn is_valid(&self, word: &str) -> bool {
        let word_chars: Vec<char> = word.chars().collect();

        for (i, &required) in self.required_positions.iter().enumerate() {
            if let Some(c) = required {
                if word_chars.get(i) != Some(&c) {
                    return false;
                }
            }
        }

        for &required in &self.required_letters {
            if !word_chars.contains(&required) {
                return false;
            }
        }

        true
    }

    pub fn is_empty(&self) -> bool {
        self.required_positions.iter().all(|p| p.is_none()) && self.required_letters.is_empty()
    }
}

/// The main Wordle solver
#[derive(Debug, Clone)]
pub struct WordleSolver {
    all_words: Vec<String>,
    possible_answers: Vec<String>,
    hard_mode: bool,
    constraints: HardModeConstraints,
}

impl WordleSolver {
    pub fn new(words: Vec<String>) -> Self {
        Self {
            possible_answers: words.clone(),
            all_words: words,
            hard_mode: false,
            constraints: HardModeConstraints::new(),
        }
    }

    pub fn set_hard_mode(&mut self, enabled: bool) {
        self.hard_mode = enabled;
    }

    pub fn is_hard_mode(&self) -> bool {
        self.hard_mode
    }

    pub fn remaining_count(&self) -> usize {
        self.possible_answers.len()
    }

    pub fn possible_answers(&self) -> &[String] {
        &self.possible_answers
    }

    pub fn all_words(&self) -> &[String] {
        &self.all_words
    }

    pub fn reset(&mut self) {
        self.possible_answers = self.all_words.clone();
        self.constraints = HardModeConstraints::new();
    }

    pub fn apply_feedback(&mut self, guess: &str, pattern: FeedbackPattern) {
        if self.hard_mode {
            self.constraints.update(guess, pattern);
        }
        self.possible_answers.retain(|word| {
            FeedbackPattern::calculate(guess, word) == pattern
        });
    }

    fn valid_guesses(&self) -> Vec<&String> {
        if self.hard_mode && !self.constraints.is_empty() {
            self.all_words
                .iter()
                .filter(|w| self.constraints.is_valid(w))
                .collect()
        } else {
            self.all_words.iter().collect()
        }
    }

    pub fn calculate_entropy_for_word(&self, guess: &str) -> f64 {
        let n = self.possible_answers.len() as f64;
        if n <= 1.0 {
            return 0.0;
        }

        let mut pattern_counts = [0u32; FeedbackPattern::NUM_PATTERNS];

        for answer in &self.possible_answers {
            let pattern = FeedbackPattern::calculate(guess, answer);
            pattern_counts[pattern.0 as usize] += 1;
        }

        let mut entropy = 0.0;
        for &count in &pattern_counts {
            if count > 0 {
                let p = count as f64 / n;
                entropy -= p * p.log2();
            }
        }

        entropy
    }

    pub fn find_best_guess(&self) -> Option<GuessAnalysis> {
        self.find_best_guesses(1).into_iter().next()
    }

    pub fn find_best_guesses(&self, n: usize) -> Vec<GuessAnalysis> {
        if self.possible_answers.is_empty() {
            return vec![];
        }

        if self.possible_answers.len() == 1 {
            return vec![GuessAnalysis {
                word: self.possible_answers[0].clone(),
                entropy: 0.0,
                expected_remaining: 1.0,
                is_possible_answer: true,
            }];
        }

        if self.possible_answers.len() == 2 {
            return vec![GuessAnalysis {
                word: self.possible_answers[0].clone(),
                entropy: 1.0,
                expected_remaining: 1.0,
                is_possible_answer: true,
            }];
        }

        let valid_guesses = self.valid_guesses();
        let mut analyses: Vec<GuessAnalysis> = valid_guesses
            .par_iter()
            .map(|word| {
                let entropy = self.calculate_entropy_for_word(word);
                let is_possible = self.possible_answers.contains(*word);
                let expected_remaining =
                    self.possible_answers.len() as f64 / 2_f64.powf(entropy);

                GuessAnalysis {
                    word: (*word).clone(),
                    entropy,
                    expected_remaining,
                    is_possible_answer: is_possible,
                }
            })
            .collect();

        analyses.sort_by(|a, b| {
            match b.entropy.partial_cmp(&a.entropy) {
                Some(std::cmp::Ordering::Equal) => b.is_possible_answer.cmp(&a.is_possible_answer),
                Some(ord) => ord,
                None => std::cmp::Ordering::Equal,
            }
        });

        analyses.truncate(n);
        analyses
    }

    pub fn get_top_guesses(&self, n: usize) -> Vec<GuessAnalysis> {
        self.find_best_guesses(n)
    }

    /// Solve a Wordle puzzle automatically, given a function that provides feedback
    /// Returns the sequence of guesses made
    pub fn solve_with_feedback<F>(&mut self, mut get_feedback: F) -> Vec<(String, FeedbackPattern)>
    where
        F: FnMut(&str) -> FeedbackPattern,
    {
        let mut guesses = Vec::new();

        for _ in 0..6 {
            let best = match self.find_best_guess() {
                Some(g) => g,
                None => break,
            };

            let pattern = get_feedback(&best.word);
            guesses.push((best.word.clone(), pattern));

            if pattern.is_win() {
                break;
            }

            self.apply_feedback(&best.word, pattern);
        }

        guesses
    }

    /// Solve a puzzle knowing the target word (for testing/benchmarking)
    pub fn solve_for_target(&mut self, target: &str) -> Vec<(String, FeedbackPattern)> {
        self.solve_with_feedback(|guess| FeedbackPattern::calculate(guess, target))
    }

    /// Calculate the average number of guesses needed to solve all words
    pub fn benchmark_average_guesses(&self) -> f64 {
        let total_guesses: usize = self
            .all_words
            .par_iter()
            .map(|target| {
                let mut solver = self.clone();
                let guesses = solver.solve_for_target(target);
                guesses.len()
            })
            .sum();

        total_guesses as f64 / self.all_words.len() as f64
    }

    /// Get distribution of guess counts across all words
    pub fn benchmark_guess_distribution(&self) -> Vec<(usize, usize)> {
        let guess_counts: Vec<usize> = self
            .all_words
            .par_iter()
            .map(|target| {
                let mut solver = self.clone();
                let guesses = solver.solve_for_target(target);
                guesses.len()
            })
            .collect();

        let max_guesses = *guess_counts.iter().max().unwrap_or(&0);
        let mut distribution = vec![0usize; max_guesses + 1];

        for count in guess_counts {
            distribution[count] += 1;
        }

        distribution
            .into_iter()
            .enumerate()
            .filter(|(_, count)| *count > 0)
            .collect()
    }
}
