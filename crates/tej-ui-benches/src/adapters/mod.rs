//! Framework adapters implementing `GuiBenchHarness`.

pub mod dioxus_adapter;
pub mod egui_adapter;
pub mod gpui_adapter;
pub mod iced_adapter;
pub mod slint_adapter;
pub mod tej_adapter;

pub use dioxus_adapter::DioxusAdapter;
pub use egui_adapter::EguiAdapter;
pub use gpui_adapter::GpuiAdapter;
pub use iced_adapter::IcedAdapter;
pub use slint_adapter::SlintAdapter;
pub use tej_adapter::TejAdapter;
