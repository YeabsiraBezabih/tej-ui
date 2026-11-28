use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tej_ui_widgets::{
    FormController, FormField, email, max_length, min_length, numeric_range, required,
    required_option,
};

#[test]
fn test_form_field_validation_lifecycle() {
    let username = FormField::new("username", String::new())
        .validator(required("Username is required"))
        .validator(min_length(3, "Must be at least 3 characters"))
        .validator(max_length(10, "Must be at most 10 characters"));

    assert_eq!(username.name(), "username");
    assert!(!username.is_dirty());
    assert!(!username.is_touched());

    // Initially validate empty field
    assert!(!username.validate());
    assert!(!username.is_valid());
    assert_eq!(username.errors(), vec!["Username is required".to_string()]);

    // Set invalid short value
    username.set("al".into());
    assert!(username.is_dirty());
    assert!(!username.is_valid());
    assert_eq!(
        username.errors(),
        vec!["Must be at least 3 characters".to_string()]
    );

    // Set valid value
    username.set("alice".into());
    assert!(username.is_valid());
    assert_eq!(username.errors(), [] as [std::string::String; 0]);

    // Reset
    username.reset("bob".into());
    assert!(!username.is_dirty());
    assert!(!username.is_touched());
    assert_eq!(username.get(), "bob");
}

#[test]
fn test_email_and_range_validators() {
    let email_field =
        FormField::new("email", String::new()).validator(email("Invalid email address"));

    email_field.set("not-an-email".into());
    assert!(!email_field.is_valid());
    assert_eq!(
        email_field.errors(),
        vec!["Invalid email address".to_string()]
    );

    email_field.set("user@example.com".into());
    assert!(email_field.is_valid());

    let age_field = FormField::new("age", 15u32).validator(numeric_range(
        18,
        120,
        "Must be between 18 and 120",
    ));

    assert!(!age_field.validate());
    age_field.set(25);
    assert!(age_field.is_valid());

    let opt_field: FormField<Option<String>> =
        FormField::new("role", None).validator(required_option("Please select a role"));
    assert!(!opt_field.validate());
    opt_field.set(Some("Admin".into()));
    assert!(opt_field.is_valid());
}

#[test]
fn test_form_controller_batch_submit() {
    let mut controller = FormController::new();

    let name = FormField::new("name", String::new()).validator(required("Name required"));
    let email_f = FormField::new("email", String::new())
        .validator(required("Email required"))
        .validator(email("Email invalid"));

    controller.register(&name);
    controller.register(&email_f);

    assert!(!controller.is_dirty());

    name.set("Jane".into());
    assert!(controller.is_dirty());
    assert!(!controller.validate_all()); // email is still empty / not evaluated

    email_f.set("jane@corp.com".into());
    assert!(controller.validate_all());

    let submitted = Arc::new(AtomicBool::new(false));
    let s_clone = Arc::clone(&submitted);

    let success = controller.submit(move || {
        s_clone.store(true, Ordering::SeqCst);
    });

    assert!(success);
    assert!(submitted.load(Ordering::SeqCst));
}

#[test]
fn test_input_mask_formatting_and_form_field() {
    use tej_ui_widgets::{FormField, InputMask};

    // 1. Phone mask
    let phone_mask = InputMask::phone_us();
    assert_eq!(phone_mask.placeholder_count(), 10);
    assert_eq!(phone_mask.format("5551234567"), "(555) 123-4567");
    assert_eq!(phone_mask.unmask("(555) 123-4567"), "5551234567");
    assert!(phone_mask.is_complete("(555) 123-4567"));
    assert!(!phone_mask.is_complete("(555) 123"));

    // 2. Date ISO mask
    let date_mask = InputMask::date_iso();
    assert_eq!(date_mask.format("20261115"), "2026-11-15");
    assert_eq!(date_mask.unmask("2026-11-15"), "20261115");
    assert!(date_mask.is_complete("2026-11-15"));

    // 3. Custom pattern mask (e.g. product SKU AAA-####)
    let sku_mask = InputMask::parse("AAA-####");
    assert_eq!(sku_mask.format("XYZ9876"), "XYZ-9876");
    assert_eq!(sku_mask.unmask("XYZ-9876"), "XYZ9876");

    // 4. FormField integration with with_mask
    let raw_field = FormField::new("phone", String::new());
    let masked = raw_field.with_mask(InputMask::phone_us());

    masked.set_raw("4155552671");
    assert_eq!(masked.formatted_value(), "(415) 555-2671");
    assert_eq!(masked.raw_value(), "4155552671");
    assert!(masked.is_complete());
    assert!(masked.field().is_dirty());
}
