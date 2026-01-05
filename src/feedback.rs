//! Feedback calculation for Wordle guesses.
//!
//! This module handles computing the feedback pattern (green/yellow/gray)
//! for a guess against a target word.

use crate::WORD_LENGTH;

/// Represents the feedback for a single letter position.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Feedback {
    /// Correct letter in correct position (green)
    Correct,
    /// Correct letter in wrong position (yellow)
    Present,
    /// Letter not in word (gray)
    Absent,
}

impl Feedback {
    /// Convert to a character for display
    pub fn to_char(self) -> char {
        match self {
            Feedback::Correct => '🟩',
            Feedback::Present => '🟨',
            Feedback::Absent => '⬛',
        }
    }

    /// Parse from a character (g=green, y=yellow, b=black/gray)
    pub fn from_char(c: char) -> Option<Self> {
        match c.to_ascii_lowercase() {
            'g' | '2' => Some(Feedback::Correct),
            'y' | '1' => Some(Feedback::Present),
            'b' | 'x' | '0' => Some(Feedback::Absent),
            _ => None,
        }
    }
}

/// A complete feedback pattern for a 5-letter guess.
/// Encoded as a single u8 value (0-242) for efficiency.
/// Each position can be 0 (absent), 1 (present), or 2 (correct).
/// Pattern = p0 + 3*p1 + 9*p2 + 27*p3 + 81*p4
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FeedbackPattern(pub u8);

impl FeedbackPattern {
    /// The pattern indicating all correct (winning)
    pub const ALL_CORRECT: Self = Self(2 + 2 * 3 + 2 * 9 + 2 * 27 + 2 * 81); // 242

    /// Total number of possible patterns (3^5)
    pub const NUM_PATTERNS: usize = 243;

    /// Create a new pattern from individual feedback values
    pub fn new(feedbacks: [Feedback; WORD_LENGTH]) -> Self {
        let mut pattern: u8 = 0;
        let mut multiplier: u8 = 1;
        for fb in feedbacks {
            let value = match fb {
                Feedback::Absent => 0,
                Feedback::Present => 1,
                Feedback::Correct => 2,
            };
            pattern += value * multiplier;
            multiplier *= 3;
        }
        Self(pattern)
    }

    /// Calculate the feedback pattern for a guess against a target word.
    ///
    /// This implements the standard Wordle feedback rules:
    /// - Green (Correct): Letter is in the correct position
    /// - Yellow (Present): Letter is in the word but wrong position
    /// - Gray (Absent): Letter is not in the word (or all instances accounted for)
    pub fn calculate(guess: &str, target: &str) -> Self {
        let guess_bytes = guess.as_bytes();
        let target_bytes = target.as_bytes();

        debug_assert_eq!(guess_bytes.len(), WORD_LENGTH);
        debug_assert_eq!(target_bytes.len(), WORD_LENGTH);

        let mut feedback = [Feedback::Absent; WORD_LENGTH];
        let mut target_remaining = [0u8; 26];

        for i in 0..WORD_LENGTH {
            if guess_bytes[i] == target_bytes[i] {
                feedback[i] = Feedback::Correct;
            } else {
                let idx = (target_bytes[i] - b'a') as usize;
                target_remaining[idx] += 1;
            }
        }

        for i in 0..WORD_LENGTH {
            if feedback[i] != Feedback::Correct {
                let idx = (guess_bytes[i] - b'a') as usize;
                if target_remaining[idx] > 0 {
                    feedback[i] = Feedback::Present;
                    target_remaining[idx] -= 1;
                }
            }
        }

        Self::new(feedback)
    }

    /// Convert pattern to array of feedbacks
    pub fn to_feedbacks(self) -> [Feedback; WORD_LENGTH] {
        let mut pattern = self.0;
        let mut feedbacks = [Feedback::Absent; WORD_LENGTH];
        for feedback in feedbacks.iter_mut() {
            *feedback = match pattern % 3 {
                0 => Feedback::Absent,
                1 => Feedback::Present,
                2 => Feedback::Correct,
                _ => unreachable!(),
            };
            pattern /= 3;
        }
        feedbacks
    }

    /// Check if this pattern represents a win (all correct)
    pub fn is_win(self) -> bool {
        self == Self::ALL_CORRECT
    }

    /// Parse a pattern from a string like "gybbb" or "21000"
    pub fn parse(s: &str) -> Option<Self> {
        if s.len() != WORD_LENGTH {
            return None;
        }
        let feedbacks: Option<Vec<_>> = s.chars().map(Feedback::from_char).collect();
        let feedbacks = feedbacks?;
        let arr: [Feedback; WORD_LENGTH] = feedbacks.try_into().ok()?;
        Some(Self::new(arr))
    }

    /// Display as emoji string
    pub fn to_emoji_string(self) -> String {
        self.to_feedbacks().iter().map(|f| f.to_char()).collect()
    }
}

impl std::fmt::Display for FeedbackPattern {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_emoji_string())
    }
}
