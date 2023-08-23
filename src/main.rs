#![windows_subsystem = "windows"]

use std::time::{Duration, Instant};

use eframe::{App, CreationContext, NativeOptions};
use egui::{
    Align, Align2, Button, CentralPanel, Color32, FontId, Frame, Key, Layout, Pos2, Rect, RichText,
    Stroke, TextStyle, Vec2,
};
use rand::Rng;

const GAME_WIDTH: i16 = 20;
const GAME_HEIGHT: i16 = 14;
const MINE_PROBABILITY: f64 = 0.15;

struct MinesweeperApp {
    game: Game,
    cursor_x: i16,
    cursor_y: i16,
}

struct Game {
    first: bool,
    probability: f64,
    play_state: PlayState,
    width: i16,
    height: i16,
    fields: Vec<Field>,
    start_time: Instant,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PlayState {
    Playing,
    Won(Duration),
    Lost(Duration),
}

impl Game {
    fn new(width: i16, height: i16, probability: f64) -> Self {
        let len = (width * height) as usize;
        let mut game = Self {
            first: true,
            probability,
            play_state: PlayState::Playing,
            width,
            height,
            fields: vec![Field::free(0); len],
            start_time: Instant::now(),
        };

        game.gen_board();

        game
    }

    fn clear_board(&mut self) {
        for f in self.fields.iter_mut() {
            *f = Field::free(0);
        }
    }

    fn gen_board(&mut self) {
        let mut rng = rand::thread_rng();
        for y in 0..self.height {
            for x in 0..self.width {
                if rng.gen_bool(self.probability) {
                    self[(x, y)] = Field::mine();

                    self.increment_field((x - 1, y - 1));
                    self.increment_field((x - 1, y + 0));
                    self.increment_field((x - 1, y + 1));
                    self.increment_field((x + 0, y - 1));
                    self.increment_field((x + 0, y + 1));
                    self.increment_field((x + 1, y - 1));
                    self.increment_field((x + 1, y + 0));
                    self.increment_field((x + 1, y + 1));
                }
            }
        }
    }

    fn increment_field(&mut self, (x, y): (i16, i16)) {
        if x >= 0 && x < self.width && y >= 0 && y < self.height {
            if let FieldState::Free(neighbors) = &mut self[(x, y)].state {
                *neighbors += 1;
            }
        }
    }

    fn click(&mut self, (x, y): (i16, i16)) {
        if x < 0 || x >= self.width || y < 0 || y >= self.height {
            return;
        }

        let first = self.first;
        loop {
            let field = &mut self[(x, y)];
            if field.show == ShowState::Hint {
                return;
            }

            match field.state {
                FieldState::Free(neighbours) => {
                    if first && neighbours != 0 {
                        self.clear_board();
                        self.gen_board();
                        continue;
                    }

                    if let ShowState::Show = field.show {
                        let num_hinted_mines = self.count_hinted_mine((x - 1, y - 1))
                            + self.count_hinted_mine((x - 1, y + 0))
                            + self.count_hinted_mine((x - 1, y + 1))
                            + self.count_hinted_mine((x + 0, y - 1))
                            + self.count_hinted_mine((x + 0, y + 1))
                            + self.count_hinted_mine((x + 1, y - 1))
                            + self.count_hinted_mine((x + 1, y + 0))
                            + self.count_hinted_mine((x + 1, y + 1));

                        if num_hinted_mines == neighbours {
                            self.show_if_not_hinted((x - 1, y - 1));
                            self.show_if_not_hinted((x - 1, y + 0));
                            self.show_if_not_hinted((x - 1, y + 1));
                            self.show_if_not_hinted((x + 0, y - 1));
                            self.show_if_not_hinted((x + 0, y + 1));
                            self.show_if_not_hinted((x + 1, y - 1));
                            self.show_if_not_hinted((x + 1, y + 0));
                            self.show_if_not_hinted((x + 1, y + 1));
                        }
                    }

                    self.show_neighbors((x, y));
                    self.check_if_won();
                    break;
                }
                FieldState::Mine => {
                    if first {
                        self.clear_board();
                        self.gen_board();
                        continue;
                    }

                    self.lose();
                    break;
                }
            }
        }

        if self.first {
            self.start_time = Instant::now();
            self.first = false;
        }
    }

    fn hint(&mut self, (x, y): (i16, i16)) {
        if x < 0 || x >= self.width || y < 0 || y >= self.height {
            return;
        }

        let field = &mut self[(x, y)];
        if field.show == ShowState::Hint {
            field.show = ShowState::Hide;
        } else if field.show == ShowState::Hide {
            field.show = ShowState::Hint;
        }
    }

    fn lose(&mut self) {
        let duration = Instant::now() - self.start_time;
        self.play_state = PlayState::Lost(duration);
        for f in self.fields.iter_mut() {
            if let FieldState::Mine = f.state {
                f.show = ShowState::Show;
            }
        }
    }

    fn check_if_won(&mut self) {
        for f in self.fields.iter() {
            if let FieldState::Free(_) = f.state {
                if f.show != ShowState::Show {
                    return;
                }
            }
        }

        let duration = Instant::now() - self.start_time;
        self.play_state = PlayState::Won(duration);
        for f in self.fields.iter_mut() {
            f.show = ShowState::Show;
        }
    }

    fn show_if_not_hinted(&mut self, (x, y): (i16, i16)) {
        if x < 0 || x >= self.width || y < 0 || y >= self.height {
            return;
        }

        let field = &mut self[(x, y)];
        if field.show == ShowState::Show || field.show == ShowState::Hint {
            return;
        }

        if let FieldState::Mine = field.state {
            self.lose();
            return;
        }

        self.show_neighbors((x, y));
    }

    fn show_neighbors(&mut self, (x, y): (i16, i16)) {
        if x < 0 || x >= self.width || y < 0 || y >= self.height {
            return;
        }

        let field = &mut self[(x, y)];
        if field.show == ShowState::Show {
            return;
        }

        field.show = ShowState::Show;

        if field.state != FieldState::Free(0) {
            return;
        }

        self.show_neighbors((x - 1, y - 1));
        self.show_neighbors((x - 1, y + 0));
        self.show_neighbors((x - 1, y + 1));
        self.show_neighbors((x + 0, y - 1));
        self.show_neighbors((x + 0, y + 1));
        self.show_neighbors((x + 1, y - 1));
        self.show_neighbors((x + 1, y + 0));
        self.show_neighbors((x + 1, y + 1));
    }

    fn count_hinted_mine(&self, (x, y): (i16, i16)) -> u8 {
        if x < 0 || x >= self.width || y < 0 || y >= self.height {
            return 0;
        }

        if self[(x, y)].show == ShowState::Hint {
            return 1;
        }

        return 0;
    }

    fn open_mine_count(&self) -> i32 {
        let mut mines = 0;
        let mut hints = 0;
        for f in self.fields.iter() {
            if let FieldState::Mine = f.state {
                mines += 1;
            }
            if let ShowState::Hint = f.show {
                hints += 1;
            }
        }
        mines - hints
    }

    fn play_duration(&self) -> Duration {
        match self.play_state {
            PlayState::Playing => Instant::now() - self.start_time,
            PlayState::Won(duration) => duration,
            PlayState::Lost(duration) => duration,
        }
    }
}

impl std::ops::Index<(i16, i16)> for Game {
    type Output = Field;

    fn index(&self, (x, y): (i16, i16)) -> &Self::Output {
        &self.fields[self.width as usize * y as usize + x as usize]
    }
}

impl std::ops::IndexMut<(i16, i16)> for Game {
    fn index_mut(&mut self, (x, y): (i16, i16)) -> &mut Self::Output {
        &mut self.fields[self.width as usize * y as usize + x as usize]
    }
}

#[derive(Clone, Copy, Debug)]
struct Field {
    show: ShowState,
    state: FieldState,
}

impl Field {
    fn free(neighbors: u8) -> Self {
        Self {
            show: ShowState::Hide,
            state: FieldState::Free(neighbors),
        }
    }

    fn mine() -> Self {
        Self {
            show: ShowState::Hide,
            state: FieldState::Mine,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ShowState {
    Hide,
    Hint,
    Show,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum FieldState {
    Free(u8),
    Mine,
}

impl MinesweeperApp {
    fn new(_c: &CreationContext) -> Self {
        Self {
            cursor_x: 0,
            cursor_y: 0,
            game: Game::new(GAME_WIDTH, GAME_HEIGHT, MINE_PROBABILITY),
        }
    }

    fn cursor_left(&mut self) {
        self.cursor_x -= 1;
        if self.cursor_x < 0 {
            self.cursor_x = self.game.width - 1;
        }
    }

    fn cursor_right(&mut self) {
        self.cursor_x += 1;
        if self.cursor_x >= self.game.width {
            self.cursor_x = 0
        }
    }

    fn cursor_up(&mut self) {
        self.cursor_y -= 1;
        if self.cursor_y < 0 {
            self.cursor_y = self.game.height - 1;
        }
    }

    fn cursor_down(&mut self) {
        self.cursor_y += 1;
        if self.cursor_y >= self.game.height {
            self.cursor_y = 0
        }
    }
}

impl App for MinesweeperApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        CentralPanel::default()
            .frame(Frame::none())
            .show(ctx, |ui| {
                let menu_bar_height = 40.0;
                let available_size = ui.available_size() - Vec2::new(0.0, menu_bar_height);
                let cells = Vec2::new(self.game.width as f32, self.game.height as f32);
                let ratio = available_size / cells;
                let cell_size = Vec2::splat(ratio.min_elem());
                let board_size = cells * cell_size;
                let board_offset =
                    Pos2::new(0.0, menu_bar_height) + (available_size - board_size) * 0.5;
                let board_rect = Rect::from_min_size(board_offset, board_size);
                ui.allocate_ui(Vec2::new(ui.available_width(), menu_bar_height), |ui| {
                    ui.horizontal(|ui| {
                        ui.add_space(board_offset.x);
                        let open_mine_count = if self.game.first {
                            "?".to_string()
                        } else {
                            self.game.open_mine_count().to_string()
                        };
                        let text = RichText::new(open_mine_count).font(FontId::monospace(30.0));
                        ui.label(text);

                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            ui.add_space(board_offset.x);
                            let play_duration = if self.game.first {
                                Duration::ZERO
                            } else {
                                self.game.play_duration()
                            };
                            let text = RichText::new(format_duration(play_duration))
                                .font(FontId::monospace(30.0));
                            ui.label(text);

                            ui.add_space(20.0);
                            let text = RichText::new("\u{21bb}").font(FontId::monospace(30.0));
                            let button = Button::new(text).frame(false);
                            if ui.add(button).clicked() {
                                self.game = Game::new(GAME_WIDTH, GAME_HEIGHT, MINE_PROBABILITY);
                            }
                        });
                    });
                });

                // input
                ctx.input(|i| {
                    // arrow keys
                    if i.key_pressed(Key::ArrowUp) {
                        self.cursor_up();
                    } else if i.key_pressed(Key::ArrowRight) {
                        self.cursor_right();
                    } else if i.key_pressed(Key::ArrowDown) {
                        self.cursor_down();
                    } else if i.key_pressed(Key::ArrowLeft) {
                        self.cursor_left();
                    }

                    // wasd keys
                    if i.key_pressed(Key::W) {
                        self.cursor_up();
                    } else if i.key_pressed(Key::D) {
                        self.cursor_right();
                    } else if i.key_pressed(Key::S) {
                        self.cursor_down();
                    } else if i.key_pressed(Key::A) {
                        self.cursor_left();
                    }

                    // vim keys
                    if i.key_pressed(Key::K) {
                        self.cursor_up();
                    } else if i.key_pressed(Key::L) {
                        self.cursor_right();
                    } else if i.key_pressed(Key::J) {
                        self.cursor_down();
                    } else if i.key_pressed(Key::H) {
                        self.cursor_left();
                    }

                    if ui.input(|i| i.key_pressed(Key::R)) {
                        self.game = Game::new(GAME_WIDTH, GAME_HEIGHT, MINE_PROBABILITY);
                    }

                    if self.game.play_state == PlayState::Playing {
                        // sweep
                        if i.key_pressed(Key::Enter) || i.key_pressed(Key::Space) {
                            if i.modifiers.ctrl {
                                self.game.hint((self.cursor_x, self.cursor_y));
                            } else {
                                self.game.click((self.cursor_x, self.cursor_y));
                            }
                        }

                        let mut clicked = false;
                        let mut hint = false;
                        if i.pointer.primary_clicked() {
                            clicked = true;
                        } else if i.pointer.secondary_clicked() {
                            clicked = true;
                            hint = true;
                        }
                        if clicked {
                            if let Some(pos) = i.pointer.hover_pos() {
                                let cell_idx = (pos.to_vec2() - board_offset.to_vec2()) / cell_size;
                                let (x, y) = (cell_idx.x.floor() as i16, cell_idx.y.floor() as i16);

                                if hint {
                                    self.game.hint((x, y));
                                } else {
                                    self.game.click((x, y));
                                }
                            }
                        }
                    }
                });

                // draw
                let painter = ui.painter();
                let bg_color = Color32::BLACK;
                let cell_stroke = Stroke::new(1.0, bg_color);
                painter.rect(board_rect, 0.0, bg_color, Stroke::NONE);

                for y in 0..self.game.height {
                    for x in 0..self.game.width {
                        let field = self.game[(x, y)];
                        let cell_pos = board_offset + Vec2::new(x as f32, y as f32) * cell_size;
                        let cell_rect = Rect::from_min_size(cell_pos, cell_size);

                        match field.show {
                            ShowState::Hide => {
                                painter.rect(cell_rect, 0.0, Color32::from_gray(0x40), cell_stroke);
                            }
                            ShowState::Hint => {
                                painter.rect(
                                    cell_rect,
                                    0.0,
                                    Color32::from_rgb(0xf0, 0xc0, 0x30),
                                    cell_stroke,
                                );
                            }
                            ShowState::Show => {
                                let cell_center_pos = cell_pos + cell_size / 2.0;
                                let mut text_style =
                                    TextStyle::Monospace.resolve(ctx.style().as_ref());
                                text_style.size = cell_size.y * 0.8;

                                match field.state {
                                    FieldState::Free(c) => {
                                        painter.rect(
                                            cell_rect,
                                            0.0,
                                            Color32::from_gray(0x80),
                                            cell_stroke,
                                        );

                                        if c != 0 {
                                            const COLORS: [Color32; 8] = [
                                                Color32::BLUE,
                                                Color32::GREEN,
                                                Color32::RED,
                                                Color32::DARK_BLUE,
                                                Color32::DARK_RED,
                                                Color32::LIGHT_BLUE,
                                                Color32::BLACK,
                                                Color32::GRAY,
                                            ];
                                            let num_color = COLORS[c as usize - 1];
                                            painter.text(
                                                cell_center_pos,
                                                Align2::CENTER_CENTER,
                                                c,
                                                text_style,
                                                num_color,
                                            );
                                        }
                                    }
                                    FieldState::Mine => {
                                        let color = match self.game.play_state {
                                            PlayState::Won(_) => Color32::from_gray(0x80),
                                            PlayState::Lost(_) => {
                                                Color32::from_rgb(0xd0, 0x60, 0x30)
                                            }
                                            PlayState::Playing => {
                                                unreachable!("can't show a mine if still playing")
                                            }
                                        };
                                        painter.rect(cell_rect, 0.0, color, cell_stroke);
                                        painter.text(
                                            cell_center_pos,
                                            Align2::CENTER_CENTER,
                                            "*",
                                            text_style,
                                            Color32::BLACK,
                                        );
                                    }
                                }
                            }
                        }
                    }
                }

                // cursor
                let cursor_pos = board_offset
                    + Vec2::new(self.cursor_x as f32, self.cursor_y as f32) * cell_size;
                let cursor_rect = Rect::from_min_size(cursor_pos, cell_size);
                painter.rect(
                    cursor_rect,
                    0.0,
                    Color32::TRANSPARENT,
                    Stroke::new(2.0, Color32::from_rgb(0xd0, 0xd0, 0xf0)),
                );
            });
    }
}

fn format_duration(duration: Duration) -> String {
    let total_secs = duration.as_secs();
    let secs = total_secs % 60;
    let mins = total_secs / 60;
    format!("{mins:2}:{secs:02}")
}

fn main() {
    let options = NativeOptions {
        drag_and_drop_support: true,
        follow_system_theme: true,
        ..Default::default()
    };
    let res = eframe::run_native(
        "minesweeper",
        options,
        Box::new(|c| Box::new(MinesweeperApp::new(c))),
    );
    if let Err(e) = res {
        println!("error running app: {e}");
    }
}
