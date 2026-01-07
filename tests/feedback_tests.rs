use wordle_bot::{Feedback, FeedbackPattern};

#[test]
fn test_all_correct() {
    let pattern = FeedbackPattern::calculate("crane", "crane");
    assert!(pattern.is_win());
    assert_eq!(pattern, FeedbackPattern::ALL_CORRECT);
}

#[test]
fn test_all_absent() {
    let pattern = FeedbackPattern::calculate("quick", "dream");
    let expected = FeedbackPattern::new([
        Feedback::Absent,
        Feedback::Absent,
        Feedback::Absent,
        Feedback::Absent,
        Feedback::Absent,
    ]);
    assert_eq!(pattern, expected);
}

#[test]
fn test_mixed_feedback() {
    let pattern = FeedbackPattern::calculate("crane", "charm");
    let feedbacks = pattern.to_feedbacks();
    assert_eq!(feedbacks[0], Feedback::Correct);
    assert_eq!(feedbacks[1], Feedback::Present);
    assert_eq!(feedbacks[2], Feedback::Correct);
    assert_eq!(feedbacks[3], Feedback::Absent);
    assert_eq!(feedbacks[4], Feedback::Absent);
}

#[test]
fn test_duplicate_letters_in_guess() {
    let pattern = FeedbackPattern::calculate("speed", "creep");
    let feedbacks = pattern.to_feedbacks();
    assert_eq!(feedbacks[0], Feedback::Absent);
    assert_eq!(feedbacks[1], Feedback::Present);
    assert_eq!(feedbacks[2], Feedback::Correct);
    assert_eq!(feedbacks[3], Feedback::Correct);
    assert_eq!(feedbacks[4], Feedback::Absent);
}

#[test]
fn test_duplicate_letters_in_target() {
    let pattern = FeedbackPattern::calculate("arose", "creep");
    let feedbacks = pattern.to_feedbacks();
    assert_eq!(feedbacks[0], Feedback::Absent);
    assert_eq!(feedbacks[1], Feedback::Correct);
    assert_eq!(feedbacks[2], Feedback::Absent);
    assert_eq!(feedbacks[3], Feedback::Absent);
    assert_eq!(feedbacks[4], Feedback::Present);
}

#[test]
fn test_duplicate_guess_limited_target() {
    let pattern = FeedbackPattern::calculate("geese", "creep");
    let feedbacks = pattern.to_feedbacks();
    assert_eq!(feedbacks[0], Feedback::Absent);
    assert_eq!(feedbacks[1], Feedback::Present);
    assert_eq!(feedbacks[2], Feedback::Correct);
    assert_eq!(feedbacks[3], Feedback::Absent);
    assert_eq!(feedbacks[4], Feedback::Absent);
}

#[test]
fn test_pattern_encoding_decoding() {
    for pattern_val in 0..FeedbackPattern::NUM_PATTERNS {
        let pattern = FeedbackPattern(pattern_val as u8);
        let feedbacks = pattern.to_feedbacks();
        let reconstructed = FeedbackPattern::new(feedbacks);
        assert_eq!(pattern, reconstructed);
    }
}

#[test]
fn test_pattern_parse() {
    let pattern = FeedbackPattern::parse("gybbb").unwrap();
    let feedbacks = pattern.to_feedbacks();
    assert_eq!(feedbacks[0], Feedback::Correct);
    assert_eq!(feedbacks[1], Feedback::Present);
    assert_eq!(feedbacks[2], Feedback::Absent);
    assert_eq!(feedbacks[3], Feedback::Absent);
    assert_eq!(feedbacks[4], Feedback::Absent);

    let pattern2 = FeedbackPattern::parse("21000").unwrap();
    assert_eq!(pattern, pattern2);
}

#[test]
fn test_pattern_parse_invalid() {
    assert!(FeedbackPattern::parse("gybbb1").is_none());
    assert!(FeedbackPattern::parse("gybb").is_none());
    assert!(FeedbackPattern::parse("gybzb").is_none());
}

#[test]
fn test_emoji_display() {
    let pattern = FeedbackPattern::new([
        Feedback::Correct,
        Feedback::Present,
        Feedback::Absent,
        Feedback::Absent,
        Feedback::Correct,
    ]);
    assert_eq!(pattern.to_emoji_string(), "🟩🟨⬛⬛🟩");
}

#[test]
fn test_specific_wordle_cases() {
    let pattern = FeedbackPattern::calculate("sores", "those");
    let feedbacks = pattern.to_feedbacks();
    assert_eq!(feedbacks[0], Feedback::Present);
    assert_eq!(feedbacks[1], Feedback::Present);
    assert_eq!(feedbacks[2], Feedback::Absent);
    assert_eq!(feedbacks[3], Feedback::Present);
    assert_eq!(feedbacks[4], Feedback::Absent);
}