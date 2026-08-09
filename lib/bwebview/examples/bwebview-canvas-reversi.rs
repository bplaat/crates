/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! A native Canvas Reversi game against a simple AI.

use std::cell::RefCell;
use std::f32::consts::TAU;
use std::rc::Rc;

#[cfg(target_os = "macos")]
use bwebview::{
    Accelerator, KeyCode, MenuBarBuilder, MenuBuilder, MenuItem, MenuItemRole, Modifiers,
};
use bwebview::{
    CanvasBuilder, CanvasEvent, CanvasRenderingContext2d, Color, CursorIcon, Event,
    EventLoopBuilder, FontWeight, LogicalSize, MouseEvent, TextAlign, TextBaseline, Theme,
    WindowBuilder, WindowEvent,
};

#[path = "bwebview-canvas-reversi/engine.rs"]
mod engine;
use engine::{CellState, Move, Othello, Player};

const WINDOW_WIDTH: f32 = 600.0;
const WINDOW_HEIGHT: f32 = 600.0;
const MIN_WINDOW_WIDTH: f32 = 360.0;
const MIN_WINDOW_HEIGHT: f32 = 360.0;
const GRID_OFFSET: f32 = 31.0;
const CELL: f32 = 52.0;
const CELL_GAP: f32 = 2.0;
const CELL_STEP: f32 = CELL + CELL_GAP;
const BOARD_X: f32 = GRID_OFFSET + CELL_STEP;
const BOARD_Y: f32 = GRID_OFFSET + CELL_STEP;
const BOARD_BACKGROUND_X: f32 = GRID_OFFSET + CELL;
const BOARD_BACKGROUND_Y: f32 = GRID_OFFSET + CELL;
const BOARD_BACKGROUND_SIZE: f32 = CELL * 8.0 + CELL_GAP * 9.0;

#[derive(Clone, Copy)]
struct Palette {
    background: u32,
    text: Color,
    grid: Color,
    board: Color,
    board_hover: Color,
    move_hint: Color,
}

impl Palette {
    const fn for_theme(theme: Theme) -> Self {
        match theme {
            Theme::Light => Self {
                background: 0xededed,
                text: Color::rgb(18, 18, 18),
                grid: Color::rgb(20, 20, 20),
                board: Color::rgb(27, 137, 86),
                board_hover: Color::rgb(37, 165, 105),
                move_hint: Color::rgba(4, 49, 31, 130),
            },
            Theme::Dark => Self {
                background: 0x1b201d,
                text: Color::rgb(235, 240, 237),
                grid: Color::rgb(72, 79, 75),
                board: Color::rgb(24, 126, 80),
                board_hover: Color::rgb(34, 151, 97),
                move_hint: Color::rgba(210, 239, 226, 155),
            },
        }
    }
}

#[derive(Clone)]
struct Game {
    board: Othello,
    turn: Player,
    hover: Option<(usize, usize)>,
    game_over: bool,
    window_size: LogicalSize,
    top_inset: f32,
    theme: Theme,
}
impl Default for Game {
    fn default() -> Self {
        Self::new(Theme::Light, 0.0)
    }
}

impl Game {
    fn new(theme: Theme, top_inset: f32) -> Self {
        Self {
            board: Othello::default(),
            turn: Player::Black,
            hover: None,
            game_over: false,
            window_size: LogicalSize::new(WINDOW_WIDTH, WINDOW_HEIGHT),
            top_inset,
            theme,
        }
    }
}

#[derive(Clone, Copy)]
struct GameLayout {
    scale: f32,
    offset_x: f32,
    offset_y: f32,
    window_size: LogicalSize,
}

impl GameLayout {
    fn new(window_size: LogicalSize, top_inset: f32) -> Self {
        let top_inset = top_inset.clamp(0.0, window_size.height);
        let usable_height = window_size.height - top_inset;
        let scale = (window_size.width / WINDOW_WIDTH)
            .min(usable_height / WINDOW_HEIGHT)
            .max(0.01);
        Self {
            scale,
            offset_x: (window_size.width - WINDOW_WIDTH * scale) / 2.0,
            offset_y: top_inset + (usable_height - WINDOW_HEIGHT * scale) / 2.0,
            window_size,
        }
    }

    fn game_point(self, event: &MouseEvent) -> Option<(f32, f32)> {
        let x = (event.client_x - self.offset_x) / self.scale;
        let y = (event.client_y - self.offset_y) / self.scale;
        ((0.0..WINDOW_WIDTH).contains(&x) && (0.0..WINDOW_HEIGHT).contains(&y)).then_some((x, y))
    }
}

impl Game {
    fn play(&mut self, row: usize, col: usize) -> bool {
        if !self.board.make_move(self.turn, Move { row, col }) {
            return false;
        }
        self.turn = self.turn.other();
        self.advance_passes();
        true
    }
    fn advance_passes(&mut self) {
        if !self.board.has_valid_move(self.turn) {
            self.turn = self.turn.other();
            if !self.board.has_valid_move(self.turn) {
                self.game_over = true
            }
        }
    }
    fn ai_move(&mut self) {
        while self.turn == Player::White && !self.game_over {
            let Some(movement) = self.board.compute_move(Player::White) else {
                self.advance_passes();
                break;
            };
            self.play(movement.row, movement.col);
        }
    }
    fn status(&self) -> String {
        if self.game_over {
            let (b, w) = (
                self.board.score(Player::Black),
                self.board.score(Player::White),
            );
            if b > w {
                format!("Human wins {b}-{w}!")
            } else if w > b {
                format!("Computer wins {w}-{b}!")
            } else {
                "Draw!".into()
            }
        } else if self.turn == Player::Black {
            "Human's move.".into()
        } else {
            "Computer's move...".into()
        }
    }
}

fn main() {
    let event_loop_builder = EventLoopBuilder::new();
    #[cfg(target_os = "macos")]
    let event_loop_builder = event_loop_builder.macos_set_menu(
        MenuBarBuilder::new().menu(
            MenuBuilder::new("File")
                .item(
                    MenuItem::new("New Game", "game.new")
                        .accelerator(Accelerator::new(Modifiers::COMMAND, KeyCode::KeyN)),
                )
                .separator()
                .role(MenuItemRole::Close),
        ),
    );
    let event_loop = event_loop_builder.build();
    let initial_theme = event_loop.theme();
    let initial_palette = Palette::for_theme(initial_theme);
    #[allow(unused_mut)]
    let mut window_builder = WindowBuilder::new()
        .title("Othello")
        .size(LogicalSize::new(WINDOW_WIDTH, WINDOW_HEIGHT))
        .min_size(LogicalSize::new(MIN_WINDOW_WIDTH, MIN_WINDOW_HEIGHT))
        .resizable(true)
        .background_color(initial_palette.background)
        .center();
    #[cfg(target_os = "macos")]
    {
        window_builder = window_builder.macos_titlebar_style(bwebview::MacosTitlebarStyle::Hidden);
    }
    let mut window = window_builder.build();
    #[cfg(target_os = "macos")]
    let top_inset = window.macos_titlebar_size().height;
    #[cfg(not(target_os = "macos"))]
    let top_inset = 0.0;
    #[cfg(target_os = "macos")]
    {
        window.set_size(LogicalSize::new(WINDOW_WIDTH, WINDOW_HEIGHT + top_inset));
        window.set_min_size(LogicalSize::new(
            MIN_WINDOW_WIDTH,
            MIN_WINDOW_HEIGHT + top_inset,
        ));
    }
    let mut canvas = CanvasBuilder::new(&window).build();
    let game = Rc::new(RefCell::new(Game::new(initial_theme, top_inset)));
    let events = game.clone();
    event_loop.run(move |event| match event {
        Event::Canvas(CanvasEvent::Draw(context)) => {
            let mut game = events.borrow_mut();
            game.window_size = LogicalSize::new(context.width(), context.height());
            let layout = GameLayout::new(game.window_size, game.top_inset);
            draw(context, &game, layout);
        }
        Event::Window(WindowEvent::Resize(size)) => {
            let mut game = events.borrow_mut();
            game.window_size = size;
            #[cfg(target_os = "macos")]
            {
                game.top_inset = window.macos_titlebar_size().height;
            }
            canvas.request_redraw();
        }
        Event::Window(WindowEvent::ThemeChange(theme)) => {
            events.borrow_mut().theme = theme;
            window.set_background_color(Palette::for_theme(theme).background);
            canvas.request_redraw();
        }
        Event::Window(WindowEvent::MouseMove(event)) => {
            let hover = {
                let game = events.borrow();
                board_cell(&event, GameLayout::new(game.window_size, game.top_inset))
            };
            let mut game = events.borrow_mut();
            if game.hover != hover {
                game.hover = hover;
                let pointer = hover.is_some_and(|(row, col)| {
                    game.turn == Player::Black
                        && game.board.is_valid_move(Player::Black, Move { row, col })
                });
                drop(game);
                window.set_cursor(if pointer {
                    CursorIcon::Pointer
                } else {
                    CursorIcon::Default
                });
                canvas.request_redraw();
            }
        }
        Event::Window(WindowEvent::MouseLeave(_)) => {
            events.borrow_mut().hover = None;
            window.set_cursor(CursorIcon::Default);
            canvas.request_redraw();
        }
        Event::Window(WindowEvent::MouseDown(event)) => {
            let game = events.borrow();
            let layout = GameLayout::new(game.window_size, game.top_inset);
            drop(game);
            if let Some((r, c)) = board_cell(&event, layout) {
                let mut game = events.borrow_mut();
                if game.turn == Player::Black && game.play(r, c) {
                    game.ai_move();
                    drop(game);
                    canvas.request_redraw();
                }
            }
        }
        Event::Window(WindowEvent::KeyDown(event)) if event.key.eq_ignore_ascii_case("r") => {
            reset_game(&events);
            canvas.request_redraw();
        }
        #[cfg(target_os = "macos")]
        Event::MacosMenuItem(action) if action == "game.new" => {
            reset_game(&events);
            canvas.request_redraw();
        }
        _ => {}
    })
}

fn reset_game(game: &RefCell<Game>) {
    let (window_size, top_inset, theme) = {
        let game = game.borrow();
        (game.window_size, game.top_inset, game.theme)
    };
    *game.borrow_mut() = Game {
        window_size,
        ..Game::new(theme, top_inset)
    };
}

fn board_cell(event: &MouseEvent, layout: GameLayout) -> Option<(usize, usize)> {
    let (x, y) = layout.game_point(event)?;
    let col = ((x - BOARD_X) / CELL_STEP).floor() as isize;
    let row = ((y - BOARD_Y) / CELL_STEP).floor() as isize;
    ((0..8).contains(&row) && (0..8).contains(&col)).then_some((row as usize, col as usize))
}

fn draw(ctx: &mut CanvasRenderingContext2d<'_>, game: &Game, layout: GameLayout) {
    let palette = Palette::for_theme(game.theme);
    ctx.set_fill_style(Color::from_rgb(palette.background));
    ctx.fill_rect(
        0.0,
        0.0,
        layout.window_size.width,
        layout.window_size.height,
    );
    ctx.save();
    ctx.translate(layout.offset_x, layout.offset_y);
    ctx.scale(layout.scale, layout.scale);
    ctx.set_fill_style(Color::from_rgb(palette.background));
    ctx.fill_rect(0.0, 0.0, WINDOW_WIDTH, WINDOW_HEIGHT);
    ctx.set_text_align(TextAlign::Center);
    ctx.set_text_baseline(TextBaseline::Middle);
    ctx.set_font("sans-serif", 19.0);
    ctx.set_font_weight(FontWeight::Bold);
    ctx.set_fill_style(palette.text);
    for col in 0..8 {
        ctx.fill_text(
            ((b'A' + col as u8) as char).to_string(),
            BOARD_X + col as f32 * CELL_STEP + CELL / 2.0,
            GRID_OFFSET + CELL / 2.0,
        );
    }
    for row in 0..8 {
        ctx.fill_text(
            (row + 1).to_string(),
            GRID_OFFSET + CELL / 2.0,
            BOARD_Y + row as f32 * CELL_STEP + CELL / 2.0,
        );
    }

    // Match the original macOS 10x10 composition: a black grid backing with
    // eight green cells in its center, coordinate gutters above and left, and
    // the game status occupying the bottom gutter.
    ctx.set_fill_style(palette.grid);
    ctx.fill_rect(
        BOARD_BACKGROUND_X,
        BOARD_BACKGROUND_Y,
        BOARD_BACKGROUND_SIZE,
        BOARD_BACKGROUND_SIZE,
    );
    let legal_moves = game.board.valid_moves(Player::Black);
    for row in 0..8 {
        for col in 0..8 {
            let x = BOARD_X + col as f32 * CELL_STEP;
            let y = BOARD_Y + row as f32 * CELL_STEP;
            ctx.set_fill_style(
                if game.turn == Player::Black && game.hover == Some((row, col)) {
                    palette.board_hover
                } else {
                    palette.board
                },
            );
            ctx.fill_rect(x, y, CELL, CELL);

            let (cx, cy) = (x + CELL / 2.0, y + CELL / 2.0);
            let cell = game.board.cell_state(row, col);
            if cell != CellState::Empty {
                draw_disc(ctx, cx, cy, cell);
            } else if game.turn == Player::Black && legal_moves.contains(&Move { row, col }) {
                circle(
                    ctx,
                    cx,
                    cy,
                    if game.hover == Some((row, col)) {
                        CELL * 0.12
                    } else {
                        CELL * 0.075
                    },
                    palette.move_hint,
                );
            }
        }
    }
    ctx.set_fill_style(palette.text);
    ctx.set_font("sans-serif", 19.0);
    ctx.set_font_weight(FontWeight::Bold);
    ctx.fill_text(
        game.status(),
        WINDOW_WIDTH / 2.0,
        GRID_OFFSET + CELL * 9.5 + CELL_GAP * 9.0,
    );
    ctx.restore();
}

fn draw_disc(ctx: &mut CanvasRenderingContext2d<'_>, cx: f32, cy: f32, cell: CellState) {
    let radius = CELL * 0.46;

    // The game builds its small contact shadow from the same path primitives
    // as every other shape; Canvas itself only owns drawing fundamentals.
    circle(ctx, cx, cy + 2.0, radius, Color::rgba(0, 0, 0, 80));
    match cell {
        CellState::Black => {
            circle(ctx, cx, cy, radius, Color::rgb(8, 8, 10));
            circle(
                ctx,
                cx - 1.2,
                cy - 1.8,
                radius * 0.91,
                Color::rgb(31, 31, 34),
            );
            circle(
                ctx,
                cx - radius * 0.25,
                cy - radius * 0.3,
                radius * 0.24,
                Color::rgba(255, 255, 255, 24),
            );
        }
        CellState::White => {
            circle(ctx, cx, cy, radius, Color::rgb(174, 174, 178));
            circle(
                ctx,
                cx - 1.0,
                cy - 1.8,
                radius * 0.91,
                Color::rgb(246, 246, 248),
            );
            circle(
                ctx,
                cx - radius * 0.25,
                cy - radius * 0.3,
                radius * 0.25,
                Color::rgba(255, 255, 255, 180),
            );
        }
        CellState::Empty => {}
    }
}

fn circle(ctx: &mut CanvasRenderingContext2d<'_>, x: f32, y: f32, radius: f32, color: Color) {
    ctx.begin_path();
    ctx.arc(x, y, radius, 0.0, TAU, false);
    ctx.set_fill_style(color);
    ctx.fill();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn game_applies_a_human_move() {
        let mut game = Game::default();
        assert!(game.play(2, 3));
        assert_eq!(game.board.score(Player::Black), 4);
        assert_eq!(game.board.score(Player::White), 1);
    }

    #[test]
    fn rejects_occupied_and_illegal() {
        let mut game = Game::default();
        assert!(!game.play(3, 3));
        assert!(!game.play(0, 0));
    }

    #[test]
    fn ends_when_neither_can_move() {
        let mut game = Game {
            board: "x".repeat(64).parse().expect("valid full board"),
            turn: Player::White,
            hover: None,
            game_over: false,
            window_size: LogicalSize::new(WINDOW_WIDTH, WINDOW_HEIGHT),
            top_inset: 0.0,
            theme: Theme::Light,
        };
        game.advance_passes();
        assert!(game.game_over);
    }

    #[test]
    fn game_layout_centers_with_a_fixed_aspect_ratio() {
        let wide = GameLayout::new(LogicalSize::new(1_000.0, 800.0), 0.0);
        assert_eq!(wide.scale, 4.0 / 3.0);
        assert_eq!((wide.offset_x, wide.offset_y), (100.0, 0.0));

        let tall = GameLayout::new(LogicalSize::new(720.0, 1_000.0), 0.0);
        assert_eq!(tall.scale, 1.2);
        assert_eq!((tall.offset_x, tall.offset_y), (0.0, 140.0));
    }

    #[test]
    fn game_layout_reserves_the_hidden_titlebar() {
        let layout = GameLayout::new(LogicalSize::new(600.0, 628.0), 28.0);
        assert_eq!(layout.scale, 1.0);
        assert_eq!((layout.offset_x, layout.offset_y), (0.0, 28.0));

        let titlebar = MouseEvent {
            client_x: 300.0,
            client_y: 14.0,
            screen_x: 300.0,
            screen_y: 14.0,
            movement_x: 0.0,
            movement_y: 0.0,
            button: 0,
            buttons: 0,
            detail: 0,
            alt_key: false,
            ctrl_key: false,
            meta_key: false,
            shift_key: false,
        };
        assert!(layout.game_point(&titlebar).is_none());
    }

    #[test]
    fn light_and_dark_palettes_have_distinct_contrast() {
        let light = Palette::for_theme(Theme::Light);
        let dark = Palette::for_theme(Theme::Dark);
        assert_ne!(light.background, dark.background);
        assert!(u16::from(light.text.red) < u16::from(dark.text.red));
        assert_ne!(light.board, dark.board);
        assert_ne!(light.move_hint, dark.move_hint);
    }

    #[test]
    fn new_game_preserves_theme_and_window_size() {
        let game = RefCell::new(Game {
            window_size: LogicalSize::new(800.0, 700.0),
            top_inset: 28.0,
            theme: Theme::Dark,
            ..Game::default()
        });
        reset_game(&game);
        let game = game.borrow();
        assert_eq!(game.theme, Theme::Dark);
        assert_eq!(game.window_size.width, 800.0);
        assert_eq!(game.window_size.height, 700.0);
        assert_eq!(game.top_inset, 28.0);
    }
}
