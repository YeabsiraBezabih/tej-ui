//! Production desktop GPU runner (`winit` 0.30 + `wgpu` 24 `GpuCompositor`) and `GpuSurfaceBridge`.
//!
//! Bridges `tej-ui-view`'s `SurfaceCompositor` trait directly onto `tej-ui-hal`'s hardware-accelerated
//! `GpuCompositor` (instanced SDF quads, `lyon`-tessellated vector strokes, and VRAM atlas glyphs)
//! inside an event-driven `winit::application::ApplicationHandler` running under `ControlFlow::Wait`
//! for 0.0% idle CPU usage.

#![cfg(all(feature = "native-gpu", not(target_arch = "wasm32")))]

use std::fmt::Debug;
use std::sync::Arc;

use tej_ui_core::{Epoch, ReactiveRuntime};
use tej_ui_hal::gpu::{
    AtlasFormat, BackendConfig, FrameStats, GlyphKey, GlyphKind, GpuCompositor, GpuContext,
    GpuError, PathBuilder, PathTessellator, PremulColor, QuadInstance, RenderTarget,
    TessellatedMesh, VramAtlas, WindowSurface,
};
use tej_ui_hal::{Color, DirtyRectTracker, MouseButton, SurfaceCompositor, UiEvent, WindowConfig};
use tej_ui_layout::{LayoutArena, LayoutEngine, Point, Rect, Size};
use tej_ui_text::{FontRasterizer, FontRegistry, SystemFontDb};
use tej_ui_view::{EventDispatcher, PaintContext, View};

use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::{
    ElementState, KeyEvent, MouseButton as WinitMouseButton, MouseScrollDelta, WindowEvent,
};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{Key, NamedKey};
use winit::window::{Window, WindowAttributes, WindowId as WinitWindowId};

/// Default logical clip bounds representing an unclipped infinite plane.
const UNCLIPPED_BOUNDS: [f32; 4] = [-1e6, -1e6, 2e6, 2e6];

/// Recorded glyph draw command generated from `SurfaceCompositor::draw_text` / `draw_text_styled`.
#[derive(Clone, Debug, PartialEq)]
pub struct GlyphDrawCommand {
    /// Stable font face identifier from `FontRegistry`.
    pub font_id: u64,
    /// Whether this glyph uses a monospace font face.
    pub monospace: bool,
    /// CSS font weight (100..=900).
    pub weight: u16,
    /// Rasterization font size in physical pixels.
    pub size_px: u16,
    /// Character to rasterize and render.
    pub ch: char,
    /// Logical destination bounds `[x, y, width, height]`.
    pub bounds: [f32; 4],
    /// Premultiplied linear RGBA tint color.
    pub color: PremulColor,
    /// Active logical clip bounds `[x, y, width, height]`.
    pub clip_bounds: [f32; 4],
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct LogicalClip {
    bounds: [f32; 4],
    radii: [f32; 4],
}

/// Hardware GPU bridge implementing `SurfaceCompositor` on top of `GpuCompositor`.
///
/// Translates `fill_rect`, `fill_rounded_rect`, `draw_line`, and `draw_text` calls from any
/// `tej-ui-view::View` tree into GPU-ready `QuadInstance`, `TessellatedMesh`, and `GlyphDrawCommand`
/// batches, then flushes them into `GpuCompositor` for submission to a `WindowSurface` or
/// `HeadlessTarget`.
pub struct GpuSurfaceBridge {
    logical_size: Size,
    scale_factor: f32,
    clear_color: PremulColor,
    dirty_rects: Vec<Rect>,
    clip_stack: Vec<LogicalClip>,
    quads: Vec<QuadInstance>,
    vector_mesh: TessellatedMesh,
    glyphs: Vec<GlyphDrawCommand>,
    line_count: usize,
    frame_count: u64,
    rasterizer: Option<Arc<FontRasterizer>>,
}

impl Debug for GpuSurfaceBridge {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GpuSurfaceBridge")
            .field("logical_size", &self.logical_size)
            .field("scale_factor", &self.scale_factor)
            .field("quad_count", &self.quads.len())
            .field("vector_indices", &self.vector_mesh.indices.len())
            .field("glyph_count", &self.glyphs.len())
            .field("line_count", &self.line_count)
            .field("frame_count", &self.frame_count)
            .finish_non_exhaustive()
    }
}

#[inline]
fn dim_to_f32(v: u32) -> f32 {
    f32::from(u16::try_from(v).unwrap_or(u16::MAX))
}

#[inline]
fn count_to_f32(v: usize) -> f32 {
    f32::from(u16::try_from(v).unwrap_or(u16::MAX))
}

#[inline]
fn bearing_to_f32(v: i32) -> f32 {
    f32::from(i16::try_from(v).unwrap_or(0))
}

impl GpuSurfaceBridge {
    /// Construct a new GPU surface bridge for the given logical viewport size and HiDPI scale factor.
    #[must_use]
    pub fn new(logical_size: Size, scale_factor: f32) -> Self {
        Self {
            logical_size,
            scale_factor: if scale_factor.is_finite() && scale_factor > 0.0 {
                scale_factor
            } else {
                1.0
            },
            clear_color: PremulColor::BLACK,
            dirty_rects: Vec::new(),
            clip_stack: Vec::new(),
            quads: Vec::with_capacity(256),
            vector_mesh: TessellatedMesh::new(),
            glyphs: Vec::with_capacity(256),
            line_count: 0,
            frame_count: 0,
            rasterizer: None,
        }
    }

    /// Construct a bridge that attempts to load the host OS sans-serif font for real glyph rasterization.
    #[must_use]
    pub fn with_system_font(logical_size: Size, scale_factor: f32) -> Self {
        let mut bridge = Self::new(logical_size, scale_factor);
        let db = SystemFontDb::new();
        if let Some(font) = db.query_font("sans-serif", 400, tej_ui_text::FontSlant::Normal)
            && let Ok(rast) = FontRasterizer::from_bytes(font.blob.data())
        {
            bridge.rasterizer = Some(Arc::new(rast));
        }
        bridge
    }

    /// Attach a custom `FontRasterizer` for glyph mask generation.
    pub fn set_rasterizer(&mut self, rasterizer: Arc<FontRasterizer>) {
        self.rasterizer = Some(rasterizer);
    }

    /// Update the display HiDPI scale factor.
    pub fn set_scale_factor(&mut self, scale_factor: f32) {
        if scale_factor.is_finite() && scale_factor > 0.0 {
            self.scale_factor = scale_factor;
        }
    }

    /// Set the frame clear color used when flushing to `GpuCompositor`.
    pub fn set_clear_color(&mut self, color: Color) {
        self.clear_color = PremulColor::from_rgba8(color.r, color.g, color.b, color.a);
    }

    /// Push a logical clipping rectangle with optional corner radii `[tl, tr, br, bl]`.
    ///
    /// Nested clips are intersected in logical space.
    pub fn push_clip(&mut self, rect: Rect, corner_radii: [f32; 4]) {
        let candidate = [
            rect.origin.x,
            rect.origin.y,
            rect.size.width.max(0.0),
            rect.size.height.max(0.0),
        ];

        let intersected = if let Some(parent) = self.clip_stack.last() {
            intersect_logical_bounds(parent.bounds, candidate)
        } else {
            candidate
        };

        self.clip_stack.push(LogicalClip {
            bounds: intersected,
            radii: corner_radii,
        });
    }

    /// Pop the most recently pushed logical clipping rectangle.
    pub fn pop_clip(&mut self) -> bool {
        self.clip_stack.pop().is_some()
    }

    /// Current active logical clip bounds, or `None` if unclipped.
    #[must_use]
    pub fn active_clip(&self) -> Option<[f32; 4]> {
        self.clip_stack.last().map(|c| c.bounds)
    }

    /// Current clip stack depth.
    #[must_use]
    pub fn clip_depth(&self) -> usize {
        self.clip_stack.len()
    }

    /// Returns the queued `QuadInstance` slice for the current frame.
    #[must_use]
    pub fn quads(&self) -> &[QuadInstance] {
        &self.quads
    }

    /// Returns the tessellated vector mesh built from `draw_line` calls in the current frame.
    #[must_use]
    pub fn vector_mesh(&self) -> &TessellatedMesh {
        &self.vector_mesh
    }

    /// Returns the queued glyph draw commands for the current frame.
    #[must_use]
    pub fn glyphs(&self) -> &[GlyphDrawCommand] {
        &self.glyphs
    }

    /// Number of `draw_line` segments translated in the current frame.
    #[must_use]
    pub const fn line_count(&self) -> usize {
        self.line_count
    }

    /// Number of completed frames (`end_frame` calls).
    #[must_use]
    pub const fn frame_count(&self) -> u64 {
        self.frame_count
    }

    /// Current logical viewport size.
    #[must_use]
    pub const fn logical_size(&self) -> Size {
        self.logical_size
    }

    /// Current HiDPI scale factor.
    #[must_use]
    pub const fn scale_factor(&self) -> f32 {
        self.scale_factor
    }

    /// Flush all translated quads, vector lines, and text glyphs into `GpuCompositor` and submit
    /// the frame to `target`.
    pub fn flush_to_compositor(
        &mut self,
        ctx: &GpuContext,
        compositor: &mut GpuCompositor,
        target: &mut RenderTarget<'_>,
        mut atlas: Option<&mut VramAtlas>,
    ) -> Result<FrameStats, GpuError> {
        let width_px = target.width();
        let height_px = target.height();
        compositor.begin_frame(width_px, height_px, self.scale_factor, self.clear_color);

        for &quad in &self.quads {
            if quad.clip_bounds == UNCLIPPED_BOUNDS {
                compositor.draw_quad(quad);
            } else {
                compositor.push_clip(quad.clip_bounds, quad.clip_radii);
                compositor.draw_quad(quad);
                let _ = compositor.pop_clip();
            }
        }

        if let Some(ref mut vram_atlas) = atlas {
            for cmd in &self.glyphs {
                if cmd.ch.is_whitespace() {
                    continue;
                }

                let key = GlyphKey::new(cmd.font_id, u32::from(cmd.ch), 0, cmd.size_px);

                let alloc_opt = if let Some(existing) = vram_atlas.get(&key) {
                    Some(existing)
                } else if let Some(rast) = FontRegistry::global()
                    .rasterizer(
                        if cmd.monospace {
                            tej_ui_text::FontFamilyKind::Monospace
                        } else {
                            tej_ui_text::FontFamilyKind::SansSerif
                        },
                        cmd.weight,
                        tej_ui_text::FontSlant::Normal,
                    )
                    .map(|(_, r)| r)
                    .or_else(|| self.rasterizer.clone())
                {
                    let g = rast.rasterize(cmd.ch, f32::from(cmd.size_px));
                    if g.metrics.width > 0 && g.metrics.height > 0 {
                        vram_atlas
                            .insert(
                                key,
                                g.metrics.width as u32,
                                g.metrics.height as u32,
                                &g.mask,
                            )
                            .ok()
                    } else {
                        None
                    }
                } else {
                    // Fallback 4x4 coverage block when no system font rasterizer is loaded.
                    let fallback = [255u8; 16];
                    vram_atlas.insert(key, 4, 4, &fallback).ok()
                };

                if let Some(alloc) = alloc_opt {
                    if cmd.clip_bounds == UNCLIPPED_BOUNDS {
                        compositor.draw_glyph(
                            cmd.bounds,
                            alloc.uv_bounds,
                            cmd.color,
                            GlyphKind::AlphaMask,
                        );
                    } else {
                        compositor.push_clip(cmd.clip_bounds, [0.0; 4]);
                        compositor.draw_glyph(
                            cmd.bounds,
                            alloc.uv_bounds,
                            cmd.color,
                            GlyphKind::AlphaMask,
                        );
                        let _ = compositor.pop_clip();
                    }
                }
            }
        }

        let mesh_opt = (!self.vector_mesh.is_empty()).then_some(&self.vector_mesh);
        compositor.submit_frame(ctx, target, atlas, mesh_opt)
    }

    fn apply_active_clip(&self, mut quad: QuadInstance) -> QuadInstance {
        if let Some(clip) = self.clip_stack.last() {
            quad.clip_bounds = clip.bounds;
            quad.clip_radii = clip.radii;
        }
        quad
    }
}

impl SurfaceCompositor for GpuSurfaceBridge {
    fn begin_frame(&mut self, dirty_rects: &[Rect]) {
        self.dirty_rects.clear();
        self.dirty_rects.extend_from_slice(dirty_rects);
        self.clip_stack.clear();
        self.quads.clear();
        self.vector_mesh.clear();
        self.glyphs.clear();
        self.line_count = 0;
    }

    fn fill_rect(&mut self, rect: Rect, color: Color) {
        if rect.size.width <= 0.0 || rect.size.height <= 0.0 || color.a == 0 {
            return;
        }
        let premul = PremulColor::from_rgba8(color.r, color.g, color.b, color.a);
        let quad = QuadInstance::solid_rect(
            [
                rect.origin.x,
                rect.origin.y,
                rect.size.width,
                rect.size.height,
            ],
            premul,
        );
        self.quads.push(self.apply_active_clip(quad));
    }

    fn fill_rounded_rect(&mut self, rect: Rect, radius: f32, color: Color) {
        if rect.size.width <= 0.0 || rect.size.height <= 0.0 || color.a == 0 {
            return;
        }
        let premul = PremulColor::from_rgba8(color.r, color.g, color.b, color.a);
        let r = radius.max(0.0);
        let quad = QuadInstance::rounded_rect(
            [
                rect.origin.x,
                rect.origin.y,
                rect.size.width,
                rect.size.height,
            ],
            [r; 4],
            premul,
        );
        self.quads.push(self.apply_active_clip(quad));
    }

    fn draw_line(&mut self, p1: Point, p2: Point, stroke_width: f32, color: Color) {
        if color.a == 0 {
            return;
        }
        let width = stroke_width.max(0.5);
        let premul = PremulColor::from_rgba8(color.r, color.g, color.b, color.a);

        let mut pb = PathBuilder::new();
        pb.move_to(p1.x, p1.y);
        pb.line_to(p2.x, p2.y);
        pb.end_open();
        let path = pb.build();

        let mut tess = PathTessellator::new();
        tess.tessellate_stroke(&path, premul, width, &mut self.vector_mesh);
        self.line_count += 1;
    }

    fn push_clip(&mut self, rect: Rect, corner_radii: [f32; 4]) {
        Self::push_clip(self, rect, corner_radii);
    }

    fn pop_clip(&mut self) {
        let _ = Self::pop_clip(self);
    }

    fn draw_text(&mut self, text: &str, origin: Point, font_size: f32, color: Color) {
        self.draw_text_styled(
            text,
            origin,
            font_size,
            400,
            tej_ui_hal::TextFamily::SansSerif,
            color,
        );
    }

    fn draw_text_styled(
        &mut self,
        text: &str,
        origin: Point,
        font_size: f32,
        weight: u16,
        family: tej_ui_hal::TextFamily,
        color: Color,
    ) {
        let monospace = matches!(family, tej_ui_hal::TextFamily::Monospace);
        if text.is_empty() || color.a == 0 {
            return;
        }

        let safe_size = if font_size.is_finite() && font_size > 0.0 {
            font_size
        } else {
            14.0
        };
        let premul = PremulColor::from_rgba8(color.r, color.g, color.b, color.a);
        let clip_bounds = self
            .clip_stack
            .last()
            .map_or(UNCLIPPED_BOUNDS, |c| c.bounds);

        let sf = self.scale_factor.max(0.5);
        let inv_sf = 1.0 / sf;
        let phys_size = (safe_size * sf).round().max(1.0);
        let size_px = phys_size.clamp(1.0, f32::from(u16::MAX)) as u16;

        let family_kind = if monospace {
            tej_ui_text::FontFamilyKind::Monospace
        } else {
            tej_ui_text::FontFamilyKind::SansSerif
        };
        let font_entry = FontRegistry::global()
            .rasterizer(family_kind, weight, tej_ui_text::FontSlant::Normal)
            .or_else(|| self.rasterizer.as_ref().map(|r| (0, Arc::clone(r))));

        if let Some((font_id, rast)) = font_entry {
            let (ascent_px, _descent_px, line_height_px) = rast.horizontal_line_metrics(phys_size);
            let mut pen_x_px = (origin.x * sf).round();
            let mut baseline_y_px = (origin.y * sf).round() + ascent_px.round();

            for ch in text.chars() {
                if ch == '\n' {
                    pen_x_px = (origin.x * sf).round();
                    baseline_y_px += line_height_px.round();
                    continue;
                }

                let g = rast.rasterize(ch, phys_size);
                if !ch.is_whitespace() && g.metrics.width > 0 && g.metrics.height > 0 {
                    let gx_px = pen_x_px.round() + bearing_to_f32(g.metrics.xmin);
                    let gy_px = baseline_y_px
                        - bearing_to_f32(g.metrics.ymin)
                        - count_to_f32(g.metrics.height);
                    self.glyphs.push(GlyphDrawCommand {
                        font_id,
                        monospace,
                        weight,
                        size_px,
                        ch,
                        bounds: [
                            gx_px * inv_sf,
                            gy_px * inv_sf,
                            count_to_f32(g.metrics.width) * inv_sf,
                            count_to_f32(g.metrics.height) * inv_sf,
                        ],
                        color: premul,
                        clip_bounds,
                    });
                }

                pen_x_px += g.metrics.advance_width;
            }
        } else {
            let mut cursor_x = origin.x;
            let mut cursor_y = origin.y;
            let line_height = safe_size * 1.35;
            let approx_w = safe_size * 0.6;

            for ch in text.chars() {
                if ch == '\n' {
                    cursor_x = origin.x;
                    cursor_y += line_height;
                    continue;
                }

                if !ch.is_whitespace() {
                    self.glyphs.push(GlyphDrawCommand {
                        font_id: 0,
                        monospace,
                        weight,
                        size_px,
                        ch,
                        bounds: [cursor_x, cursor_y, approx_w, safe_size],
                        color: premul,
                        clip_bounds,
                    });
                }

                cursor_x += approx_w;
            }
        }
    }

    fn end_frame(&mut self) {
        self.frame_count += 1;
    }

    fn resize(&mut self, size: Size) {
        self.logical_size = size;
    }
}

fn intersect_logical_bounds(a: [f32; 4], b: [f32; 4]) -> [f32; 4] {
    let x0 = a[0].max(b[0]);
    let y0 = a[1].max(b[1]);
    let x1 = (a[0] + a[2]).min(b[0] + b[2]);
    let y1 = (a[1] + a[3]).min(b[1] + b[3]);
    [x0, y0, (x1 - x0).max(0.0), (y1 - y0).max(0.0)]
}

struct ActiveGpuState {
    window: Arc<Window>,
    ctx: GpuContext,
    target: RenderTarget<'static>,
    compositor: GpuCompositor,
    atlas: VramAtlas,
    bridge: GpuSurfaceBridge,
}

/// Production desktop `winit` 0.30 `ApplicationHandler` driving `GpuCompositor` with
/// `ControlFlow::Wait` (0.0% idle CPU) and reactive epoch wakeups.
pub struct NativeWinitApp<V: View> {
    config: WindowConfig,
    root_view: V,
    view_state: Option<V::State>,
    layout_arena: LayoutArena,
    root_node: Option<u32>,
    dirty_tracker: DirtyRectTracker,
    dispatcher: EventDispatcher,
    gpu: Option<ActiveGpuState>,
    cursor_pos: Point,
    scroll_offset_y: f32,
    last_reactive_epoch: Epoch,
    needs_relayout: bool,
    needs_repaint: bool,
}

impl<V: View> Debug for NativeWinitApp<V> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NativeWinitApp")
            .field("config", &self.config)
            .field("cursor_pos", &self.cursor_pos)
            .field("scroll_offset_y", &self.scroll_offset_y)
            .field("last_reactive_epoch", &self.last_reactive_epoch)
            .field("needs_relayout", &self.needs_relayout)
            .field("needs_repaint", &self.needs_repaint)
            .finish_non_exhaustive()
    }
}

impl<V: View> NativeWinitApp<V> {
    /// Create a new native desktop application handler from a `WindowConfig` and root `View`.
    #[must_use]
    pub fn new(config: WindowConfig, root_view: V) -> Self {
        Self {
            config,
            root_view,
            view_state: None,
            layout_arena: LayoutArena::new(),
            root_node: None,
            dirty_tracker: DirtyRectTracker::new(),
            dispatcher: EventDispatcher::new(),
            gpu: None,
            cursor_pos: Point::new(0.0, 0.0),
            scroll_offset_y: 0.0,
            last_reactive_epoch: ReactiveRuntime::epoch(),
            needs_relayout: true,
            needs_repaint: true,
        }
    }

    fn dispatch_ui_event(&mut self, event: &UiEvent) {
        let Some(root_node) = self.root_node else {
            return;
        };
        let res = self
            .dispatcher
            .dispatch(event, &self.layout_arena, root_node);
        if res.handled {
            self.needs_relayout = true;
            self.needs_repaint = true;
            self.dirty_tracker.mark_full_dirty();
        } else if res.needs_repaint {
            self.needs_repaint = true;
            self.dirty_tracker.mark_full_dirty();
        }
    }

    fn render_frame(&mut self) {
        let Some(gpu) = self.gpu.as_mut() else {
            return;
        };
        let Some(state) = self.view_state.as_mut() else {
            return;
        };

        let current_epoch = ReactiveRuntime::epoch();
        if current_epoch != self.last_reactive_epoch {
            self.last_reactive_epoch = current_epoch;
            self.needs_relayout = true;
            self.needs_repaint = true;
            self.dirty_tracker.mark_full_dirty();
        }

        if self.needs_repaint || self.needs_relayout || self.dirty_tracker.has_dirty() {
            self.root_view.patch(state, 0);
        }

        let logical_size = gpu.bridge.logical_size();
        if self.needs_relayout || self.root_node.is_none() {
            self.layout_arena.clear();
            let node = self.root_view.layout(state, &mut self.layout_arena);
            LayoutEngine::compute_layout(&mut self.layout_arena, node, logical_size);
            self.dispatcher.clear_handlers();
            self.root_view.register_events(state, &mut self.dispatcher);
            self.root_node = Some(node);
            self.dirty_tracker.mark_full_dirty();
            self.needs_relayout = false;
            self.needs_repaint = true;
        }

        let theme = tej_ui_view::use_theme();
        gpu.bridge.set_clear_color(theme.colors.background);
        gpu.bridge.begin_frame(self.dirty_tracker.rects());
        gpu.bridge.fill_rect(
            Rect::new(0.0, 0.0, logical_size.width, logical_size.height),
            theme.colors.background,
        );

        let mut paint_ctx = PaintContext::new(
            &mut gpu.bridge,
            &self.layout_arena,
            &theme,
            self.dirty_tracker.rects(),
        )
        .with_scroll(self.scroll_offset_y, 0.0)
        .with_interaction(
            self.dispatcher.focused_node,
            self.dispatcher.hovered_node,
            self.dispatcher.pressed_node,
        );
        self.root_view.paint(state, &mut paint_ctx);
        paint_ctx.flush_overlays();
        gpu.bridge.end_frame();

        let _ = gpu.bridge.flush_to_compositor(
            &gpu.ctx,
            &mut gpu.compositor,
            &mut gpu.target,
            Some(&mut gpu.atlas),
        );

        self.dirty_tracker.reset();
        self.needs_repaint = false;
    }

    fn handle_resized(&mut self, width: u32, height: u32) {
        if width == 0 || height == 0 {
            return;
        }
        if let Some(gpu) = self.gpu.as_mut() {
            let _ = gpu.target.resize(&gpu.ctx, width, height);
            let sf = gpu.bridge.scale_factor();
            let logical = Size::new(
                (dim_to_f32(width) / sf).max(1.0),
                (dim_to_f32(height) / sf).max(1.0),
            );
            gpu.bridge.resize(logical);
            gpu.window.request_redraw();
            self.needs_relayout = true;
            self.needs_repaint = true;
            self.dirty_tracker.mark_full_dirty();
            self.dispatch_ui_event(&UiEvent::Resize {
                width: logical.width,
                height: logical.height,
            });
        }
    }

    fn handle_scale_factor_changed(&mut self, scale_factor: f64) {
        let sf = scale_factor as f32;
        if let Some(gpu) = self.gpu.as_mut() {
            gpu.bridge.set_scale_factor(sf);
            let phys = gpu.window.inner_size();
            let logical = Size::new(
                (dim_to_f32(phys.width) / sf).max(1.0),
                (dim_to_f32(phys.height) / sf).max(1.0),
            );
            gpu.bridge.resize(logical);
            gpu.window.request_redraw();
        }
        self.needs_relayout = true;
        self.needs_repaint = true;
        self.dirty_tracker.mark_full_dirty();
        self.dispatch_ui_event(&UiEvent::ScaleFactorChanged { scale_factor: sf });
    }

    fn handle_mouse_input(&mut self, state: ElementState, button: WinitMouseButton) {
        let Some(mapped_btn) = map_winit_mouse_button(button) else {
            return;
        };
        let adj = Point::new(self.cursor_pos.x, self.cursor_pos.y + self.scroll_offset_y);
        let ui_evt = match state {
            ElementState::Pressed => UiEvent::MouseDown {
                button: mapped_btn,
                position: adj,
            },
            ElementState::Released => UiEvent::MouseUp {
                button: mapped_btn,
                position: adj,
            },
        };
        self.dispatch_ui_event(&ui_evt);
        if (self.needs_repaint || self.needs_relayout)
            && let Some(gpu) = self.gpu.as_ref()
        {
            gpu.window.request_redraw();
        }
    }

    fn handle_mouse_wheel(&mut self, delta: MouseScrollDelta) {
        let (dx, dy) = match delta {
            MouseScrollDelta::LineDelta(x, y) => (x, y),
            MouseScrollDelta::PixelDelta(p) => (p.x as f32 / 20.0, p.y as f32 / 20.0),
        };
        self.scroll_offset_y = (self.scroll_offset_y - dy * 45.0).max(0.0);
        self.needs_repaint = true;
        self.dirty_tracker.mark_full_dirty();
        self.dispatch_ui_event(&UiEvent::Scroll {
            delta_x: dx,
            delta_y: dy,
            position: self.cursor_pos,
        });
        if let Some(gpu) = self.gpu.as_ref() {
            gpu.window.request_redraw();
        }
    }

    fn handle_keyboard_input(&mut self, event: &KeyEvent) {
        let key_code = map_winit_key_code(event);
        let ui_evt = match event.state {
            ElementState::Pressed => UiEvent::KeyDown { key_code },
            ElementState::Released => UiEvent::KeyUp { key_code },
        };
        self.dispatch_ui_event(&ui_evt);

        if event.state == ElementState::Pressed
            && let Some(ref text) = event.text
        {
            let filtered: String = text.chars().filter(|c| !c.is_control()).collect();
            if !filtered.is_empty() {
                self.dispatch_ui_event(&UiEvent::TextInput { text: filtered });
            }
        }

        if (self.needs_repaint || self.needs_relayout)
            && let Some(gpu) = self.gpu.as_ref()
        {
            gpu.window.request_redraw();
        }
    }
}

impl<V: View> ApplicationHandler for NativeWinitApp<V> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.gpu.is_some() {
            return;
        }

        let attrs = WindowAttributes::default()
            .with_title(self.config.title.clone())
            .with_inner_size(LogicalSize::new(
                f64::from(self.config.size.width),
                f64::from(self.config.size.height),
            ))
            .with_resizable(self.config.resizable)
            .with_decorations(self.config.decorated);

        let Ok(raw_window) = event_loop.create_window(attrs) else {
            event_loop.exit();
            return;
        };
        let window = Arc::new(raw_window);
        let phys_size = window.inner_size();
        let scale_factor = window.scale_factor() as f32;
        let logical_size = Size::new(
            (dim_to_f32(phys_size.width) / scale_factor).max(1.0),
            (dim_to_f32(phys_size.height) / scale_factor).max(1.0),
        );

        let backend_cfg = BackendConfig::default();
        let Ok(ctx) = GpuContext::new_headless(&backend_cfg) else {
            event_loop.exit();
            return;
        };

        let Ok(surface) = WindowSurface::new(
            &ctx,
            Arc::clone(&window),
            phys_size.width.max(1),
            phys_size.height.max(1),
            backend_cfg.present_mode,
        ) else {
            event_loop.exit();
            return;
        };

        let format = surface.format();
        let compositor = GpuCompositor::new(&ctx, format, &backend_cfg);
        let Ok(atlas) = VramAtlas::new(ctx.device(), 1024, 1024, AtlasFormat::Grayscale) else {
            event_loop.exit();
            return;
        };
        let bridge = GpuSurfaceBridge::with_system_font(logical_size, scale_factor);

        let mut state = self.root_view.build();
        self.layout_arena.clear();
        let root_node = self.root_view.layout(&mut state, &mut self.layout_arena);
        LayoutEngine::compute_layout(&mut self.layout_arena, root_node, logical_size);
        self.dispatcher.clear_handlers();
        self.root_view.register_events(&state, &mut self.dispatcher);

        self.view_state = Some(state);
        self.root_node = Some(root_node);
        self.dirty_tracker.mark_full_dirty();
        self.last_reactive_epoch = ReactiveRuntime::epoch();
        self.needs_relayout = false;
        self.needs_repaint = true;

        window.request_redraw();

        self.gpu = Some(ActiveGpuState {
            window,
            ctx,
            target: RenderTarget::Window(surface),
            compositor,
            atlas,
            bridge,
        });
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WinitWindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                self.dispatch_ui_event(&UiEvent::CloseRequested);
                event_loop.exit();
            }
            WindowEvent::Resized(new_size) => self.handle_resized(new_size.width, new_size.height),
            WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                self.handle_scale_factor_changed(scale_factor);
            }
            WindowEvent::CursorMoved { position, .. } => {
                let sf = self.gpu.as_ref().map_or(1.0, |g| g.bridge.scale_factor());
                let logical_pos = Point::new(position.x as f32 / sf, position.y as f32 / sf);
                self.cursor_pos = logical_pos;
                let adj = Point::new(logical_pos.x, logical_pos.y + self.scroll_offset_y);
                self.dispatch_ui_event(&UiEvent::MouseMove { position: adj });
                if self.needs_repaint
                    && let Some(gpu) = self.gpu.as_ref()
                {
                    gpu.window.request_redraw();
                }
            }
            WindowEvent::MouseInput { state, button, .. } => self.handle_mouse_input(state, button),
            WindowEvent::MouseWheel { delta, .. } => self.handle_mouse_wheel(delta),
            WindowEvent::KeyboardInput { event, .. } => self.handle_keyboard_input(&event),
            WindowEvent::Focused(focused) => {
                let evt = if focused {
                    UiEvent::FocusGained
                } else {
                    UiEvent::FocusLost
                };
                self.dispatch_ui_event(&evt);
            }
            WindowEvent::RedrawRequested => {
                self.render_frame();
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        let current_epoch = ReactiveRuntime::epoch();
        if current_epoch != self.last_reactive_epoch {
            self.last_reactive_epoch = current_epoch;
            self.needs_relayout = true;
            self.needs_repaint = true;
            self.dirty_tracker.mark_full_dirty();
            if let Some(gpu) = self.gpu.as_ref() {
                gpu.window.request_redraw();
            }
        }
    }
}

fn map_winit_mouse_button(button: WinitMouseButton) -> Option<MouseButton> {
    match button {
        WinitMouseButton::Left => Some(MouseButton::Left),
        WinitMouseButton::Right => Some(MouseButton::Right),
        WinitMouseButton::Middle => Some(MouseButton::Middle),
        _ => None,
    }
}

fn map_winit_key_code(event: &KeyEvent) -> u32 {
    match &event.logical_key {
        Key::Named(named) => match named {
            NamedKey::Enter => 13,
            NamedKey::Tab => 9,
            NamedKey::Backspace => 8,
            NamedKey::Escape => 27,
            NamedKey::Space => 32,
            NamedKey::ArrowLeft => 37,
            NamedKey::ArrowUp => 38,
            NamedKey::ArrowRight => 39,
            NamedKey::ArrowDown => 40,
            NamedKey::Delete => 127,
            NamedKey::Home => 36,
            NamedKey::End => 35,
            _ => 0,
        },
        Key::Character(s) => s.chars().next().map_or(0, u32::from),
        _ => 0,
    }
}

/// Launch the production desktop `winit` 0.30 + `wgpu` 24 event loop with `ControlFlow::Wait`.
pub fn run_native_app<V: View>(
    config: WindowConfig,
    root_view: V,
) -> Result<(), Box<dyn std::error::Error>> {
    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(ControlFlow::Wait);
    let mut app = NativeWinitApp::new(config, root_view);
    event_loop.run_app(&mut app)?;
    Ok(())
}
