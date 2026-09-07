/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::time::Duration;

use crate::Game;

const WARMUP: usize = 30;
const SAMPLES: usize = 240;

pub(crate) struct RenderProfile {
    frames: usize,
    samples: Vec<Duration>,
}

impl RenderProfile {
    pub(crate) fn new(game: &mut Game) -> Self {
        // A reproducible position after 32 legal moves, without AI search cost.
        for _ in 0..32 {
            let moves = game.board.valid_move_bits(game.turn);
            if moves == 0 {
                break;
            }
            let index = moves.trailing_zeros() as usize;
            assert!(game.play(index / 8, index % 8));
        }
        Self {
            frames: 0,
            samples: Vec::with_capacity(SAMPLES),
        }
    }

    pub(crate) fn record(&mut self, duration: Duration) -> bool {
        self.frames += 1;
        if self.frames > WARMUP {
            self.samples.push(duration);
        }
        if self.samples.len() != SAMPLES {
            return false;
        }
        self.samples.sort_unstable();
        let median = self.samples[SAMPLES / 2].as_secs_f64() * 1_000_000.0;
        let p95 = self.samples[SAMPLES * 95 / 100].as_secs_f64() * 1_000_000.0;
        println!(
            "render: {SAMPLES} frames after {WARMUP} warmup, median {median:.1} us, p95 {p95:.1} us"
        );
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn profile_has_a_fixed_position_and_sample_count() {
        let mut game = Game::default();
        let mut profile = RenderProfile::new(&mut game);
        assert_eq!(game.revision, 32);
        assert!(!game.thinking);
        for _ in 0..WARMUP + SAMPLES - 1 {
            assert!(!profile.record(Duration::from_micros(1)));
        }
        assert!(profile.record(Duration::from_micros(1)));
    }
}
