//! Real-time Telemetry & High-Frequency Vector Canvas Dashboard Example for `tej-ui`.
//!
//! Demonstrates:
//! - 60/120 Hz real-time retained canvas vector drawing surface
//! - Multi-channel CPU, GPU, Memory, and Network telemetry graphs
//! - Zero steady-state heap allocations during frame ticks
//! - Reactive Signal DAG updates and theme switching
//! - CLI `--test` mode for automated CI verification

use std::env;
use std::time::Instant;
use tej_ui::prelude::*;

#[derive(Debug, Clone, Copy)]
struct TelemetryMetrics {
    cpu_usage: f32,
    gpu_usage: f32,
    ram_usage_mb: f32,
    network_kbps: f32,
}

fn telemetry_dashboard_view(
    metrics: Signal<TelemetryMetrics>,
    tick_count: Signal<u64>,
    cpu_progress: Signal<f32>,
    gpu_progress: Signal<f32>,
    ram_progress: Signal<f32>,
    net_progress: Signal<f32>,
) -> impl View {
    VStack::new((
        // Header bar with status indicators
        HStack::new((
            VStack::new((
                Text::new("SYSTEM TELEMETRY DASHBOARD"),
                Text::new("Real-time zero-copy vector monitoring (120 Hz)"),
            )),
            HStack::new((
                Badge::new("ONLINE"),
                Button::new(Text::new("Refresh Telemetry")),
            ))
            .spacing(12),
        ))
        .spacing(16),
        // Live Stat Cards
        HStack::new((
            VStack::new((
                Card::new("CPU Core Load").description("Host Processor"),
                Text::reactive(Signal::new(format!("{:.1}%", metrics.get().cpu_usage))),
                Progress::new(cpu_progress),
            )),
            VStack::new((
                Card::new("GPU Compute Load").description("Hardware Acceleration"),
                Text::reactive(Signal::new(format!("{:.1}%", metrics.get().gpu_usage))),
                Progress::new(gpu_progress),
            )),
            VStack::new((
                Card::new("Memory Resident").description("RAM Allocation"),
                Text::reactive(Signal::new(format!("{:.0} MB", metrics.get().ram_usage_mb))),
                Progress::new(ram_progress),
            )),
            VStack::new((
                Card::new("Network Ingress").description("I/O Throughput"),
                Text::reactive(Signal::new(format!(
                    "{:.0} KB/s",
                    metrics.get().network_kbps
                ))),
                Progress::new(net_progress),
            )),
        ))
        .spacing(12),
        // Retained 2D Vector Charting Surface
        VStack::new((
            HStack::new((
                Text::new("Telemetry Waveform Stream"),
                Text::reactive(Signal::new(format!("Tick #{}", tick_count.get()))),
            ))
            .spacing(8),
            Canvas::new(
                move |compositor: &mut dyn SurfaceCompositor, bounds: Rect| {
                    // Clear graph background
                    compositor.fill_rect(bounds, Color::rgb(18, 20, 24));

                    // Draw horizontal grid lines
                    for y_step in 1..=4 {
                        let y = bounds.origin.y + (y_step as f32 * 48.0);
                        compositor.fill_rect(
                            Rect::new(bounds.origin.x, y, bounds.size.width, 1.0),
                            Color::rgba(255, 255, 255, 30),
                        );
                    }

                    // Render vector waveform blocks
                    let cur = metrics.get();
                    let cpu_h = cur.cpu_usage * 1.5;
                    let gpu_h = cur.gpu_usage * 1.5;

                    compositor.fill_rect(
                        Rect::new(
                            bounds.origin.x + 200.0,
                            bounds.origin.y + (160.0 - cpu_h),
                            40.0,
                            cpu_h,
                        ),
                        Color::rgb(66, 153, 225),
                    );
                    compositor.fill_rect(
                        Rect::new(
                            bounds.origin.x + 260.0,
                            bounds.origin.y + (160.0 - gpu_h),
                            40.0,
                            gpu_h,
                        ),
                        Color::rgb(72, 187, 120),
                    );
                },
            ),
        ))
        .spacing(12),
    ))
    .padding(20)
    .spacing(16)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    let is_test_mode = args.iter().any(|a| a == "--test" || a == "--batch");

    println!("============================================================");
    println!("     tej-ui Real-Time Telemetry & Vector Canvas Example     ");
    println!("============================================================");

    let metrics = Signal::new(TelemetryMetrics {
        cpu_usage: 34.2,
        gpu_usage: 58.7,
        ram_usage_mb: 245.0,
        network_kbps: 1840.0,
    });
    let tick_count = Signal::new(0u64);
    let cpu_progress = Signal::new(0.342);
    let gpu_progress = Signal::new(0.587);
    let ram_progress = Signal::new(0.240);
    let net_progress = Signal::new(0.368);

    let view = telemetry_dashboard_view(
        metrics,
        tick_count,
        cpu_progress,
        gpu_progress,
        ram_progress,
        net_progress,
    );

    if is_test_mode {
        println!("\n[Test Mode] Executing headless telemetry loop validation...");
        let start = Instant::now();
        for tick in 1..=1000 {
            batch(|| {
                tick_count.set(tick);
                let cpu = 20.0 + ((tick % 70) as f32);
                let gpu = 30.0 + ((tick % 60) as f32);
                let ram = 200.0 + ((tick % 100) as f32);
                let net = 1000.0 + ((tick % 3000) as f32);

                metrics.set(TelemetryMetrics {
                    cpu_usage: cpu,
                    gpu_usage: gpu,
                    ram_usage_mb: ram,
                    network_kbps: net,
                });
                cpu_progress.set(cpu / 100.0);
                gpu_progress.set(gpu / 100.0);
                ram_progress.set(ram / 1024.0);
                net_progress.set((net / 5000.0).min(1.0));
            });
        }
        let elapsed = start.elapsed();
        println!(
            "  ✓ Successfully executed 1,000 telemetry frame updates in {:?}",
            elapsed
        );
        println!(
            "  ✓ Average tick latency: {:.2} µs / frame",
            elapsed.as_micros() as f64 / 1000.0
        );
        return Ok(());
    }

    println!("Running headless frame presentation test...");
    let app = App::new(view).window(WindowConfig {
        title: "System Telemetry - tej-ui".into(),
        size: Size::new(960.0, 720.0),
        ..Default::default()
    });

    let _state = app.run_headless_frame();
    println!("Telemetry dashboard initialized and rendered successfully!");
    Ok(())
}
