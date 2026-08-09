/*
 * Copyright (c) 2014 Gabriele Cirulli
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

#![doc = include_str!("../README.md")]
#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

mod game;

use std::cell::RefCell;
use std::collections::VecDeque;
use std::fs;
use std::path::PathBuf;
use std::rc::Rc;
use std::time::Instant;

#[cfg(target_os = "macos")]
use bwebview::{
    Accelerator, KeyCode, MenuBarBuilder, MenuBuilder, MenuItem, MenuItemRole, Modifiers,
};
use bwebview::{
    CanvasBuilder, CanvasEvent, CanvasRenderingContext2d, Color, Event, EventLoopBuilder,
    FontWeight, KeyboardEvent, LogicalSize, MouseEvent, TextAlign, TextBaseline, Theme,
    WindowBuilder, WindowEvent,
};
use directories::ProjectDirs;
use game::{Direction, Game, MoveAnimation};

const WINDOW_WIDTH: f32 = 600.0;
const WINDOW_HEIGHT: f32 = 850.0;
const MIN_WINDOW_WIDTH: f32 = 375.0;
const MIN_WINDOW_HEIGHT: f32 = 531.25;
const BOARD_X: f32 = 50.0;
const BOARD_Y: f32 = 216.0;
const BOARD_SIZE: f32 = 500.0;
const BOARD_PADDING: f32 = 15.0;
const GRID_CELL_SIZE: f32 = 106.25;
const GRID_CELL_STEP: f32 = 121.25;
const TILE_SIZE: f32 = 107.0;
const TILE_STEP: f32 = 121.0;
const SLIDE_TIME: f32 = 0.1;
const POP_TIME: f32 = 0.2;

#[derive(Clone, Copy)]
struct Palette {
    background: u32,
    text: Color,
    board: Color,
    grid: Color,
    score_panel: Color,
    score_label: Color,
    score_value: Color,
    button: Color,
    button_text: Color,
}

impl Palette {
    const fn for_theme(theme: Theme) -> Self {
        match theme {
            Theme::Light => Self {
                background: 0xfaf8ef,
                text: Color::rgb(119, 110, 101),
                board: Color::rgb(187, 173, 160),
                grid: Color::rgb(205, 193, 180),
                score_panel: Color::rgb(187, 173, 160),
                score_label: Color::rgb(238, 228, 218),
                score_value: Color::rgb(255, 255, 255),
                button: Color::rgb(143, 122, 102),
                button_text: Color::rgb(249, 246, 242),
            },
            Theme::Dark => Self {
                background: 0x1c1b1a,
                text: Color::rgb(242, 237, 230),
                board: Color::rgb(74, 67, 61),
                grid: Color::rgb(49, 44, 40),
                score_panel: Color::rgb(98, 87, 77),
                score_label: Color::rgb(215, 204, 193),
                score_value: Color::rgb(255, 252, 248),
                button: Color::rgb(161, 102, 63),
                button_text: Color::rgb(255, 249, 241),
            },
        }
    }
}

#[derive(Clone)]
struct Animation {
    movement: Option<MoveAnimation>,
    appearing: Vec<usize>,
    started: Instant,
}

struct App {
    game: Game,
    storage: Storage,
    animation: Option<Animation>,
    terminated_at: Option<Instant>,
    queued_moves: VecDeque<Direction>,
    mouse_down: Option<(f32, f32)>,
    window_size: LogicalSize,
    theme: Theme,
}

impl App {
    fn new(storage: Storage, theme: Theme) -> Self {
        let loaded = storage.load();
        let (game, animation) = if let Some(game) = loaded.game {
            (game, None)
        } else {
            let game = Game::new(loaded.best);
            let appearing = occupied_cells(&game);
            (
                game,
                Some(Animation {
                    movement: None,
                    appearing,
                    started: Instant::now(),
                }),
            )
        };
        let terminated_at = game.is_terminated().then(Instant::now);
        storage.save(&game);
        Self {
            game,
            storage,
            animation,
            terminated_at,
            queued_moves: VecDeque::new(),
            mouse_down: None,
            window_size: LogicalSize::new(WINDOW_WIDTH, WINDOW_HEIGHT),
            theme,
        }
    }

    fn new_game(&mut self) {
        self.game = Game::new(self.game.best);
        self.animation = Some(Animation {
            movement: None,
            appearing: occupied_cells(&self.game),
            started: Instant::now(),
        });
        self.terminated_at = None;
        self.queued_moves.clear();
        self.storage.save(&self.game);
    }

    fn queue_move(&mut self, direction: Direction) {
        self.queued_moves.push_back(direction);
        self.process_queued_moves();
    }

    fn process_queued_moves(&mut self) {
        if self
            .animation
            .as_ref()
            .is_some_and(|animation| animation.started.elapsed().as_secs_f32() < SLIDE_TIME)
        {
            return;
        }
        while let Some(direction) = self.queued_moves.pop_front() {
            if let Some(movement) = self.game.move_tiles(direction) {
                let new_tile = movement.new_tile;
                self.animation = Some(Animation {
                    movement: Some(movement),
                    appearing: vec![new_tile],
                    started: Instant::now(),
                });
                if self.game.is_terminated() {
                    self.terminated_at = Some(Instant::now());
                    self.queued_moves.clear();
                }
                self.storage.save(&self.game);
                break;
            }
        }
    }

    fn keep_playing(&mut self) {
        self.game.continue_game();
        self.terminated_at = None;
        self.storage.save(&self.game);
    }

    fn needs_redraw(&self, now: Instant) -> bool {
        self.animation.as_ref().is_some_and(|animation| {
            now.duration_since(animation.started).as_secs_f32()
                < if animation
                    .movement
                    .as_ref()
                    .is_some_and(|movement| movement.score_added > 0)
                {
                    0.7
                } else {
                    SLIDE_TIME + POP_TIME
                }
        }) || self
            .terminated_at
            .is_some_and(|started| now.duration_since(started).as_secs_f32() < 2.0)
            || !self.queued_moves.is_empty()
    }
}

#[derive(Clone, Copy)]
struct Layout {
    scale: f32,
    offset_x: f32,
    offset_y: f32,
}

impl Layout {
    fn new(size: LogicalSize) -> Self {
        let scale = (size.width / WINDOW_WIDTH)
            .min(size.height / WINDOW_HEIGHT)
            .max(0.01);
        Self {
            scale,
            offset_x: (size.width - WINDOW_WIDTH * scale) / 2.0,
            offset_y: (size.height - WINDOW_HEIGHT * scale) / 2.0,
        }
    }

    fn point(self, event: &MouseEvent) -> (f32, f32) {
        (
            (event.client_x - self.offset_x) / self.scale,
            (event.client_y - self.offset_y) / self.scale,
        )
    }
}

fn main() {
    let event_loop_builder = EventLoopBuilder::new().app_id("nl", "bplaat", "2048");
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
        .title("2048")
        .size(LogicalSize::new(WINDOW_WIDTH, WINDOW_HEIGHT))
        .min_size(LogicalSize::new(MIN_WINDOW_WIDTH, MIN_WINDOW_HEIGHT))
        .resizable(true)
        .background_color(initial_palette.background)
        .center()
        .remember_window_state();
    #[cfg(target_os = "macos")]
    {
        window_builder = window_builder.macos_titlebar_style(bwebview::MacosTitlebarStyle::Hidden);
    }
    let mut window = window_builder.build();
    let mut canvas = CanvasBuilder::new(&window).build();
    let app = Rc::new(RefCell::new(App::new(Storage::new(), initial_theme)));
    let events = app.clone();

    event_loop.run(move |event| {
        // Keep the native window handle alive for the full event-loop lifetime.
        let _window = &window;
        match event {
            Event::Canvas(CanvasEvent::Draw(context)) => {
                let mut app = events.borrow_mut();
                app.window_size = LogicalSize::new(context.width(), context.height());
                app.process_queued_moves();
                draw(context, &app);
                let needs_redraw = app.needs_redraw(Instant::now());
                if needs_redraw {
                    drop(app);
                    canvas.request_animation_frame();
                } else {
                    app.animation = None;
                }
            }
            Event::Window(WindowEvent::Resize(size)) => {
                events.borrow_mut().window_size = size;
                canvas.request_redraw();
            }
            Event::Window(WindowEvent::ThemeChange(theme)) => {
                events.borrow_mut().theme = theme;
                window.set_background_color(Palette::for_theme(theme).background);
                canvas.request_redraw();
            }
            Event::Window(WindowEvent::KeyDown(event)) => {
                if let Some(direction) = key_direction(&event) {
                    events.borrow_mut().queue_move(direction);
                    canvas.request_animation_frame();
                } else if no_modifiers(&event) && event.key.eq_ignore_ascii_case("r") {
                    events.borrow_mut().new_game();
                    canvas.request_animation_frame();
                }
            }
            Event::Window(WindowEvent::MouseDown(event)) if event.button == 0 => {
                let layout = Layout::new(events.borrow().window_size);
                events.borrow_mut().mouse_down = Some(layout.point(&event));
            }
            Event::Window(WindowEvent::MouseUp(event)) if event.button == 0 => {
                let layout = Layout::new(events.borrow().window_size);
                let point = layout.point(&event);
                let start = events.borrow_mut().mouse_down.take();
                if let Some(start) = start {
                    handle_pointer(&mut events.borrow_mut(), start, point);
                    canvas.request_animation_frame();
                }
            }
            #[cfg(target_os = "macos")]
            Event::MacosMenuItem(action) if action == "game.new" => {
                events.borrow_mut().new_game();
                canvas.request_animation_frame();
            }
            _ => {}
        }
    })
}

const fn no_modifiers(event: &KeyboardEvent) -> bool {
    !event.alt_key && !event.ctrl_key && !event.meta_key && !event.shift_key
}

fn key_direction(event: &KeyboardEvent) -> Option<Direction> {
    if !no_modifiers(event) {
        return None;
    }
    Some(match event.key.to_ascii_lowercase().as_str() {
        "arrowup" | "w" | "k" => Direction::Up,
        "arrowright" | "d" | "l" => Direction::Right,
        "arrowdown" | "s" | "j" => Direction::Down,
        "arrowleft" | "a" | "h" => Direction::Left,
        _ => return None,
    })
}

fn handle_pointer(app: &mut App, start: (f32, f32), end: (f32, f32)) {
    let dx = end.0 - start.0;
    let dy = end.1 - start.1;
    if inside(start, BOARD_X, BOARD_Y, BOARD_SIZE, BOARD_SIZE) && dx.abs().max(dy.abs()) > 10.0 {
        app.queue_move(if dx.abs() > dy.abs() {
            if dx > 0.0 {
                Direction::Right
            } else {
                Direction::Left
            }
        } else if dy > 0.0 {
            Direction::Down
        } else {
            Direction::Up
        });
        return;
    }
    if inside(end, 418.0, 134.0, 132.0, 40.0) {
        app.new_game();
    } else if app.game.is_terminated() && inside(end, 190.0, 520.0, 220.0, 45.0) {
        if app.game.won && !app.game.keep_playing {
            app.keep_playing();
        } else {
            app.new_game();
        }
    }
}

const fn inside(point: (f32, f32), x: f32, y: f32, width: f32, height: f32) -> bool {
    point.0 >= x && point.0 <= x + width && point.1 >= y && point.1 <= y + height
}

fn draw(ctx: &mut CanvasRenderingContext2d<'_>, app: &App) {
    let layout = Layout::new(app.window_size);
    let palette = Palette::for_theme(app.theme);
    ctx.set_fill_style(Color::from_rgb(palette.background));
    ctx.fill_rect(0.0, 0.0, app.window_size.width, app.window_size.height);
    ctx.save();
    ctx.translate(layout.offset_x, layout.offset_y);
    ctx.scale(layout.scale, layout.scale);
    ctx.set_fill_style(Color::from_rgb(palette.background));
    ctx.fill_rect(0.0, 0.0, WINDOW_WIDTH, WINDOW_HEIGHT);

    ctx.set_fill_style(palette.text);
    ctx.set_font("sans-serif", 80.0);
    ctx.set_font_weight(FontWeight::Bold);
    ctx.set_text_baseline(TextBaseline::Top);
    ctx.fill_text("2048", 50.0, 42.0);

    draw_score(ctx, &palette, "SCORE", app.game.score, 320.0);
    draw_score(ctx, &palette, "BEST", app.game.best, 440.0);

    ctx.set_font("sans-serif", 18.0);
    ctx.set_font_weight(FontWeight::Normal);
    ctx.set_fill_style(palette.text);
    ctx.set_text_baseline(TextBaseline::Middle);
    let x = text_run(ctx, "Join the numbers and get to the ", 50.0, 155.0, false);
    text_run(ctx, "2048 tile!", x, 155.0, true);
    draw_button(ctx, &palette, (418.0, 134.0, 132.0, 40.0), "New Game", 18.0);

    rounded_rect(
        ctx,
        BOARD_X,
        BOARD_Y,
        BOARD_SIZE,
        BOARD_SIZE,
        6.0,
        palette.board,
    );
    for row in 0..4 {
        for col in 0..4 {
            rounded_rect(
                ctx,
                grid_cell_x(col),
                grid_cell_y(row),
                GRID_CELL_SIZE,
                GRID_CELL_SIZE,
                3.0,
                palette.grid,
            );
        }
    }
    draw_tiles(ctx, app, &palette);

    draw_explanation(ctx, &palette);
    if app.game.is_terminated() {
        draw_message(ctx, app, &palette);
    }
    ctx.restore();
}

fn draw_score(
    ctx: &mut CanvasRenderingContext2d<'_>,
    palette: &Palette,
    label: &str,
    value: u32,
    x: f32,
) {
    rounded_rect(ctx, x, 48.0, 115.0, 55.0, 3.0, palette.score_panel);
    ctx.set_text_align(TextAlign::Center);
    ctx.set_font_weight(FontWeight::Bold);
    ctx.set_fill_style(palette.score_label);
    ctx.set_font("sans-serif", 13.0);
    ctx.set_text_baseline(TextBaseline::Middle);
    ctx.fill_text(label, x + 57.5, 64.0);
    ctx.set_fill_style(palette.score_value);
    ctx.set_font("sans-serif", 25.0);
    ctx.fill_text(value.to_string(), x + 57.5, 87.0);
    ctx.set_text_align(TextAlign::Left);
}

fn draw_tiles(ctx: &mut CanvasRenderingContext2d<'_>, app: &App, palette: &Palette) {
    let elapsed = app.animation.as_ref().map_or(f32::INFINITY, |animation| {
        animation.started.elapsed().as_secs_f32()
    });
    if elapsed < SLIDE_TIME
        && let Some(movement) = app
            .animation
            .as_ref()
            .and_then(|animation| animation.movement.as_ref())
    {
        let progress = ease_in_out((elapsed / SLIDE_TIME).clamp(0.0, 1.0));
        for motion in &movement.motions {
            let (from_x, from_y) = tile_position(motion.from);
            let (to_x, to_y) = tile_position(motion.to);
            draw_tile(
                ctx,
                motion.value,
                from_x + (to_x - from_x) * progress,
                from_y + (to_y - from_y) * progress,
                1.0,
                app.theme,
            );
        }
    } else {
        for (cell, &value) in app.game.cells.iter().enumerate() {
            if value == 0 {
                continue;
            }
            let mut scale = 1.0;
            if let Some(animation) = &app.animation {
                let pop_progress = ((elapsed - SLIDE_TIME) / POP_TIME).clamp(0.0, 1.0);
                if animation.appearing.contains(&cell) {
                    scale = ease_out(pop_progress);
                } else if animation
                    .movement
                    .as_ref()
                    .is_some_and(|movement| movement.merged.contains(&cell))
                {
                    scale = pop_scale(pop_progress);
                }
            }
            let (x, y) = tile_position(cell);
            draw_tile(ctx, value, x, y, scale, app.theme);
        }
    }

    if let Some(animation) = &app.animation
        && let Some(movement) = &animation.movement
        && movement.score_added > 0
        && elapsed < 0.6
    {
        let progress = ease_in((elapsed / 0.6).clamp(0.0, 1.0));
        ctx.set_global_alpha(1.0 - progress);
        ctx.set_fill_style(palette.text);
        ctx.set_font("sans-serif", 25.0);
        ctx.set_font_weight(FontWeight::Bold);
        ctx.set_text_align(TextAlign::Center);
        ctx.set_text_baseline(TextBaseline::Middle);
        ctx.fill_text(
            format!("+{}", movement.score_added),
            377.5,
            112.0 - 75.0 * progress,
        );
        ctx.set_global_alpha(1.0);
        ctx.set_text_align(TextAlign::Left);
    }
}

fn draw_tile(
    ctx: &mut CanvasRenderingContext2d<'_>,
    value: u32,
    x: f32,
    y: f32,
    scale: f32,
    theme: Theme,
) {
    if scale <= 0.0 {
        return;
    }
    let (background, foreground, font_size) = tile_style(value, theme);
    let size = TILE_SIZE * scale;
    let offset = (TILE_SIZE - size) / 2.0;
    rounded_rect(ctx, x + offset, y + offset, size, size, 3.0, background);
    ctx.set_fill_style(foreground);
    ctx.set_font("sans-serif", font_size * scale);
    ctx.set_font_weight(FontWeight::Bold);
    ctx.set_text_align(TextAlign::Center);
    ctx.set_text_baseline(TextBaseline::Middle);
    ctx.fill_text(
        value.to_string(),
        x + TILE_SIZE / 2.0,
        y + TILE_SIZE / 2.0 + 1.0,
    );
    ctx.set_text_align(TextAlign::Left);
}

const fn tile_style(value: u32, theme: Theme) -> (Color, Color, f32) {
    let palette = Palette::for_theme(theme);
    let light = palette.button_text;
    if matches!(theme, Theme::Dark) {
        return match value {
            2 => (Color::rgb(216, 206, 194), Color::rgb(77, 69, 62), 55.0),
            4 => (Color::rgb(190, 164, 134), Color::rgb(63, 56, 50), 55.0),
            8 => (Color::rgb(226, 160, 95), light, 55.0),
            16 => (Color::rgb(230, 132, 85), light, 55.0),
            32 => (Color::rgb(226, 107, 85), light, 55.0),
            64 => (Color::rgb(222, 78, 56), light, 55.0),
            128 => (Color::rgb(214, 176, 79), light, 45.0),
            256 => (Color::rgb(218, 170, 62), light, 45.0),
            512 => (Color::rgb(222, 165, 48), light, 45.0),
            1024 => (Color::rgb(226, 159, 35), light, 35.0),
            2048 => (Color::rgb(232, 171, 35), light, 35.0),
            _ => (Color::rgb(46, 43, 40), light, 30.0),
        };
    }
    match value {
        2 => (Color::rgb(238, 228, 218), palette.text, 55.0),
        4 => (Color::rgb(237, 224, 200), palette.text, 55.0),
        8 => (Color::rgb(242, 177, 121), light, 55.0),
        16 => (Color::rgb(245, 149, 99), light, 55.0),
        32 => (Color::rgb(246, 124, 95), light, 55.0),
        64 => (Color::rgb(246, 94, 59), light, 55.0),
        128 => (Color::rgb(237, 207, 114), light, 45.0),
        256 => (Color::rgb(237, 204, 97), light, 45.0),
        512 => (Color::rgb(237, 200, 80), light, 45.0),
        1024 => (Color::rgb(237, 197, 63), light, 35.0),
        2048 => (Color::rgb(237, 194, 46), light, 35.0),
        _ => (Color::rgb(60, 58, 50), light, 30.0),
    }
}

fn draw_explanation(ctx: &mut CanvasRenderingContext2d<'_>, palette: &Palette) {
    ctx.set_fill_style(palette.text);
    ctx.set_font("sans-serif", 18.0);
    ctx.set_text_baseline(TextBaseline::Middle);
    let mut x = 50.0;
    x = text_run(ctx, "HOW TO PLAY:", x, 775.0, true);
    x = text_run(ctx, " Use your ", x, 775.0, false);
    x = text_run(ctx, "arrow keys", x, 775.0, true);
    text_run(ctx, " to move the tiles. When", x, 775.0, false);
    let mut x = 50.0;
    x = text_run(
        ctx,
        "two tiles with the same number touch, they ",
        x,
        805.0,
        false,
    );
    text_run(ctx, "merge into one!", x, 805.0, true);
}

fn text_run(ctx: &mut CanvasRenderingContext2d<'_>, text: &str, x: f32, y: f32, bold: bool) -> f32 {
    ctx.set_font_weight(if bold {
        FontWeight::Bold
    } else {
        FontWeight::Normal
    });
    ctx.fill_text(text, x, y);
    x + ctx.measure_text(text)
}

fn draw_message(ctx: &mut CanvasRenderingContext2d<'_>, app: &App, palette: &Palette) {
    let alpha = app.terminated_at.map_or(1.0, |started| {
        ease_in_out(((started.elapsed().as_secs_f32() - 1.2) / 0.8).clamp(0.0, 1.0))
    });
    if alpha <= 0.0 {
        return;
    }
    let won = app.game.won && !app.game.keep_playing;
    ctx.set_fill_style(if won {
        let color = tile_style(2048, app.theme).0;
        Color::rgba(color.red, color.green, color.blue, (150.0 * alpha) as u8)
    } else if matches!(app.theme, Theme::Dark) {
        Color::rgba(24, 25, 22, (210.0 * alpha) as u8)
    } else {
        Color::rgba(238, 228, 218, (128.0 * alpha) as u8)
    });
    ctx.fill_rect(BOARD_X, BOARD_Y, BOARD_SIZE, BOARD_SIZE);
    ctx.set_global_alpha(alpha);
    ctx.set_fill_style(if won {
        palette.button_text
    } else {
        palette.text
    });
    ctx.set_font("sans-serif", 60.0);
    ctx.set_font_weight(FontWeight::Bold);
    ctx.set_text_align(TextAlign::Center);
    ctx.set_text_baseline(TextBaseline::Middle);
    ctx.fill_text(if won { "You win!" } else { "Game over!" }, 300.0, 460.0);
    draw_button(
        ctx,
        palette,
        (190.0, 520.0, 220.0, 45.0),
        if won { "Keep going" } else { "Try again" },
        18.0,
    );
    ctx.set_global_alpha(1.0);
    ctx.set_text_align(TextAlign::Left);
}

fn draw_button(
    ctx: &mut CanvasRenderingContext2d<'_>,
    palette: &Palette,
    bounds: (f32, f32, f32, f32),
    label: &str,
    font_size: f32,
) {
    let (x, y, width, height) = bounds;
    rounded_rect(ctx, x, y, width, height, 3.0, palette.button);
    ctx.set_fill_style(palette.button_text);
    ctx.set_font("sans-serif", font_size);
    ctx.set_font_weight(FontWeight::Bold);
    ctx.set_text_align(TextAlign::Center);
    ctx.set_text_baseline(TextBaseline::Middle);
    ctx.fill_text(label, x + width / 2.0, y + height / 2.0 + 1.0);
    ctx.set_text_align(TextAlign::Left);
}

fn rounded_rect(
    ctx: &mut CanvasRenderingContext2d<'_>,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    radius: f32,
    fill: Color,
) {
    ctx.begin_path();
    ctx.round_rect(x, y, width, height, radius);
    ctx.set_fill_style(fill);
    ctx.fill();
}

const fn tile_x(col: usize) -> f32 {
    BOARD_X + BOARD_PADDING + col as f32 * TILE_STEP
}

const fn tile_y(row: usize) -> f32 {
    BOARD_Y + BOARD_PADDING + row as f32 * TILE_STEP
}

const fn grid_cell_x(col: usize) -> f32 {
    BOARD_X + BOARD_PADDING + col as f32 * GRID_CELL_STEP
}

const fn grid_cell_y(row: usize) -> f32 {
    BOARD_Y + BOARD_PADDING + row as f32 * GRID_CELL_STEP
}

const fn tile_position(cell: usize) -> (f32, f32) {
    (tile_x(cell % 4), tile_y(cell / 4))
}

fn occupied_cells(game: &Game) -> Vec<usize> {
    game.cells
        .iter()
        .enumerate()
        .filter_map(|(cell, &value)| (value != 0).then_some(cell))
        .collect()
}

fn ease_in_out(value: f32) -> f32 {
    value * value * (3.0 - 2.0 * value)
}

fn ease_in(value: f32) -> f32 {
    value * value
}

fn ease_out(value: f32) -> f32 {
    1.0 - (1.0 - value) * (1.0 - value)
}

fn pop_scale(value: f32) -> f32 {
    if value < 0.5 {
        ease_out(value * 2.0) * 1.2
    } else {
        1.2 - ease_in_out((value - 0.5) * 2.0) * 0.2
    }
}

struct LoadedState {
    best: u32,
    game: Option<Game>,
}

struct Storage {
    path: Option<PathBuf>,
}

impl Storage {
    fn new() -> Self {
        Self {
            path: ProjectDirs::from("nl", "bplaat", "2048")
                .map(|dirs| dirs.config_dir().join("game-state")),
        }
    }

    fn load(&self) -> LoadedState {
        let Some(path) = &self.path else {
            return LoadedState {
                best: 0,
                game: None,
            };
        };
        let Ok(text) = fs::read_to_string(path) else {
            return LoadedState {
                best: 0,
                game: None,
            };
        };
        let value = |name: &str| {
            text.lines()
                .find_map(|line| line.strip_prefix(name))
                .unwrap_or_default()
        };
        let best = value("best=").parse().unwrap_or(0);
        let score = value("score=").parse().ok();
        let cells: Vec<u32> = value("cells=")
            .split(',')
            .filter_map(|cell| cell.parse().ok())
            .collect();
        let game = score.and_then(|score| {
            let cells: [u32; 16] = cells.try_into().ok()?;
            Some(Game::from_state(
                cells,
                score,
                best,
                value("won=") == "1",
                value("keep-playing=") == "1",
            ))
        });
        LoadedState { best, game }
    }

    fn save(&self, game: &Game) {
        let Some(path) = &self.path else { return };
        let Some(parent) = path.parent() else { return };
        if fs::create_dir_all(parent).is_err() {
            return;
        }
        let mut state = format!("best={}\n", game.best);
        if !game.is_over() {
            state.push_str(&format!(
                "score={}\nwon={}\nkeep-playing={}\ncells={}\n",
                game.score,
                u8::from(game.won),
                u8::from(game.keep_playing),
                game.cells
                    .iter()
                    .map(u32::to_string)
                    .collect::<Vec<_>>()
                    .join(",")
            ));
        }
        let _ = fs::write(path, state);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn rapid_moves_are_queued_and_processed() {
        let mut app = App {
            game: Game::from_state(
                [2, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
                0,
                0,
                false,
                false,
            ),
            storage: Storage { path: None },
            animation: None,
            terminated_at: None,
            queued_moves: VecDeque::new(),
            mouse_down: None,
            window_size: LogicalSize::new(WINDOW_WIDTH, WINDOW_HEIGHT),
            theme: Theme::Light,
        };

        app.queue_move(Direction::Left);
        app.queue_move(Direction::Down);
        assert_eq!(app.game.score, 4);
        assert_eq!(app.queued_moves.len(), 1);

        if let Some(animation) = &mut app.animation {
            animation.started = Instant::now() - Duration::from_millis(101);
        }
        app.process_queued_moves();
        assert!(app.queued_moves.is_empty());
        assert!(app.game.cells[12..].iter().any(|&value| value != 0));
    }

    #[test]
    fn wide_windows_center_the_original_aspect_ratio() {
        let layout = Layout::new(LogicalSize::new(1200.0, WINDOW_HEIGHT));
        assert_eq!(layout.scale, 1.0);
        assert_eq!(layout.offset_x, 300.0);
        assert_eq!(layout.offset_y, 0.0);
    }

    #[test]
    fn light_and_dark_palettes_are_distinct() {
        let light = Palette::for_theme(Theme::Light);
        let dark = Palette::for_theme(Theme::Dark);
        assert_ne!(light.background, dark.background);
        assert_ne!(light.text, dark.text);
        assert_ne!(tile_style(2, Theme::Light), tile_style(2, Theme::Dark));
        assert_ne!(
            tile_style(2048, Theme::Light),
            tile_style(2048, Theme::Dark)
        );
    }
}
