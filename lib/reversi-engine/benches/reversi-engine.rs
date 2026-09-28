/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! Legal move generation and bounded search at different game stages.

#![allow(missing_docs)]

use std::hint::black_box;

use criterion::{Criterion, criterion_group, criterion_main};
use reversi_engine::{Othello, Player};

fn position(plies: usize) -> (Othello, Player) {
    let mut board = Othello::default();
    let mut player = Player::Black;
    for _ in 0..plies {
        if let Some(movement) = board.valid_moves(player).first() {
            assert!(board.make_move(player, *movement));
        } else if !board.has_valid_move(player.other()) {
            break;
        }
        player = player.other();
    }
    (board, player)
}

fn engine(c: &mut Criterion) {
    for (name, plies) in [("opening", 0), ("middle", 24), ("late", 48)] {
        let (board, player) = position(plies);
        assert!(board.has_valid_move(player));

        let mut moves = c.benchmark_group("reversi_legal_moves");
        moves.bench_function(name, |b| {
            b.iter(|| black_box(black_box(board).valid_moves(black_box(player))));
        });
        moves.finish();

        let mut search = c.benchmark_group("reversi_search");
        search.bench_function(name, |b| {
            b.iter(|| {
                black_box(black_box(board).compute_move_cancellable(
                    black_box(player),
                    5,
                    100_000,
                    &|| false,
                ))
            });
        });
        search.finish();
    }
}

criterion_group!(benches, engine);
criterion_main!(benches);
