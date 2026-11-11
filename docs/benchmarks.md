# `tej-ui` Benchmarks & Performance Targets

## Target Specifications & Competitor Comparison

| Metric | Target (`tej-ui`) | Tauri (WebView) | GPUI / Iced (GPU Canvas) | Slint (DSL) |
|---|---|---|---|---|
| **Binary Size (Stripped Release)** | **< 3.0 MB** | 4 – 10 MB | 10 – 25 MB | 2 – 5 MB |
| **Idle RAM (1080p Display)** | **< 15 MB** | 60 – 110 MB | 45 – 70 MB | < 5 MB |
| **Idle RAM (4K High-DPI Display)** | **< 25 MB** | 90 – 220 MB | 70 – 130 MB | 8 – 15 MB |
| **Cold Startup Time** | **< 35 ms** | 180 – 350 ms | 60 – 120 ms | 20 – 40 ms |
| **Steady-State Frame Allocations** | **0 bytes** | High (DOM GC) | Low | 0 bytes |
| **1M Row Virtual List RAM** | **< 25 MB (Flat)** | > 300 MB | 150 – 250 MB | > 100 MB |
| **Signal Dispatch Latency** | **< 12 ns** | N/A (IPC) | 120 – 400 ns | 30 – 80 ns |

---

## Benchmark Suites to Implement

1. **`bench_signals`**:
   - Measures raw throughput and latency of signal updates across deep graphs (depth 10, 50, 100), wide fan-out (1 to 1,000 subscribers), and diamond dependency DAGs.
2. **`bench_layout`**:
   - Tests `Taffy` layout computation time on flat SoA arrays for 100, 1,000, and 10,000 nested flex containers.
3. **`bench_db_virtual_list`**:
   - Measures frame time and memory usage when rapidly scrolling through a 1,000,000-row `mydb` table reading `&str` slices directly from memory-mapped slotted pages.
4. **`bench_startup_memory`**:
   - Automated RSS memory measurement after window launch and 5 seconds of idle time.
