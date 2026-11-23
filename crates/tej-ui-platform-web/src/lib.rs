//! `tej-platform-web`: Browser WASM canvas windowing shell, WebGL/WebGPU compositor, and DOM bridge.

pub mod events;
pub mod surface;
pub mod window;

pub use events::WebEventBridge;
pub use surface::WebSurfaceCompositor;
pub use window::WebWindow;
