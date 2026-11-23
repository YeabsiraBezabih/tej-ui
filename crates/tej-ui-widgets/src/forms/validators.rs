//! Pre-built validation combinators for form fields.

use std::sync::Arc;

/// Validator closure type returning an optional error string if validation fails.
pub type ValidatorFn<T> = Arc<dyn Fn(&T) -> Option<String> + Send + Sync>;

/// Validates that a string is non-empty after trimming whitespace.
#[must_use]
pub fn required(error_msg: impl Into<String>) -> ValidatorFn<String> {
    let msg = error_msg.into();
    Arc::new(move |val: &String| {
        if val.trim().is_empty() {
            Some(msg.clone())
        } else {
            None
        }
    })
}

/// Validates that an optional field has `Some` value.
#[must_use]
pub fn required_option<T: Send + Sync + 'static>(
    error_msg: impl Into<String>,
) -> ValidatorFn<Option<T>> {
    let msg = error_msg.into();
    Arc::new(move |val: &Option<T>| {
        if val.is_none() {
            Some(msg.clone())
        } else {
            None
        }
    })
}

/// Validates that a string's character length is at least `min`.
#[must_use]
pub fn min_length(min: usize, error_msg: impl Into<String>) -> ValidatorFn<String> {
    let msg = error_msg.into();
    Arc::new(move |val: &String| {
        let trimmed = val.trim();
        if !trimmed.is_empty() && trimmed.len() < min {
            Some(msg.clone())
        } else {
            None
        }
    })
}

/// Validates that a string's character length is at most `max`.
#[must_use]
pub fn max_length(max: usize, error_msg: impl Into<String>) -> ValidatorFn<String> {
    let msg = error_msg.into();
    Arc::new(move |val: &String| {
        if val.len() > max {
            Some(msg.clone())
        } else {
            None
        }
    })
}

/// Validates standard email address syntax (`user@domain.tld`).
#[must_use]
pub fn email(error_msg: impl Into<String>) -> ValidatorFn<String> {
    let msg = error_msg.into();
    Arc::new(move |val: &String| {
        let trimmed = val.trim();
        if trimmed.is_empty() {
            return None; // Blank values deferred to required() validator
        }
        let parts: Vec<&str> = trimmed.split('@').collect();
        let is_valid = parts.len() == 2
            && !parts[0].is_empty()
            && parts[1].contains('.')
            && !parts[1].starts_with('.')
            && !parts[1].ends_with('.');

        if is_valid { None } else { Some(msg.clone()) }
    })
}

/// Validates that an integer or float is within inclusive bounds `[min, max]`.
#[must_use]
pub fn numeric_range<T: PartialOrd + Copy + Send + Sync + 'static>(
    min: T,
    max: T,
    error_msg: impl Into<String>,
) -> ValidatorFn<T> {
    let msg = error_msg.into();
    Arc::new(move |val: &T| {
        if *val < min || *val > max {
            Some(msg.clone())
        } else {
            None
        }
    })
}
