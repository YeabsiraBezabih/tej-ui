use std::sync::Arc;
use tej_ui_widgets::{GuardResult, Router};

#[test]
fn test_router_path_matching_and_param_extraction() {
    let router = Router::new("/", |path| format!("404: {path}"));

    router.add_route("/users/:id", vec![], |m| {
        format!("User: {}", m.params.get("id").unwrap())
    });

    router.add_route("/settings", vec![], |m| {
        let tab = m
            .query
            .get("tab")
            .cloned()
            .unwrap_or_else(|| "general".into());
        format!("Settings Tab: {tab}")
    });

    // 1. Initial route
    assert_eq!(router.current_path(), "/");
    assert_eq!(router.resolve_current(), "404: /");

    // 2. Dynamic route with parameter
    router.navigate("/users/42");
    assert_eq!(router.current_path(), "/users/42");
    assert_eq!(router.resolve_current(), "User: 42");

    // 3. Route with query parameter
    router.navigate("/settings?tab=security");
    assert_eq!(router.resolve_current(), "Settings Tab: security");

    // 4. Back navigation
    assert!(router.back());
    assert_eq!(router.current_path(), "/users/42");
    assert_eq!(router.resolve_current(), "User: 42");

    assert!(router.back());
    assert_eq!(router.current_path(), "/");
    assert!(!router.back()); // Cannot go back beyond initial
}

#[test]
fn test_router_guards_and_redirects() {
    let is_authenticated = Arc::new(std::sync::atomic::AtomicBool::new(false));
    let auth_clone = Arc::clone(&is_authenticated);

    let router = Router::new("/login", |path| format!("Page: {path}"));

    let auth_guard = Arc::new(move |_path: &str, _params: &_| {
        if auth_clone.load(std::sync::atomic::Ordering::SeqCst) {
            GuardResult::Allow
        } else {
            GuardResult::Redirect("/login".into())
        }
    });

    router.add_route("/dashboard", vec![auth_guard], |_| "Dashboard".into());
    router.add_route("/login", vec![], |_| "Login Page".into());

    // Try navigating to dashboard while unauthenticated -> should redirect to /login
    router.navigate("/dashboard");
    assert_eq!(router.current_path(), "/login");
    assert_eq!(router.resolve_current(), "Login Page");

    // Authenticate and navigate again -> should succeed
    is_authenticated.store(true, std::sync::atomic::Ordering::SeqCst);
    router.navigate("/dashboard");
    assert_eq!(router.current_path(), "/dashboard");
    assert_eq!(router.resolve_current(), "Dashboard");
}

#[test]
fn test_router_custom_scheme_deep_linking() {
    let router = Router::new("/", |path| format!("Default: {path}")).with_custom_scheme("obsidian");

    router.add_route("/open/:note", vec![], |m| {
        format!("Note: {}", m.params.get("note").unwrap())
    });

    // Deep link via custom URI scheme "obsidian://open/my-first-note"
    let handled = router.handle_deep_link("obsidian://open/my-first-note");
    assert!(handled);
    assert_eq!(router.current_path(), "/open/my-first-note");
    assert_eq!(router.resolve_current(), "Note: my-first-note");
}
