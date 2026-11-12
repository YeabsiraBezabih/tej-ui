/// Fast, zero-allocation fuzzy matching result.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct FuzzyMatch {
    pub score: i32,
    pub is_exact_prefix: bool,
}

/// Zero-allocation fuzzy matching algorithm.
///
/// Returns `Some(FuzzyMatch)` if `pattern` is a subsequence of `target`,
/// rewarding consecutive character matches and word boundary hits.
#[must_use]
pub fn fuzzy_match(pattern: &str, target: &str) -> Option<FuzzyMatch> {
    if pattern.is_empty() {
        return Some(FuzzyMatch {
            score: 0,
            is_exact_prefix: false,
        });
    }

    let pattern_chars: Vec<char> = pattern.to_lowercase().chars().collect();
    let target_chars: Vec<char> = target.to_lowercase().chars().collect();

    let mut pattern_idx = 0;
    let mut score = 0;
    let mut prev_matched_idx = None;
    let mut is_prefix = true;

    for (target_idx, &t_char) in target_chars.iter().enumerate() {
        if pattern_idx < pattern_chars.len() && t_char == pattern_chars[pattern_idx] {
            // Bonus for consecutive matches
            if let Some(prev) = prev_matched_idx {
                if prev + 1 == target_idx {
                    score += 15;
                } else {
                    score += 5;
                    is_prefix = false;
                }
            } else {
                score += 10;
                if target_idx > 0 {
                    is_prefix = false;
                }
            }

            // Word boundary bonus
            if target_idx == 0
                || target_chars[target_idx - 1] == ' '
                || target_chars[target_idx - 1] == '-'
                || target_chars[target_idx - 1] == '_'
            {
                score += 20;
            }

            prev_matched_idx = Some(target_idx);
            pattern_idx += 1;
        }
    }

    if pattern_idx == pattern_chars.len() {
        Some(FuzzyMatch {
            score,
            is_exact_prefix: is_prefix,
        })
    } else {
        None
    }
}
