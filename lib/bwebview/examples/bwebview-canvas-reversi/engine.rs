/*
 * Bitboard Othello engine derived from Hans Wennborg's C implementation:
 * https://www.hanshq.net/othello.html
 *
 * Adapted to safe, idiomatic Rust for the bwebview Canvas Reversi example.
 */

use std::fmt::{self, Display, Formatter};
use std::str::FromStr;

const BOARD_CELLS: usize = 64;
const DIRECTIONS: usize = 8;
const WIN_BONUS: i32 = 1 << 20;
const CORNER_MASK: u64 = 0x8100_0000_0000_0081;

const SHIFT_MASKS: [u64; DIRECTIONS] = [
    0x7f7f_7f7f_7f7f_7f7f,
    0x007f_7f7f_7f7f_7f7f,
    u64::MAX,
    0x00fe_fefe_fefe_fefe,
    0xfefe_fefe_fefe_fefe,
    0xfefe_fefe_fefe_fe00,
    u64::MAX,
    0x7f7f_7f7f_7f7f_7f00,
];
const LEFT_SHIFTS: [u32; DIRECTIONS] = [0, 0, 0, 0, 1, 9, 8, 7];
const RIGHT_SHIFTS: [u32; DIRECTIONS] = [1, 9, 8, 7, 0, 0, 0, 0];

/// A player and disk color.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Player {
    Black,
    White,
}

impl Player {
    pub(crate) const fn other(self) -> Self {
        match self {
            Self::Black => Self::White,
            Self::White => Self::Black,
        }
    }

    const fn index(self) -> usize {
        self as usize
    }
}

/// Contents of one board cell.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum CellState {
    Black,
    White,
    Empty,
}

/// A zero-indexed board move.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Move {
    pub(crate) row: usize,
    pub(crate) col: usize,
}

impl Move {
    const fn from_index(index: usize) -> Self {
        Self {
            row: index / 8,
            col: index % 8,
        }
    }

    const fn index(self) -> usize {
        self.row * 8 + self.col
    }
}

/// Compact Othello state represented by one bitboard per player.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Othello {
    disks: [u64; 2],
}

impl Default for Othello {
    fn default() -> Self {
        let mut game = Self { disks: [0; 2] };
        game.set_cell_state(3, 4, CellState::Black);
        game.set_cell_state(4, 3, CellState::Black);
        game.set_cell_state(3, 3, CellState::White);
        game.set_cell_state(4, 4, CellState::White);
        game
    }
}

#[allow(dead_code)]
impl Othello {
    pub(crate) fn cell_state(self, row: usize, col: usize) -> CellState {
        let mask = cell_mask(row, col);
        if self.disks[Player::Black.index()] & mask != 0 {
            CellState::Black
        } else if self.disks[Player::White.index()] & mask != 0 {
            CellState::White
        } else {
            CellState::Empty
        }
    }

    pub(crate) fn set_cell_state(&mut self, row: usize, col: usize, state: CellState) {
        let mask = cell_mask(row, col);
        self.disks[0] &= !mask;
        self.disks[1] &= !mask;
        match state {
            CellState::Black => self.disks[Player::Black.index()] |= mask,
            CellState::White => self.disks[Player::White.index()] |= mask,
            CellState::Empty => {}
        }
    }

    pub(crate) const fn score(self, player: Player) -> u32 {
        self.disks[player.index()].count_ones()
    }

    pub(crate) fn valid_moves(self, player: Player) -> Vec<Move> {
        let mut bits = self.valid_move_bits(player);
        let mut moves = Vec::with_capacity(bits.count_ones() as usize);
        while bits != 0 {
            let index = bits.trailing_zeros() as usize;
            moves.push(Move::from_index(index));
            bits &= bits - 1;
        }
        moves
    }

    pub(crate) fn has_valid_move(self, player: Player) -> bool {
        self.valid_move_bits(player) != 0
    }

    pub(crate) fn is_valid_move(self, player: Player, movement: Move) -> bool {
        movement.row < 8
            && movement.col < 8
            && self.valid_move_bits(player) & (1_u64 << movement.index()) != 0
    }

    pub(crate) fn make_move(&mut self, player: Player, movement: Move) -> bool {
        if !self.is_valid_move(player, movement) {
            return false;
        }
        let (my_disks, opponent_disks) = player_disks_mut(&mut self.disks, player);
        resolve_move(my_disks, opponent_disks, movement.index());
        true
    }

    pub(crate) fn evaluate(self, player: Player) -> i32 {
        let my_disks = self.disks[player.index()];
        let opponent_disks = self.disks[player.other().index()];
        evaluate(
            my_disks,
            opponent_disks,
            generate_moves(my_disks, opponent_disks),
            generate_moves(opponent_disks, my_disks),
        )
    }

    pub(crate) fn negamax(self, player: Player, depth: u32) -> i32 {
        let mut evaluations = 0;
        negamax(
            self.disks[player.index()],
            self.disks[player.other().index()],
            depth,
            -i32::MAX,
            i32::MAX,
            None,
            &mut evaluations,
        )
    }

    /// Compute a strong move using the source engine's depth and evaluation budget.
    pub(crate) fn compute_move(self, player: Player) -> Option<Move> {
        self.compute_move_with_budget(player, 8, 500_000)
    }

    pub(crate) fn compute_move_with_budget(
        self,
        player: Player,
        start_depth: u32,
        evaluation_budget: u32,
    ) -> Option<Move> {
        if !self.has_valid_move(player) {
            return None;
        }
        iterative_negamax(
            self.disks[player.index()],
            self.disks[player.other().index()],
            start_depth.max(1),
            evaluation_budget.max(1),
        )
        .map(Move::from_index)
    }

    pub(crate) fn random_move(self, player: Player, seed: &mut u64) -> Option<Move> {
        let moves = self.valid_moves(player);
        if moves.is_empty() {
            return None;
        }
        *seed ^= *seed << 13;
        *seed ^= *seed >> 7;
        *seed ^= *seed << 17;
        Some(moves[*seed as usize % moves.len()])
    }

    fn valid_move_bits(self, player: Player) -> u64 {
        generate_moves(
            self.disks[player.index()],
            self.disks[player.other().index()],
        )
    }
}

fn cell_mask(row: usize, col: usize) -> u64 {
    assert!(row < 8 && col < 8, "cell must be on the board");
    1_u64 << (row * 8 + col)
}

#[inline(always)]
const fn shift(disks: u64, direction: usize) -> u64 {
    if direction < DIRECTIONS / 2 {
        (disks >> RIGHT_SHIFTS[direction]) & SHIFT_MASKS[direction]
    } else {
        (disks << LEFT_SHIFTS[direction]) & SHIFT_MASKS[direction]
    }
}

#[inline]
fn generate_moves(my_disks: u64, opponent_disks: u64) -> u64 {
    debug_assert_eq!(my_disks & opponent_disks, 0);
    let empty = !(my_disks | opponent_disks);
    let mut legal = 0;
    for direction in 0..DIRECTIONS {
        let mut candidates = shift(my_disks, direction) & opponent_disks;
        for _ in 0..5 {
            candidates |= shift(candidates, direction) & opponent_disks;
        }
        legal |= shift(candidates, direction) & empty;
    }
    legal
}

#[inline]
fn resolve_move(my_disks: &mut u64, opponent_disks: &mut u64, index: usize) {
    let new_disk = 1_u64 << index;
    *my_disks |= new_disk;
    let mut captured = 0;
    for direction in 0..DIRECTIONS {
        let mut candidates = shift(new_disk, direction) & *opponent_disks;
        for _ in 0..5 {
            candidates |= shift(candidates, direction) & *opponent_disks;
        }
        if shift(candidates, direction) & *my_disks != 0 {
            captured |= candidates;
        }
    }
    debug_assert_ne!(captured, 0);
    *my_disks ^= captured;
    *opponent_disks ^= captured;
}

const fn player_disks_mut(disks: &mut [u64; 2], player: Player) -> (&mut u64, &mut u64) {
    let [black, white] = disks;
    match player {
        Player::Black => (black, white),
        Player::White => (white, black),
    }
}

#[inline]
fn frontier_disks(my_disks: u64, opponent_disks: u64) -> (u64, u64) {
    let empty = !(my_disks | opponent_disks);
    let mut my_frontier = 0;
    let mut opponent_frontier = 0;
    for direction in 0..DIRECTIONS {
        let adjacent = shift(empty, direction);
        my_frontier |= adjacent & my_disks;
        opponent_frontier |= adjacent & opponent_disks;
    }
    (my_frontier, opponent_frontier)
}

#[inline]
fn evaluate(my_disks: u64, opponent_disks: u64, my_moves: u64, opponent_moves: u64) -> i32 {
    if my_moves == 0 && opponent_moves == 0 {
        return (my_disks.count_ones() as i32 - opponent_disks.count_ones() as i32) * WIN_BONUS;
    }
    let (my_frontier, opponent_frontier) = frontier_disks(my_disks, opponent_disks);
    let corners = (my_disks & CORNER_MASK).count_ones() as i32
        - (opponent_disks & CORNER_MASK).count_ones() as i32;
    let mobility = my_moves.count_ones() as i32 - opponent_moves.count_ones() as i32;
    let frontier = my_frontier.count_ones() as i32 - opponent_frontier.count_ones() as i32;
    corners * 16 + mobility * 2 - frontier
}

#[allow(clippy::too_many_arguments)]
fn negamax(
    my_disks: u64,
    opponent_disks: u64,
    depth: u32,
    mut alpha: i32,
    beta: i32,
    mut best_move: Option<&mut usize>,
    evaluations: &mut u32,
) -> i32 {
    let my_moves = generate_moves(my_disks, opponent_disks);
    let opponent_moves = generate_moves(opponent_disks, my_disks);
    if my_moves == 0 && opponent_moves != 0 {
        return -negamax(
            opponent_disks,
            my_disks,
            depth,
            -beta,
            -alpha,
            best_move,
            evaluations,
        );
    }
    if depth == 0 || (my_moves == 0 && opponent_moves == 0) {
        *evaluations += 1;
        return evaluate(my_disks, opponent_disks, my_moves, opponent_moves);
    }

    let mut best = -i32::MAX;
    let mut remaining_moves = my_moves;
    while remaining_moves != 0 {
        let index = remaining_moves.trailing_zeros() as usize;
        remaining_moves &= remaining_moves - 1;
        let (mut next_mine, mut next_opponent) = (my_disks, opponent_disks);
        resolve_move(&mut next_mine, &mut next_opponent, index);
        let score = -negamax(
            next_opponent,
            next_mine,
            depth - 1,
            -beta,
            -alpha,
            None,
            evaluations,
        );
        if score > best {
            best = score;
            if let Some(output) = best_move.as_deref_mut() {
                *output = index;
            }
            alpha = alpha.max(score);
            if alpha >= beta {
                break;
            }
        }
    }
    best
}

fn iterative_negamax(
    my_disks: u64,
    opponent_disks: u64,
    start_depth: u32,
    evaluation_budget: u32,
) -> Option<usize> {
    let mut depth = start_depth;
    let mut evaluations = 0;
    let mut best_move = None;
    while evaluations < evaluation_budget {
        let mut iteration_move = 0;
        let score = negamax(
            my_disks,
            opponent_disks,
            depth,
            -i32::MAX,
            i32::MAX,
            Some(&mut iteration_move),
            &mut evaluations,
        );
        best_move = Some(iteration_move);
        if score.abs() >= WIN_BONUS {
            break;
        }
        depth += 1;
    }
    best_move
}

impl Display for Othello {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        writeln!(formatter, " abcdefgh ")?;
        for row in 0..8 {
            write!(formatter, "{}", row + 1)?;
            for col in 0..8 {
                formatter.write_str(match self.cell_state(row, col) {
                    CellState::Black => "x",
                    CellState::White => "o",
                    CellState::Empty => ".",
                })?;
            }
            writeln!(formatter, "{}", row + 1)?;
        }
        writeln!(formatter, " abcdefgh ")
    }
}

impl FromStr for Othello {
    type Err = &'static str;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let cells = value.chars().filter(|c| matches!(c, '.' | 'x' | 'o'));
        let mut game = Self { disks: [0; 2] };
        let mut count = 0;
        for (index, cell) in cells.enumerate() {
            if index >= BOARD_CELLS {
                return Err("board contains more than 64 cells");
            }
            let state = match cell {
                'x' => CellState::Black,
                'o' => CellState::White,
                _ => CellState::Empty,
            };
            game.set_cell_state(index / 8, index % 8, state);
            count += 1;
        }
        if count != BOARD_CELLS {
            return Err("board must contain exactly 64 cells");
        }
        Ok(game)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initial_moves_match_reference() {
        assert_eq!(
            Othello::default().valid_moves(Player::Black),
            [
                Move { row: 2, col: 3 },
                Move { row: 3, col: 2 },
                Move { row: 4, col: 5 },
                Move { row: 5, col: 4 }
            ]
        );
    }

    #[test]
    fn board_string_round_trip() {
        let game = Othello::default();
        assert_eq!(game.to_string().parse(), Ok(game));
    }

    #[test]
    fn invalid_move_does_not_mutate_board() {
        let mut game = Othello::default();
        let original = game;
        assert!(!game.make_move(Player::Black, Move { row: 0, col: 0 }));
        assert_eq!(game, original);
    }

    #[test]
    fn valid_move_flips_disks() {
        let mut game = Othello::default();
        assert!(game.make_move(Player::Black, Move { row: 2, col: 3 }));
        assert_eq!(game.score(Player::Black), 4);
        assert_eq!(game.score(Player::White), 1);
    }

    #[test]
    fn shallow_ai_returns_a_legal_move() {
        let game = Othello::default();
        let movement = game.compute_move_with_budget(Player::Black, 2, 32).unwrap();
        assert!(game.is_valid_move(Player::Black, movement));
    }

    #[test]
    fn full_ai_returns_a_legal_move() {
        let mut game = Othello::default();
        assert!(game.make_move(Player::Black, Move { row: 2, col: 3 }));
        let movement = game.compute_move(Player::White).unwrap();
        assert!(game.is_valid_move(Player::White, movement));
    }
}
