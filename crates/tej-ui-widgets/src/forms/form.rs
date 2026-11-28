//! Reactive form field controller and validation coordinator.

use crate::forms::validators::ValidatorFn;
use std::fmt::Debug;
use std::sync::Arc;
use tej_ui_core::Signal;

/// Strongly typed reactive form field tracking value, dirty, touched, and validation errors.
#[derive(Clone)]
pub struct FormField<T: Clone + Send + Sync + 'static> {
    name: String,
    value: Signal<T>,
    dirty: Signal<bool>,
    touched: Signal<bool>,
    errors: Signal<Vec<String>>,
    validators: Vec<ValidatorFn<T>>,
}

impl<T: Clone + Debug + Send + Sync + 'static> Debug for FormField<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FormField")
            .field("name", &self.name)
            .field("value", &self.value.get())
            .field("dirty", &self.dirty.get())
            .field("touched", &self.touched.get())
            .field("errors", &self.errors.get())
            .finish_non_exhaustive()
    }
}

impl<T: Clone + Send + Sync + 'static> FormField<T> {
    /// Creates a new form field with an initial value.
    pub fn new(name: impl Into<String>, initial_value: T) -> Self {
        Self {
            name: name.into(),
            value: Signal::new(initial_value),
            dirty: Signal::new(false),
            touched: Signal::new(false),
            errors: Signal::new(Vec::new()),
            validators: Vec::new(),
        }
    }

    /// Attaches a validation predicate to the field.
    #[must_use]
    pub fn validator(mut self, validator: ValidatorFn<T>) -> Self {
        self.validators.push(validator);
        self
    }

    /// Read the underlying reactive signal for the field value.
    #[must_use]
    pub const fn signal(&self) -> Signal<T> {
        self.value
    }

    /// Returns the current value.
    #[must_use]
    pub fn get(&self) -> T {
        self.value.get()
    }

    /// Updates the field value, marking it dirty and executing validation.
    pub fn set(&self, new_value: T) {
        self.value.set(new_value);
        self.dirty.set(true);
        self.validate();
    }

    /// Marks the field as touched (e.g. after blur or interaction).
    pub fn touch(&self) {
        self.touched.set(true);
        self.validate();
    }

    /// Runs all attached validators, updating the error list.
    ///
    /// Returns `true` if all validations passed.
    pub fn validate(&self) -> bool {
        let current_val = self.value.get();
        let mut errs = Vec::new();
        for v in &self.validators {
            if let Some(err) = v(&current_val) {
                errs.push(err);
            }
        }
        let is_valid = errs.is_empty();
        self.errors.set(errs);
        is_valid
    }

    /// Returns `true` if the field currently has zero validation errors.
    #[must_use]
    pub fn is_valid(&self) -> bool {
        self.errors.get().is_empty()
    }

    /// Returns `true` if the field value has been modified.
    #[must_use]
    pub fn is_dirty(&self) -> bool {
        self.dirty.get()
    }

    /// Returns `true` if the field has been touched/focused.
    #[must_use]
    pub fn is_touched(&self) -> bool {
        self.touched.get()
    }

    /// Returns the list of active validation errors.
    #[must_use]
    pub fn errors(&self) -> Vec<String> {
        self.errors.get()
    }

    /// Returns the field name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Resets the field to a clean, untouched state with a specified value.
    pub fn reset(&self, value: T) {
        self.value.set(value);
        self.dirty.set(false);
        self.touched.set(false);
        self.errors.set(Vec::new());
    }
}

/// Orchestrates multi-field form validation and submit lifecycles.
#[derive(Default, Clone)]
pub struct FormController {
    validators: Vec<Arc<dyn Fn() -> bool + Send + Sync>>,
    dirty_checks: Vec<Arc<dyn Fn() -> bool + Send + Sync>>,
}

impl Debug for FormController {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FormController")
            .field("registered_fields", &self.validators.len())
            .finish_non_exhaustive()
    }
}

impl FormController {
    /// Creates an empty form controller.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers a field with the controller for batch validation.
    pub fn register<T: Clone + Send + Sync + 'static>(&mut self, field: &FormField<T>) {
        let f_val = field.clone();
        self.validators.push(Arc::new(move || f_val.validate()));
        let f_dirty = field.clone();
        self.dirty_checks.push(Arc::new(move || f_dirty.is_dirty()));
    }

    /// Executes validation across all registered fields.
    ///
    /// Returns `true` if every field is valid.
    pub fn validate_all(&self) -> bool {
        let mut all_valid = true;
        for validator in &self.validators {
            if !validator() {
                all_valid = false;
            }
        }
        all_valid
    }

    /// Returns `true` if any registered field has been modified.
    #[must_use]
    pub fn is_dirty(&self) -> bool {
        self.dirty_checks.iter().any(|check| check())
    }

    /// Attempts form submission: executes `validate_all()`, and if successful,
    /// invokes `on_success`. Returns `true` if submission succeeded.
    pub fn submit<F: FnOnce()>(&self, on_success: F) -> bool {
        if self.validate_all() {
            on_success();
            true
        } else {
            false
        }
    }
}

/// A form field paired with an `InputMask` for formatted text inputs.
#[derive(Clone, Debug)]
pub struct MaskedFormField {
    field: FormField<String>,
    mask: crate::forms::mask::InputMask,
}

impl MaskedFormField {
    /// Create a new masked form field.
    #[must_use]
    pub fn new(field: FormField<String>, mask: crate::forms::mask::InputMask) -> Self {
        let current_val = field.get();
        let formatted = mask.format(&current_val);
        field.value.set(formatted);
        Self { field, mask }
    }

    /// Underlying form field.
    #[must_use]
    pub fn field(&self) -> &FormField<String> {
        &self.field
    }

    /// Associated input mask.
    #[must_use]
    pub fn mask(&self) -> &crate::forms::mask::InputMask {
        &self.mask
    }

    /// Read formatted masked value.
    #[must_use]
    pub fn formatted_value(&self) -> String {
        self.field.get()
    }

    /// Read unmasked raw alphanumeric value.
    #[must_use]
    pub fn raw_value(&self) -> String {
        self.mask.unmask(&self.field.get())
    }

    /// Update with raw unmasked text, which is automatically masked.
    pub fn set_raw(&self, raw: &str) {
        let formatted = self.mask.format(raw);
        self.field.set(formatted);
    }

    /// Check if all required placeholder positions in the mask are filled.
    #[must_use]
    pub fn is_complete(&self) -> bool {
        self.mask.is_complete(&self.field.get())
    }
}

impl FormField<String> {
    /// Wrap this string form field with an input mask.
    #[must_use]
    pub fn with_mask(self, mask: crate::forms::mask::InputMask) -> MaskedFormField {
        MaskedFormField::new(self, mask)
    }
}
