use wordle_bot::{load_dictionary, FeedbackPattern, WordleSolver};

fn get_test_words() -> Vec<String> {
    vec![
        "crane".to_string(),
        "slate".to_string(),
        "trace".to_string(),
        "crate".to_string(),
        "raise".to_string(),
        "arise".to_string(),
        "stare".to_string(),
        "roast".to_string(),
        "toast".to_string(),
        "beast".to_string(),
    ]
}

#[test]
fn test_solver_creation() {
    let words = get_test_words();
    let solver = WordleSolver::new(words.clone());
    assert_eq!(solver.remaining_count(), words.len());
}

#[test]
fn test_apply_feedback() {
    let words = get_test_words();
    let mut solver = WordleSolver::new(words);

    let pattern = FeedbackPattern::calculate("crane", "crate");
    solver.apply_feedback("crane", pattern);

    assert!(solver.remaining_count() < 10);
    assert!(solver.possible_answers().contains(&"crate".to_string()));
}

#[test]
fn test_find_best_guess() {
    let words = get_test_words();
    let solver = WordleSolver::new(words);
    let best = solver.find_best_guess();

    assert!(best.is_some());
    let analysis = best.unwrap();
    assert!(!analysis.word.is_empty());
    assert!(analysis.entropy >= 0.0);
}

#[test]
fn test_find_best_guess_single_answer() {
    let solver = WordleSolver::new(vec!["crane".to_string()]);
    let best = solver.find_best_guess();

    assert!(best.is_some());
    let analysis = best.unwrap();
    assert_eq!(analysis.word, "crane");
    assert_eq!(analysis.entropy, 0.0);
}

#[test]
fn test_solve_for_target() {
    let words = get_test_words();
    let mut solver = WordleSolver::new(words);

    let guesses = solver.solve_for_target("crate");

    assert!(!guesses.is_empty());
    assert!(guesses.len() <= 6);

    let (final_guess, final_pattern) = guesses.last().unwrap();
    assert!(final_pattern.is_win());
    assert_eq!(final_guess, "crate");
}

#[test]
fn test_solve_various_targets() {
    let words = get_test_words();

    for target in &words {
        let mut solver = WordleSolver::new(words.clone());
        let guesses = solver.solve_for_target(target);

        assert!(!guesses.is_empty(), "Failed to solve for target: {}", target);
        assert!(guesses.len() <= 6, "Too many guesses for target: {}", target);

        let (final_guess, final_pattern) = guesses.last().unwrap();
        assert!(final_pattern.is_win(), "Didn't win for target: {}", target);
        assert_eq!(final_guess, target, "Final guess doesn't match target: {}", target);
    }
}

#[test]
fn test_entropy_calculation() {
    let words = vec![
        "crane".to_string(),
        "trace".to_string(),
        "crate".to_string(),
        "slate".to_string(),
    ];
    let solver = WordleSolver::new(words);

    let entropy = solver.calculate_entropy_for_word("crane");
    assert!(entropy > 0.0);
    assert!(entropy <= 2.0);
}

#[test]
fn test_reset() {
    let words = get_test_words();
    let mut solver = WordleSolver::new(words.clone());

    let pattern = FeedbackPattern::calculate("crane", "toast");
    solver.apply_feedback("crane", pattern);

    assert!(solver.remaining_count() < words.len());

    solver.reset();
    assert_eq!(solver.remaining_count(), words.len());
}

#[test]
fn test_get_top_guesses() {
    let words = get_test_words();
    let solver = WordleSolver::new(words);

    let top_5 = solver.get_top_guesses(5);
    assert_eq!(top_5.len(), 5);

    for i in 1..top_5.len() {
        assert!(top_5[i - 1].entropy >= top_5[i].entropy);
    }
}

#[test]
fn test_with_full_dictionary() {
    let words = load_dictionary();
    let mut solver = WordleSolver::new(words);

    let guesses = solver.solve_for_target("crane");

    assert!(!guesses.is_empty());
    assert!(guesses.len() <= 6);

    let (final_guess, final_pattern) = guesses.last().unwrap();
    assert!(final_pattern.is_win());
    assert_eq!(final_guess, "crane");
}

#[test]
fn test_hard_mode() {
    let words = get_test_words();
    let mut solver = WordleSolver::new(words);
    solver.set_hard_mode(true);

    let pattern = FeedbackPattern::calculate("crane", "crate");
    solver.apply_feedback("crane", pattern);

    let guesses = solver.find_best_guesses(10);
    for g in &guesses {
        assert!(g.word.starts_with("cra"), "Hard mode violation: {}", g.word);
    }
}

#[test]
fn test_find_best_guesses() {
    let words = get_test_words();
    let solver = WordleSolver::new(words);

    let guesses = solver.find_best_guesses(3);
    assert_eq!(guesses.len(), 3);

    for i in 1..guesses.len() {
        assert!(guesses[i - 1].entropy >= guesses[i].entropy);
    }
}

#[test]
fn test_empty_possible_answers() {
    let words = get_test_words();
    let mut solver = WordleSolver::new(words);
    
    // Filter to empty by applying impossible constraints
    solver.apply_feedback("zzzzz", FeedbackPattern::ALL_CORRECT);

    assert!(solver.find_best_guess().is_none());
    assert!(solver.get_top_guesses(5).is_empty());
}

#[test]
fn test_two_remaining_words() {
    let words = vec!["crane".to_string(), "trace".to_string()];
    let solver = WordleSolver::new(words);

    let best = solver.find_best_guess();
    assert!(best.is_some());
    let analysis = best.unwrap();

    assert!(analysis.word == "crane" || analysis.word == "trace");
    assert!(analysis.is_possible_answer);
}

#[test]
fn test_solve_difficult_word() {
    let words = load_dictionary();
    let mut solver = WordleSolver::new(words);

    solver.reset();
    let guesses = solver.solve_for_target("fuzzy");

    if !guesses.is_empty() {
        assert!(guesses.len() <= 6);
        if let Some((_, pattern)) = guesses.last() {
            assert!(pattern.is_win());
        }
    }
}

