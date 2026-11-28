//! Type-safe router, route guards, and custom URL scheme deep linking.
//!
//! Provides declarative path matching with parameter extraction, navigation history,
//! route guards (e.g. auth checks), and custom protocol (`myapp://`) handling.

use parking_lot::RwLock;
use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::Arc;
use tej_ui_core::Signal;

/// Route guard result: allowed or redirected to an alternative route.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GuardResult {
    /// Navigation is permitted.
    Allow,
    /// Navigation is blocked and redirected to `redirect_path`.
    Redirect(String),
}

/// Route guard predicate inspecting target path and route parameters.
pub type RouteGuard = Arc<dyn Fn(&str, &HashMap<String, String>) -> GuardResult + Send + Sync>;

/// Matched route parameters and query arguments extracted from a URL path.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RouteMatch {
    /// Raw path that was matched.
    pub matched_path: String,
    /// Dynamic path parameters (e.g. `:id` -> `"42"`).
    pub params: HashMap<String, String>,
    /// Query string parameters (e.g. `?tab=security`).
    pub query: HashMap<String, String>,
}

/// A registered route definition with pattern matching and optional guards.
#[derive(Clone)]
pub struct RouteDefinition<T> {
    pub pattern: String,
    pub handler: Arc<dyn Fn(&RouteMatch) -> T + Send + Sync>,
    pub guards: Vec<RouteGuard>,
}

impl<T> Debug for RouteDefinition<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RouteDefinition")
            .field("pattern", &self.pattern)
            .field("guards_count", &self.guards.len())
            .finish_non_exhaustive()
    }
}

/// Central application router coordinating URL paths, history, and route dispatch.
#[derive(Clone)]
pub struct Router<T: Clone + Send + Sync + 'static> {
    current_path: Signal<String>,
    history: Arc<RwLock<Vec<String>>>,
    history_index: Arc<RwLock<usize>>,
    custom_scheme: Option<String>,
    routes: Arc<RwLock<Vec<RouteDefinition<T>>>>,
    fallback: Arc<dyn Fn(&str) -> T + Send + Sync>,
}

impl<T: Clone + Send + Sync + 'static> Debug for Router<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Router")
            .field("current_path", &self.current_path.get())
            .field("history_len", &self.history.read().len())
            .field("routes_count", &self.routes.read().len())
            .field("custom_scheme", &self.custom_scheme)
            .finish_non_exhaustive()
    }
}

impl<T: Clone + Send + Sync + 'static> Router<T> {
    /// Creates a new router with an initial path and a fallback 404 handler.
    pub fn new(
        initial_path: impl Into<String>,
        fallback: impl Fn(&str) -> T + Send + Sync + 'static,
    ) -> Self {
        let path = initial_path.into();
        Self {
            current_path: Signal::new(path.clone()),
            history: Arc::new(RwLock::new(vec![path])),
            history_index: Arc::new(RwLock::new(0)),
            custom_scheme: None,
            routes: Arc::new(RwLock::new(Vec::new())),
            fallback: Arc::new(fallback),
        }
    }

    /// Configures an application custom URL scheme (e.g. `"myapp"` for `myapp://...`).
    #[must_use]
    pub fn with_custom_scheme(mut self, scheme: impl Into<String>) -> Self {
        self.custom_scheme = Some(scheme.into());
        self
    }

    /// Registers a route pattern with a view/action handler and optional guards.
    pub fn add_route(
        &self,
        pattern: impl Into<String>,
        guards: Vec<RouteGuard>,
        handler: impl Fn(&RouteMatch) -> T + Send + Sync + 'static,
    ) {
        let mut routes = self.routes.write();
        routes.push(RouteDefinition {
            pattern: pattern.into(),
            handler: Arc::new(handler),
            guards,
        });
    }

    /// Reactive signal of the currently active path.
    #[must_use]
    pub const fn path_signal(&self) -> Signal<String> {
        self.current_path
    }

    /// Current route path string.
    #[must_use]
    pub fn current_path(&self) -> String {
        self.current_path.get()
    }

    /// Navigate to a new path, evaluating route guards and recording history.
    pub fn navigate(&self, target_path: &str) -> bool {
        let clean_path = Self::normalize_path(target_path);

        // Check guards against matching route
        let routes = self.routes.read();
        for route in routes.iter() {
            if let Some(m) = Self::match_pattern(&route.pattern, &clean_path) {
                for guard in &route.guards {
                    if let GuardResult::Redirect(redirect_to) = guard(&clean_path, &m.params) {
                        drop(routes);
                        return self.navigate(&redirect_to);
                    }
                }
                break;
            }
        }
        drop(routes);

        // Update history
        let mut hist = self.history.write();
        let mut idx = self.history_index.write();
        hist.truncate(*idx + 1);
        hist.push(clean_path.clone());
        *idx = hist.len() - 1;

        self.current_path.set(clean_path);
        true
    }

    /// Handle an incoming deep link URL (e.g. `"myapp://profile/123?tab=info"`).
    pub fn handle_deep_link(&self, url: &str) -> bool {
        let path = if let Some(ref scheme) = self.custom_scheme {
            let prefix = format!("{scheme}://");
            if let Some(stripped) = url.strip_prefix(&prefix) {
                format!("/{stripped}")
            } else {
                url.to_string()
            }
        } else {
            url.to_string()
        };

        self.navigate(&path)
    }

    /// Go back in navigation history if available.
    pub fn back(&self) -> bool {
        let mut idx = self.history_index.write();
        if *idx > 0 {
            *idx -= 1;
            let hist = self.history.read();
            let prev_path = hist[*idx].clone();
            self.current_path.set(prev_path);
            true
        } else {
            false
        }
    }

    /// Resolve and render the active route.
    #[must_use]
    pub fn resolve_current(&self) -> T {
        let path = self.current_path.get();
        let routes = self.routes.read();
        for route in routes.iter() {
            if let Some(m) = Self::match_pattern(&route.pattern, &path) {
                return (route.handler)(&m);
            }
        }
        (self.fallback)(&path)
    }

    fn normalize_path(raw: &str) -> String {
        let trimmed = raw.trim();
        if trimmed.starts_with('/') {
            trimmed.to_string()
        } else {
            format!("/{trimmed}")
        }
    }

    /// Matches a pattern like `"/users/:id"` against a concrete path `"/users/42?tab=info"`.
    #[must_use]
    pub fn match_pattern(pattern: &str, path: &str) -> Option<RouteMatch> {
        let (path_part, query_part) = match path.split_once('?') {
            Some((p, q)) => (p, Some(q)),
            None => (path, None),
        };

        let pat_segments: Vec<&str> = pattern.split('/').filter(|s| !s.is_empty()).collect();
        let path_segments: Vec<&str> = path_part.split('/').filter(|s| !s.is_empty()).collect();

        if pat_segments.len() != path_segments.len() {
            return None;
        }

        let mut params = HashMap::new();
        for (pat, val) in pat_segments.iter().zip(path_segments.iter()) {
            if let Some(param_name) = pat.strip_prefix(':') {
                params.insert(param_name.to_string(), (*val).to_string());
            } else if pat != val {
                return None;
            }
        }

        let mut query = HashMap::new();
        if let Some(q) = query_part {
            for pair in q.split('&') {
                if let Some((k, v)) = pair.split_once('=') {
                    query.insert(k.to_string(), v.to_string());
                } else if !pair.is_empty() {
                    query.insert(pair.to_string(), String::new());
                }
            }
        }

        Some(RouteMatch {
            matched_path: path_part.to_string(),
            params,
            query,
        })
    }
}
