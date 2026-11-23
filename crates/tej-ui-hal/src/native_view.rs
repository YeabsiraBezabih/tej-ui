//! Native view embedding escape-hatch FFI (HWND, NSView, Wayland Subsurface).

use tej_ui_layout::Rect;

/// Raw OS native view / window handle variant.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NativeViewHandle {
    /// Win32 window handle (`HWND`).
    Win32Hwnd(isize),
    /// macOS AppKit `NSView*` pointer identifier.
    AppKitNsView(usize),
    /// Wayland wl_subsurface ID.
    WaylandSubsurface(u32),
    /// WebAssembly DOM HTMLElement element ID.
    WasmDomElementId(u32),
}

/// Host controller managing the layout synchronization, clipping, and lifecycle of an embedded native view.
#[derive(Debug, Clone)]
pub struct NativeViewHost {
    handle: NativeViewHandle,
    bounds: Rect,
    clip_rect: Option<Rect>,
    is_visible: bool,
}

impl NativeViewHost {
    pub fn new(handle: NativeViewHandle, bounds: Rect) -> Self {
        Self {
            handle,
            bounds,
            clip_rect: None,
            is_visible: true,
        }
    }

    /// Native handle.
    pub fn handle(&self) -> NativeViewHandle {
        self.handle
    }

    /// Layout bounds.
    pub fn bounds(&self) -> Rect {
        self.bounds
    }

    /// Updates layout geometry and returns true if geometry changed.
    pub fn update_bounds(&mut self, bounds: Rect) -> bool {
        if self.bounds != bounds {
            self.bounds = bounds;
            true
        } else {
            false
        }
    }

    /// Sets or clears the clipping rectangle.
    pub fn set_clip_rect(&mut self, clip: Option<Rect>) {
        self.clip_rect = clip;
    }

    /// Active clipping rectangle.
    pub fn clip_rect(&self) -> Option<Rect> {
        self.clip_rect
    }

    /// Sets visibility state.
    pub fn set_visible(&mut self, visible: bool) {
        self.is_visible = visible;
    }

    /// Current visibility.
    pub fn is_visible(&self) -> bool {
        self.is_visible
    }
}
