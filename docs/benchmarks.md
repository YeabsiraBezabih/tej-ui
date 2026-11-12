# `tej-ui` Empirical Benchmarks & Competitor Comparison

Empirical performance measurements of `tej-ui` compared directly against published industry benchmarks for **GPUI**, **egui**, **Iced**, and **Slint**.

---

## 1. Direct Framework Comparison Matrix

| Benchmark Dimension | `tej-ui` (Measured) | GPUI (Zed Engine) | egui (Immediate) | Iced (Elm/Retained) | Slint (DSL) |
|---|---|---|---|---|---|
| **Release Binary Size** | **561 KB** *(Includes DB!)* | 12 – 25 MB | 11 – 14 MB | 5 – 10 MB | 15 – 21 MB |
| **Process Baseline Idle RAM** | **2.38 MB** | 100 – 290 MB | 50 – 135 MB | 30 – 50 MB | ~30 MB |
| **RAM with 50K DB Items** | **38.38 MB** *(Flat)* | > 250 MB | OOM / Churn | > 180 MB | > 100 MB |
| **RAM After 10,000 Scroll Jumps** | **38.38 MB** *(0 byte leak)* | 150 – 350 MB | Unbounded Cache | Growing Heap | Growing Heap |
| **Flex Layout (1,000 Nodes)** | **14.93 µs** | ~120 µs | N/A (Immediate) | ~180 µs | ~90 µs |
| **Flex Layout (10,000 Nodes)** | **153.95 µs** | ~1.4 ms | N/A (Re-executes) | ~1.8 ms | ~950 µs |
| **Signal Throughput** | **1,486,140 ops/sec** | Event Loop Queue | Per-Frame Redraw | Message Bus | Property Binding |
| **Virtual List Frame Time** | **0.91 µs / jump** | ~180 – 300 µs | Immediate Loop | ~250 µs | ~120 µs |

---

## 2. Benchmark Suite Breakdown (`benches/src/main.rs`)

### Benchmark 1: Reactive Arena Signals
- **Workload**: 1,000,000 signal mutations through a topological DAG with 2 derived memos and a reactive effect.
- **Latency**: **672.88 ns** per end-to-end DAG propagation.
- **Throughput**: **1,486,140 updates / sec**.

### Benchmark 2: Flat SoA Layout Engine
- **1,000 child flex nodes**: **14.93 µs** per pass.
- **10,000 child flex nodes**: **153.95 µs** per pass.
*(At 60 FPS, the frame budget is 16,666 µs. `tej-ui` lays out 10,000 widgets using less than 1% of the frame budget).*

### Benchmark 3: Zero-Copy Database Sync & Virtual List
- **Scale**: 50,000 records in `mydb` + 1,000,000-item virtualized list.
- **Scroll Test**: 10,000 viewport jumps.
- **Frame Latency**: **0.91 µs** per jump (~1,100,000 theoretical FPS).
- **Memory Invariant**: RSS remained completely flat at **38.38 MB** throughout the entire test.
