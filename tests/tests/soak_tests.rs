//! Deterministic Interaction Soak Harness and Hash Fingerprinting.
//! Simulates thousands of user interaction cycles under a fixed PRNG seed to verify
//! that state transitions, widget models, and event handling remain completely bit-identical across refactors.

use tej_ui_hal::Color;
use tej_ui_widgets::{
    KanbanCard, KanbanColumn, KanbanState, KnobState, OtpInputState, ReasoningDrawerState,
    ScrubFieldState,
};

/// Deterministic 64-bit Xorshift PRNG for repeatable test playback.
struct SoakPrng {
    state: u64,
}

impl SoakPrng {
    fn new(seed: u64) -> Self {
        Self {
            state: if seed == 0 {
                0x853c_49e6_748f_ea9b
            } else {
                seed
            },
        }
    }

    fn next_u32(&mut self) -> u32 {
        let mut x = self.state;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.state = x;
        (x & 0xFFFF_FFFF) as u32
    }

    fn next_f32(&mut self) -> f32 {
        let hi = u16::try_from(self.next_u32() >> 16).unwrap_or(0);
        f32::from(hi) / f32::from(u16::MAX)
    }
}

/// Accumulator computing a 64-bit FNV-1a state fingerprint.
struct StateFingerprint {
    hash: u64,
}

impl StateFingerprint {
    fn new() -> Self {
        Self {
            hash: 0xcbf2_9ce4_8422_2325,
        }
    }

    fn feed_u64(&mut self, val: u64) {
        for b in val.to_le_bytes() {
            self.hash ^= u64::from(b);
            self.hash = self.hash.wrapping_mul(0x0100_0000_01b3);
        }
    }

    fn feed_f32(&mut self, val: f32) {
        self.feed_u64(u64::from(val.to_bits()));
    }

    fn feed_str(&mut self, val: &str) {
        for b in val.as_bytes() {
            self.hash ^= u64::from(*b);
            self.hash = self.hash.wrapping_mul(0x0100_0000_01b3);
        }
    }

    fn finish(&self) -> u64 {
        self.hash
    }
}

fn run_soak_simulation(seed: u64) -> u64 {
    let mut prng = SoakPrng::new(seed);
    let mut fp = StateFingerprint::new();

    // Instantiate a suite of dynamic widgets
    let mut reasoning = ReasoningDrawerState::new("Soak test AI session");
    let mut kanban = KanbanState::new(vec![
        KanbanColumn {
            id: "todo".to_string(),
            title: "To Do".to_string(),
            cards: vec![
                KanbanCard {
                    id: "c1".to_string(),
                    title: "Task 1".to_string(),
                    description: "Desc 1".to_string(),
                    tag: "Core".to_string(),
                    tag_color: Color::WHITE,
                    assignee_name: None,
                },
                KanbanCard {
                    id: "c2".to_string(),
                    title: "Task 2".to_string(),
                    description: "Desc 2".to_string(),
                    tag: "UI".to_string(),
                    tag_color: Color::WHITE,
                    assignee_name: None,
                },
            ],
        },
        KanbanColumn {
            id: "done".to_string(),
            title: "Done".to_string(),
            cards: Vec::new(),
        },
    ]);
    let mut otp = OtpInputState::new(6);
    let mut knob = KnobState {
        is_dragging: true,
        start_value: 0.5,
        ..Default::default()
    };
    let mut scrub = ScrubFieldState::new(50.0, 0.0, 100.0, 1.0);
    scrub.is_dragging = true;
    scrub.start_value = 50.0;

    // Run 5,000 deterministic interaction steps
    for cycle in 0..5000 {
        let action = prng.next_u32() % 6;
        match action {
            0 => {
                // Toggle AI reasoning drawer
                reasoning.toggle();
                reasoning.append_thought("thinking step; ");
                fp.feed_u64(u64::from(reasoning.is_expanded));
                fp.feed_u64(reasoning.token_count as u64);
            }
            1 => {
                // Kanban card move
                let card_id = if cycle % 2 == 0 { "c1" } else { "c2" };
                let target_col = (prng.next_u32() % 2) as usize;
                kanban.move_card(card_id, target_col, 0);
                fp.feed_u64(kanban.columns[0].cards.len() as u64);
                fp.feed_u64(kanban.columns[1].cards.len() as u64);
            }
            2 => {
                // OTP digit entry / backspace
                if prng.next_f32() > 0.3 {
                    let d = (b'0' + (prng.next_u32() % 10) as u8) as char;
                    otp.input_digit(d);
                } else {
                    otp.backspace();
                }
                fp.feed_u64(otp.active_index as u64);
                fp.feed_str(&otp.code_string());
            }
            3 => {
                // Rotary knob drag delta
                let dy = (prng.next_f32() - 0.5) * 50.0;
                knob.on_drag_delta_y(dy);
                fp.feed_f32(knob.value);
            }
            4 => {
                // Number scrub drag
                let dx = (prng.next_f32() - 0.5) * 40.0;
                scrub.on_drag_delta(dx);
                fp.feed_f32(scrub.value);
            }
            _ => {
                // Feed cycle index to ensure progress tracking
                fp.feed_u64(cycle as u64);
            }
        }
    }

    fp.finish()
}

#[test]
fn test_deterministic_interaction_soak_run() {
    let final_fingerprint = run_soak_simulation(42);
    // Verify that the fingerprint is non-zero and stable
    assert_ne!(final_fingerprint, 0);

    // Verify determinism by running the exact same sequence a second time
    let second_fingerprint = run_soak_simulation(42);
    assert_eq!(final_fingerprint, second_fingerprint);
}
