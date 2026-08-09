/*
 * Copyright (c) 2014 Gabriele Cirulli
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! Native implementation of the original 2048 game rules.

use std::time::{SystemTime, UNIX_EPOCH};

const SIDE: usize = 4;
const CELL_COUNT: usize = SIDE * SIDE;

/// Direction in which the board moves.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Direction {
    Up,
    Right,
    Down,
    Left,
}

/// One tile's animated journey during a move.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct TileMotion {
    pub(crate) value: u32,
    pub(crate) from: usize,
    pub(crate) to: usize,
}

/// Visual information produced by a successful move.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct MoveAnimation {
    pub(crate) motions: Vec<TileMotion>,
    pub(crate) merged: Vec<usize>,
    pub(crate) new_tile: usize,
    pub(crate) score_added: u32,
}

/// Serializable 2048 game state.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Game {
    pub(crate) cells: [u32; CELL_COUNT],
    pub(crate) score: u32,
    pub(crate) best: u32,
    pub(crate) won: bool,
    pub(crate) keep_playing: bool,
    rng: u64,
}

impl Game {
    /// Create a fresh game containing two random tiles.
    pub(crate) fn new(best: u32) -> Self {
        let mut bytes = [0; 8];
        let random = getrandom::fill(&mut bytes)
            .map(|()| u64::from_ne_bytes(bytes))
            .unwrap_or_else(|_| {
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .map_or(1, |time| time.as_nanos() as u64)
            });
        let mut game = Self {
            cells: [0; CELL_COUNT],
            score: 0,
            best,
            won: false,
            keep_playing: false,
            rng: random.max(1),
        };
        game.add_random_tile();
        game.add_random_tile();
        game
    }

    /// Restore a state read from native storage.
    pub(crate) const fn from_state(
        cells: [u32; CELL_COUNT],
        score: u32,
        best: u32,
        won: bool,
        keep_playing: bool,
    ) -> Self {
        Self {
            cells,
            score,
            best,
            won,
            keep_playing,
            rng: 0x9e37_79b9_7f4a_7c15,
        }
    }

    /// Whether input should be blocked by a win or loss.
    pub(crate) fn is_terminated(&self) -> bool {
        self.is_over() || (self.won && !self.keep_playing)
    }

    /// Whether no legal moves remain.
    pub(crate) fn is_over(&self) -> bool {
        if self.cells.contains(&0) {
            return false;
        }
        for row in 0..SIDE {
            for col in 0..SIDE {
                let value = self.cells[index(row, col)];
                if col + 1 < SIDE && self.cells[index(row, col + 1)] == value {
                    return false;
                }
                if row + 1 < SIDE && self.cells[index(row + 1, col)] == value {
                    return false;
                }
            }
        }
        true
    }

    /// Continue after reaching 2048.
    pub(crate) const fn continue_game(&mut self) {
        self.keep_playing = true;
    }

    /// Apply one move and return its animation data when the board changed.
    pub(crate) fn move_tiles(&mut self, direction: Direction) -> Option<MoveAnimation> {
        if self.is_terminated() {
            return None;
        }
        let before = self.cells;
        let mut next = [0; CELL_COUNT];
        let mut motions = Vec::new();
        let mut merged = Vec::new();
        let mut score_added = 0;

        for line in 0..SIDE {
            let indices = line_indices(direction, line);
            let tiles: Vec<_> = indices
                .iter()
                .copied()
                .filter(|&cell| before[cell] != 0)
                .collect();
            let mut source = 0;
            let mut destination = 0;
            while source < tiles.len() {
                let first = tiles[source];
                let target = indices[destination];
                if source + 1 < tiles.len() && before[first] == before[tiles[source + 1]] {
                    let second = tiles[source + 1];
                    let value = before[first] * 2;
                    next[target] = value;
                    motions.push(TileMotion {
                        value: before[first],
                        from: first,
                        to: target,
                    });
                    motions.push(TileMotion {
                        value: before[second],
                        from: second,
                        to: target,
                    });
                    merged.push(target);
                    score_added += value;
                    if value == 2048 {
                        self.won = true;
                    }
                    source += 2;
                } else {
                    next[target] = before[first];
                    motions.push(TileMotion {
                        value: before[first],
                        from: first,
                        to: target,
                    });
                    source += 1;
                }
                destination += 1;
            }
        }

        if next == before {
            return None;
        }
        self.cells = next;
        self.score += score_added;
        self.best = self.best.max(self.score);
        let new_tile = self.add_random_tile();
        Some(MoveAnimation {
            motions,
            merged,
            new_tile,
            score_added,
        })
    }

    fn add_random_tile(&mut self) -> usize {
        let available: Vec<_> = self
            .cells
            .iter()
            .enumerate()
            .filter_map(|(cell, &value)| (value == 0).then_some(cell))
            .collect();
        let cell = available[self.random_index(available.len())];
        self.cells[cell] = if self.random_index(10) == 0 { 4 } else { 2 };
        cell
    }

    const fn random_index(&mut self, length: usize) -> usize {
        // xorshift64* is more than sufficient for selecting game tiles.
        self.rng ^= self.rng >> 12;
        self.rng ^= self.rng << 25;
        self.rng ^= self.rng >> 27;
        ((self.rng.wrapping_mul(0x2545_f491_4f6c_dd1d)) % length as u64) as usize
    }
}

const fn index(row: usize, col: usize) -> usize {
    row * SIDE + col
}

const fn line_indices(direction: Direction, line: usize) -> [usize; SIDE] {
    match direction {
        Direction::Up => [
            index(0, line),
            index(1, line),
            index(2, line),
            index(3, line),
        ],
        Direction::Right => [
            index(line, 3),
            index(line, 2),
            index(line, 1),
            index(line, 0),
        ],
        Direction::Down => [
            index(3, line),
            index(2, line),
            index(1, line),
            index(0, line),
        ],
        Direction::Left => [
            index(line, 0),
            index(line, 1),
            index(line, 2),
            index(line, 3),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn game(cells: [u32; CELL_COUNT]) -> Game {
        Game::from_state(cells, 0, 0, false, false)
    }

    #[test]
    fn merges_each_tile_only_once() {
        let mut game = game([2, 2, 2, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
        game.move_tiles(Direction::Left);
        assert_eq!(&game.cells[..4], &[4, 4, 0, 0]);
        assert_eq!(game.score, 8);
    }

    #[test]
    fn merge_order_matches_original_game() {
        let mut game = game([2, 2, 4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
        game.move_tiles(Direction::Right);
        assert_eq!(&game.cells[..4], &[0, 0, 4, 4]);
    }

    #[test]
    fn rejects_unchanged_move() {
        let mut game = game([2, 4, 8, 16, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
        assert!(game.move_tiles(Direction::Left).is_none());
    }

    #[test]
    fn detects_game_over() {
        let game = game([2, 4, 2, 4, 4, 2, 4, 2, 2, 4, 2, 4, 4, 2, 4, 2]);
        assert!(game.is_over());
    }

    #[test]
    fn detects_available_adjacent_match() {
        let game = game([2, 4, 2, 4, 4, 2, 4, 2, 2, 4, 2, 4, 4, 2, 2, 4]);
        assert!(!game.is_over());
    }
}
