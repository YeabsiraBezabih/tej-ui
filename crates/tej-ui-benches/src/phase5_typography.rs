//! Phase 5: Typography, Shaping, Unicode BiDi & CJK Stress.
//!
//! Renders 500 mixed-script paragraphs: ASCII code with ligatures (->, =>, !=),
//! Bidirectional Arabic/Hebrew with English numbers (UAX #9), Ge'ez (ሰላም ለዓለም),
//! Devanagari, CJK ideographs, and color emojis.

use crate::allocator::TrackingAllocator;
use std::hint::black_box;
use std::time::Instant;
use tej_ui_text::{BidiClassifier, SystemFont, TextMeasurer, TextShaper};

/// Metrics for typography and complex text shaping benchmark.
#[derive(Debug, Clone, Copy, Default)]
pub struct TypographyBenchMetric {
    pub engine_name: &'static str,
    pub paragraph_count: usize,
    pub total_glyphs_shaped: usize,
    pub shaping_latency_ms: f64,
    pub glyphs_per_sec: f64,
    pub memory_per_10k_glyphs_kb: f64,
}

fn generate_multilingual_corpus(count: usize) -> Vec<String> {
    let templates = [
        "fn calculate_delta<T: Copy>(lhs: &T, rhs: &T) -> bool { lhs != rhs => true }",
        "مرحبا بالعالم! The Arabic language mixed with English numbers 2026 and text.",
        "ሰላም ለዓለም! እንኳን ወደ tej-ui የላቀ የቅርጸ-ፊደል ማስተካከያ በሰላም መጡ።",
        "שלום עולם! עברית עם מספרים 123456 ובדיקת כיווניות UAX #9.",
        "नमस्ते दुनिया! देवनागरी लिपि और संवत् २०८३ में आपका स्वागत है।",
        "你好世界！这是一个复杂的CJK字符排版测试，包含繁体字與简体字以及标点符号。",
        "Rust UI benchmark with ligatures: fn main() -> Result<(), Box<dyn Error>> { x != y }",
        "Mixed emoji and flags: 🚀 🦀 ⚡ 🎨 🔥 🌟 🎯 💻 🌍 🏁",
    ];

    let mut corpus = Vec::with_capacity(count);
    for i in 0..count {
        let tmpl = templates[i % templates.len()];
        corpus.push(format!("[Paragraph #{i}] {tmpl}"));
    }
    corpus
}

fn bench_tej_typography(paragraphs: &[String]) -> TypographyBenchMetric {
    let shaper = TextShaper::new();
    let measurer = TextMeasurer::new();
    let snap = TrackingAllocator::reset_and_snapshot();
    let t0 = Instant::now();

    let mut total_glyphs = 0usize;
    for p in paragraphs {
        let dir = BidiClassifier::detect_base_direction(p);
        let run = shaper.shape(p, SystemFont::Body, 14.0, dir);
        let bounds = measurer.measure_slice(p, SystemFont::Body, 14.0);
        black_box(bounds);
        total_glyphs += run.glyphs.len();
    }

    let elapsed = t0.elapsed();
    let (alloc_count, bytes, _peak) = TrackingAllocator::delta_since(snap);
    let ms = elapsed.as_secs_f64() * 1000.0;
    let gps = crate::count_to_f64(total_glyphs) / elapsed.as_secs_f64().max(0.0001);
    let kb_per_10k = if total_glyphs > 0 {
        (crate::count_to_f64(bytes) / crate::count_to_f64(total_glyphs)) * 10_000.0 / 1024.0
    } else {
        0.0
    };

    let _ = alloc_count;
    TypographyBenchMetric {
        engine_name: "tej-text (Direct HarfBuzz/Bidi)",
        paragraph_count: paragraphs.len(),
        total_glyphs_shaped: total_glyphs,
        shaping_latency_ms: ms,
        glyphs_per_sec: gps,
        memory_per_10k_glyphs_kb: kb_per_10k,
    }
}

fn bench_egui_typography(paragraphs: &[String]) -> TypographyBenchMetric {
    let ctx = egui::Context::default();
    let raw = egui::RawInput::default();
    let snap = TrackingAllocator::reset_and_snapshot();
    let t0 = Instant::now();

    let output = ctx.run(raw, |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            for p in paragraphs {
                ui.label(p);
            }
        });
    });

    let primitives = ctx.tessellate(output.shapes, 2.0);
    black_box(&primitives);
    let elapsed = t0.elapsed();
    let (_alloc, bytes, _peak) = TrackingAllocator::delta_since(snap);

    let ms = elapsed.as_secs_f64() * 1000.0;
    let approx_glyphs = paragraphs.iter().map(|p| p.chars().count()).sum::<usize>();
    let gps = crate::count_to_f64(approx_glyphs) / elapsed.as_secs_f64().max(0.0001);
    let kb_per_10k = if approx_glyphs > 0 {
        (crate::count_to_f64(bytes) / crate::count_to_f64(approx_glyphs)) * 10_000.0 / 1024.0
    } else {
        0.0
    };

    TypographyBenchMetric {
        engine_name: "egui epaint (epaint_default_fonts)",
        paragraph_count: paragraphs.len(),
        total_glyphs_shaped: approx_glyphs,
        shaping_latency_ms: ms,
        glyphs_per_sec: gps,
        memory_per_10k_glyphs_kb: kb_per_10k,
    }
}

/// Execute Phase 5 Typography & Complex Script Shaping benchmarks.
#[must_use]
pub fn run_phase_5_typography() -> Vec<TypographyBenchMetric> {
    println!(
        "\n>>> Running Phase 5: Typography, Shaping, Unicode BiDi & CJK Stress (500 Paragraphs)..."
    );
    let t0 = Instant::now();
    let corpus = generate_multilingual_corpus(500);

    let metrics = vec![
        bench_tej_typography(&corpus),
        bench_egui_typography(&corpus),
    ];

    println!("    Completed Phase 5 in {:.2?}", t0.elapsed());
    metrics
}
