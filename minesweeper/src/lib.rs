use instant::SystemTime;
use rand::Rng;
use serde_derive::{Deserialize, Serialize};
use std::fmt::Display;
use std::time::Duration;

use egui::{
    Align, Align2, Button, Color32, ComboBox, FontId, Key, Layout, Pos2, Rect, RichText, Rounding,
    Sense, Stroke, TextStyle, Ui, Vec2, Visuals,
};

pub mod combination_iter;
mod gen;
pub mod stackvec;

#[derive(Serialize, Deserialize)]
pub struct Minesweeper {
    game: Game,
    long_press: bool,
    cursor_visible: bool,
    cursor_x: i16,
    cursor_y: i16,
    difficulty: Difficulty,
    unambigous: bool,
    highscores: [Vec<Duration>; 6],
}

impl Default for Minesweeper {
    fn default() -> Self {
        Self::new()
    }
}

impl Minesweeper {
    pub fn new() -> Self {
        let unambigous = false;
        Self {
            game: Game::easy(unambigous),
            long_press: false,
            cursor_visible: false,
            cursor_x: 0,
            cursor_y: 0,
            difficulty: Difficulty::Easy,
            unambigous,
            highscores: [
                Vec::new(),
                Vec::new(),
                Vec::new(),
                Vec::new(),
                Vec::new(),
                Vec::new(),
            ],
        }
    }

    fn new_game(&mut self) {
        self.game = match self.difficulty {
            Difficulty::Easy => Game::easy(self.unambigous),
            Difficulty::Medium => Game::medium(self.unambigous),
            Difficulty::Hard => Game::hard(self.unambigous),
        };
    }

    fn cursor_x_neg(&mut self) {
        self.cursor_visible = true;
        self.cursor_x -= 1;
        if self.cursor_x < 0 {
            self.cursor_x = self.game.width - 1;
        }
    }

    fn cursor_x_pos(&mut self) {
        self.cursor_visible = true;
        self.cursor_x += 1;
        if self.cursor_x >= self.game.width {
            self.cursor_x = 0
        }
    }

    fn cursor_y_neg(&mut self) {
        self.cursor_visible = true;
        self.cursor_y -= 1;
        if self.cursor_y < 0 {
            self.cursor_y = self.game.height - 1;
        }
    }

    fn cursor_y_pos(&mut self) {
        self.cursor_visible = true;
        self.cursor_y += 1;
        if self.cursor_y >= self.game.height {
            self.cursor_y = 0
        }
    }

    fn cursor_left(&mut self, flipped: bool) {
        if flipped {
            self.cursor_y_pos();
        } else {
            self.cursor_x_neg();
        }
    }

    fn cursor_right(&mut self, flipped: bool) {
        if flipped {
            self.cursor_y_neg();
        } else {
            self.cursor_x_pos();
        }
    }

    fn cursor_up(&mut self, flipped: bool) {
        if flipped {
            self.cursor_x_neg();
        } else {
            self.cursor_y_neg();
        }
    }

    fn cursor_down(&mut self, flipped: bool) {
        if flipped {
            self.cursor_x_pos();
        } else {
            self.cursor_y_pos();
        }
    }

    fn click(&mut self, frame: &mut eframe::Frame, x: i16, y: i16) {
        if let Some(duration) = self.game.click(x, y) {
            let scores = &mut self.highscores
                [self.game.difficulty as usize + (3 * self.game.unambigous as usize)];
            let idx = scores.iter().position(|d| duration < *d);
            match idx {
                Some(i) => scores.insert(i, duration),
                None => scores.push(duration),
            }
        }

        if let Some(storage) = frame.storage_mut() {
            eframe::set_value(storage, eframe::APP_KEY, self);
        }
    }

    fn hint(&mut self, frame: &mut eframe::Frame, x: i16, y: i16) {
        self.game.hint_(x, y);
        if let Some(storage) = frame.storage_mut() {
            eframe::set_value(storage, eframe::APP_KEY, self);
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
enum Difficulty {
    Easy = 0,
    Medium = 1,
    Hard = 2,
}

impl Display for Difficulty {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Difficulty::Easy => write!(f, "Easy"),
            Difficulty::Medium => write!(f, "Medium"),
            Difficulty::Hard => write!(f, "Hard"),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct Game {
    difficulty: Difficulty,
    unambigous: bool,
    num_mines: u16,
    play_state: PlayState,
    width: i16,
    height: i16,
    fields: Vec<Field>,
}

impl Game {
    fn easy(unambigous: bool) -> Self {
        Self::new(20, 14, 0.12..0.13, Difficulty::Easy, unambigous)
    }

    fn medium(unambigous: bool) -> Self {
        Self::new(30, 18, 0.16..0.17, Difficulty::Medium, unambigous)
    }

    fn hard(unambigous: bool) -> Self {
        Self::new(40, 24, 0.21..0.22, Difficulty::Hard, unambigous)
    }

    fn new(
        width: i16,
        height: i16,
        probability_range: std::ops::Range<f64>,
        difficulty: Difficulty,
        unambigous: bool,
    ) -> Self {
        let len = (width * height) as usize;

        let min = (probability_range.start * len as f64) as u16;
        let max = (probability_range.end * len as f64) as u16;
        let num_mines = rand::thread_rng().gen_range(min..max);

        Self {
            difficulty,
            unambigous,
            num_mines,
            play_state: PlayState::Init,
            width,
            height,
            fields: vec![Field::free(0); len],
        }
    }

    fn clear_board(&mut self) {
        for f in self.fields.iter_mut() {
            f.state = FieldState::Free(0);
        }
    }

    /// Returns the duration if the game was won.
    fn click(&mut self, x: i16, y: i16) -> Option<Duration> {
        if !self.is_in_bounds(x, y) {
            return None;
        }

        let first = self.play_state == PlayState::Init;
        if first {
            self.gen_board();

            let mut field = &self[(x, y)];
            loop {
                if field.state == FieldState::Free(0) {
                    if !self.unambigous || self.is_unambigous(x, y) {
                        break;
                    }
                }

                self.clear_board();
                self.gen_board();
                field = &self[(x, y)];
            }

            self.play_state = PlayState::Playing(SystemTime::now());
        }

        let field = &mut self[(x, y)];
        if field.visibility == Visibility::Hint {
            return None;
        }
        match field.state {
            FieldState::Free(neighbors) => {
                if let Visibility::Show = field.visibility {
                    let hinted_adjacents = self.hinted_adjacents(x, y);
                    if hinted_adjacents.num() == neighbors {
                        self.show_if_not_hinted(x - 1, y - 1);
                        self.show_if_not_hinted(x - 1, y + 0);
                        self.show_if_not_hinted(x - 1, y + 1);
                        self.show_if_not_hinted(x + 0, y - 1);
                        self.show_if_not_hinted(x + 0, y + 1);
                        self.show_if_not_hinted(x + 1, y - 1);
                        self.show_if_not_hinted(x + 1, y + 0);
                        self.show_if_not_hinted(x + 1, y + 1);
                    }
                }

                self.show_neighbors(x, y);
                self.check_if_won()
            }
            FieldState::Mine => {
                self.lose(x, y);
                None
            }
        }
    }

    fn hint_(&mut self, x: i16, y: i16) {
        if !self.is_in_bounds(x, y) {
            return;
        }

        let field = &mut self[(x, y)];
        if field.visibility == Visibility::Hint {
            field.visibility = Visibility::Hide;
        } else if field.visibility == Visibility::Hide {
            field.visibility = Visibility::Hint;
        }
    }

    fn lose(&mut self, x: i16, y: i16) {
        let PlayState::Playing(start) = self.play_state else {
            return;
        };
        let duration = SystemTime::now().duration_since(start).unwrap();
        self[(x, y)].visibility = Visibility::Show;
        self.play_state = PlayState::Lost(duration);
    }

    fn check_if_won(&mut self) -> Option<Duration> {
        if !self.is_solved() {
            return None;
        }

        let PlayState::Playing(start) = self.play_state else {
            return None;
        };
        let duration = SystemTime::now().duration_since(start).unwrap();
        self.play_state = PlayState::Won(duration);
        for f in self.fields.iter_mut() {
            f.visibility = Visibility::Show;
        }
        Some(duration)
    }

    fn show_if_not_hinted(&mut self, x: i16, y: i16) {
        if !self.is_in_bounds(x, y) {
            return;
        }

        let field = &mut self[(x, y)];
        if field.visibility == Visibility::Show || field.visibility == Visibility::Hint {
            return;
        }

        if let FieldState::Mine = field.state {
            self.lose(x, y);
            return;
        }

        self.show_neighbors(x, y);
    }

    fn show_neighbors(&mut self, x: i16, y: i16) {
        if !self.is_in_bounds(x, y) {
            return;
        }

        let field = &mut self[(x, y)];
        if field.visibility == Visibility::Show {
            return;
        }

        field.visibility = Visibility::Show;

        if field.state != FieldState::Free(0) {
            return;
        }

        self.show_neighbors(x - 1, y - 1);
        self.show_neighbors(x - 1, y + 0);
        self.show_neighbors(x - 1, y + 1);
        self.show_neighbors(x + 0, y - 1);
        self.show_neighbors(x + 0, y + 1);
        self.show_neighbors(x + 1, y - 1);
        self.show_neighbors(x + 1, y + 0);
        self.show_neighbors(x + 1, y + 1);
    }

    fn open_mine_count(&self) -> i16 {
        let mut hints = 0;
        for f in self.fields.iter() {
            if let Visibility::Hint = f.visibility {
                hints += 1;
            }
        }
        self.num_mines as i16 - hints
    }

    fn play_duration(&self) -> Duration {
        match self.play_state {
            PlayState::Init => Duration::ZERO,
            PlayState::Playing(start) => SystemTime::now().duration_since(start).unwrap(),
            PlayState::Won(duration) => duration,
            PlayState::Lost(duration) => duration,
        }
    }

    fn is_in_bounds(&self, x: i16, y: i16) -> bool {
        x >= 0 && x < self.width && y >= 0 && y < self.height
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

#[derive(Clone, Copy, Debug, PartialEq)]
enum PlayState {
    Init,
    Playing(SystemTime),
    Won(Duration),
    Lost(Duration),
}

#[derive(Serialize, Deserialize)]
#[serde(rename = "PlayState")]
enum PlayStateSerde {
    Init,
    Playing(Duration),
    Won(Duration),
    Lost(Duration),
}

impl serde::Serialize for PlayState {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let p = match self {
            PlayState::Init => PlayStateSerde::Init,
            PlayState::Playing(start) => {
                let duration = SystemTime::now().duration_since(*start).unwrap();
                PlayStateSerde::Playing(duration)
            }
            PlayState::Won(duration) => PlayStateSerde::Won(*duration),
            PlayState::Lost(duration) => PlayStateSerde::Lost(*duration),
        };

        p.serialize(serializer)
    }
}

impl<'de> serde::Deserialize<'de> for PlayState {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let p = PlayStateSerde::deserialize(deserializer)?;
        let p = match p {
            PlayStateSerde::Init => PlayState::Init,
            PlayStateSerde::Playing(duration) => {
                let start = SystemTime::now() - duration;
                PlayState::Playing(start)
            }
            PlayStateSerde::Won(duration) => PlayState::Won(duration),
            PlayStateSerde::Lost(duration) => PlayState::Lost(duration),
        };
        Ok(p)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
struct Field {
    visibility: Visibility,
    state: FieldState,
}

impl Field {
    fn free(neighbors: u8) -> Self {
        Self {
            visibility: Visibility::Hide,
            state: FieldState::Free(neighbors),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
enum Visibility {
    Hide,
    Hint,
    Show,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
enum FieldState {
    Free(u8),
    Mine,
}

fn format_duration(duration: Duration) -> String {
    let total_secs = duration.as_secs();
    let secs = total_secs % 60;
    let mins = total_secs / 60;
    format!("{mins:2}:{secs:02}")
}

fn board_idx_from_screen_pos(
    height: i16,
    board_offset: Pos2,
    cell_size: Vec2,
    pos: Pos2,
    flipped: bool,
) -> (i16, i16) {
    let cell_idx = (pos.to_vec2() - board_offset.to_vec2()) / cell_size;
    let (x, y) = (cell_idx.x.floor() as i16, cell_idx.y.floor() as i16);
    if flipped {
        (y, height - x - 1)
    } else {
        (x, y)
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn vibrate(_ms: u32) {}

#[cfg(target_arch = "wasm32")]
fn vibrate(ms: u32) {
    let Some(window) = web_sys::window() else { return };
    let navigator = window.navigator();
    let Ok(user_agent) = navigator.user_agent() else { return };
    let parser = woothee::parser::Parser::new();
    let Some(res) = parser.parse(&user_agent) else { return };
    if res.vendor != "Apple" {
        navigator.vibrate_with_duration(ms);
        log::info!("{res:?}");
    }
}

pub fn update(frame: &mut eframe::Frame, ui: &mut Ui, ms: &mut Minesweeper) {
    ui.ctx().request_repaint();

    let menu_bar_height = 40.0;
    let available_size = ui.available_size() - Vec2::new(0.0, menu_bar_height);
    let flipped = available_size.x < available_size.y;
    let cells;
    if flipped {
        cells = Vec2::new(ms.game.height as f32, ms.game.width as f32);
    } else {
        cells = Vec2::new(ms.game.width as f32, ms.game.height as f32);
    }
    let ratio = available_size / cells;
    let cell_size = Vec2::splat(ratio.min_elem());
    let board_size = cells * cell_size;
    let board_offset = Pos2::new(0.0, menu_bar_height) + (available_size - board_size) * 0.5;

    let board_rect = Rect::from_min_size(board_offset, board_size);
    ui.allocate_ui(Vec2::new(ui.available_width(), menu_bar_height), |ui| {
        ui.horizontal(|ui| {
            ui.add_space(board_offset.x);
            let open_mine_count = ms.game.open_mine_count().to_string();
            let text = RichText::new(open_mine_count).font(FontId::monospace(30.0));
            ui.label(text);

            ui.add_space(20.0);
            let visuals = ui.style().visuals.clone();
            let new_visuals = if visuals.dark_mode {
                let text = RichText::new("☀").font(FontId::proportional(20.0));
                ui.add(Button::new(text).frame(false))
                    .on_hover_text("Switch to light mode")
                    .clicked()
                    .then_some(Visuals::light())
            } else {
                let text = RichText::new("🌙").font(FontId::proportional(20.0));
                ui.add(Button::new(text).frame(false))
                    .on_hover_text("Switch to dark mode")
                    .clicked()
                    .then_some(Visuals::dark())
            };
            if let Some(visuals) = new_visuals {
                ui.ctx().set_visuals(visuals);
            }

            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                ui.add_space(board_offset.x);
                let play_duration = format_duration(ms.game.play_duration());
                let text = RichText::new(play_duration).font(FontId::monospace(30.0));
                ui.label(text);

                ui.add_space(20.0);
                let text = RichText::new("\u{21bb}").font(FontId::monospace(30.0));
                let button = Button::new(text).frame(false);
                if ui.add(button).clicked() {
                    ms.new_game();
                }

                ui.add_space(20.0);
                let text =
                    RichText::new(ms.difficulty.to_string()).font(FontId::proportional(20.0));
                let prev_difficulty = ms.difficulty;
                ComboBox::new("difficulty", "")
                    .selected_text(text)
                    .show_ui(ui, |ui| {
                        let text = RichText::new(Difficulty::Easy.to_string())
                            .font(FontId::proportional(20.0));
                        ui.selectable_value(&mut ms.difficulty, Difficulty::Easy, text);

                        let text = RichText::new(Difficulty::Medium.to_string())
                            .font(FontId::proportional(20.0));
                        ui.selectable_value(&mut ms.difficulty, Difficulty::Medium, text);

                        let text = RichText::new(Difficulty::Hard.to_string())
                            .font(FontId::proportional(20.0));
                        ui.selectable_value(&mut ms.difficulty, Difficulty::Hard, text);
                    });
                if ms.difficulty != prev_difficulty && ms.game.play_state == PlayState::Init {
                    ms.new_game();
                }

                ui.add_space(20.0);
                let text = RichText::new("unambigous").font(FontId::proportional(20.0));
                ui.checkbox(&mut ms.unambigous, text);
            });
        });
    });

    // input
    ui.input(|i| {
        // arrow keys
        if i.key_pressed(Key::ArrowUp) {
            ms.cursor_up(flipped);
        } else if i.key_pressed(Key::ArrowRight) {
            ms.cursor_right(flipped);
        } else if i.key_pressed(Key::ArrowDown) {
            ms.cursor_down(flipped);
        } else if i.key_pressed(Key::ArrowLeft) {
            ms.cursor_left(flipped);
        }

        // wasd keys
        if i.key_pressed(Key::W) {
            ms.cursor_up(flipped);
        } else if i.key_pressed(Key::D) {
            ms.cursor_right(flipped);
        } else if i.key_pressed(Key::S) {
            ms.cursor_down(flipped);
        } else if i.key_pressed(Key::A) {
            ms.cursor_left(flipped);
        }

        // vim keys
        if i.key_pressed(Key::K) {
            ms.cursor_up(flipped);
        } else if i.key_pressed(Key::L) {
            ms.cursor_right(flipped);
        } else if i.key_pressed(Key::J) {
            ms.cursor_down(flipped);
        } else if i.key_pressed(Key::H) {
            ms.cursor_left(flipped);
        }

        if i.key_pressed(Key::R) {
            ms.new_game();
        }

        if let PlayState::Init | PlayState::Playing(_) = ms.game.play_state {
            if i.key_pressed(Key::Enter) || i.key_pressed(Key::Space) {
                if i.modifiers.ctrl {
                    ms.hint(frame, ms.cursor_x, ms.cursor_y);
                } else {
                    ms.click(frame, ms.cursor_x, ms.cursor_y);
                }
            }
        }
    });

    let resp = ui.allocate_rect(board_rect, Sense::click_and_drag());
    if let PlayState::Init | PlayState::Playing(_) = ms.game.play_state {
        ui.input_mut(|i| {
            if i.pointer.velocity() != Vec2::ZERO {
                ms.cursor_visible = false;
            }

            if i.pointer.any_pressed() {
                ms.long_press = false;
            }

            if resp.is_pointer_button_down_on() {
                if let Some(pos) = i.pointer.press_origin() {
                    if let Some(start_time) = i.pointer.press_start_time() {
                        let duration = i.time - start_time;
                        if !ms.long_press && duration > 0.4 {
                            let (x, y) = board_idx_from_screen_pos(
                                ms.game.height,
                                board_offset,
                                cell_size,
                                pos,
                                flipped,
                            );
                            vibrate(100);
                            ms.hint(frame, x, y);
                            ms.long_press = true;
                        }
                    }
                }
            }

            if let Some(pos) = resp.interact_pointer_pos() {
                let mut clicked = false;
                let mut hint = false;
                if i.pointer.primary_released() {
                    clicked = true;
                } else if i.pointer.secondary_released() {
                    clicked = true;
                    hint = true;
                }

                if clicked && !ms.long_press {
                    let (x, y) = board_idx_from_screen_pos(
                        ms.game.height,
                        board_offset,
                        cell_size,
                        pos,
                        flipped,
                    );

                    if hint {
                        ms.hint(frame, x, y);
                    } else {
                        ms.click(frame, x, y);
                    }

                    if ms.game.is_in_bounds(x, y) {
                        ms.cursor_x = x;
                        ms.cursor_y = y;
                    }
                }
            }
        });
    }

    // draw
    let painter = ui.painter();
    let dark_mode = ui.visuals().dark_mode;
    let bg_color = ui.style().visuals.window_fill;
    let cell_stroke = Stroke::new(1.0, bg_color);
    painter.rect(board_rect, 0.0, bg_color, Stroke::NONE);

    let color_cursor = if dark_mode {
        Color32::from_rgb(0xd0, 0xe0, 0xff)
    } else {
        Color32::from_rgb(0x20, 0x40, 0x70)
    };

    let color_hide = if dark_mode {
        Color32::from_gray(0x40)
    } else {
        Color32::from_gray(0xa0)
    };
    let color_hint = if dark_mode {
        Color32::from_rgb(0xf0, 0xc0, 0x30)
    } else {
        Color32::from_rgb(0xf0, 0xc0, 0x30)
    };
    let color_show = if dark_mode {
        Color32::from_gray(0x80)
    } else {
        Color32::from_gray(0xc0)
    };
    let color_lose = if dark_mode {
        Color32::from_rgb(0xd0, 0x60, 0x30)
    } else {
        Color32::from_rgb(0xd0, 0x60, 0x30)
    };
    let colors_nums: [Color32; 8] = [
        Color32::BLUE,
        Color32::GREEN,
        Color32::RED,
        Color32::DARK_BLUE,
        Color32::DARK_RED,
        Color32::LIGHT_BLUE,
        Color32::BLACK,
        Color32::GRAY,
    ];

    for y in 0..ms.game.height {
        for x in 0..ms.game.width {
            let field = ms.game[(x, y)];

            let (x, y) = if flipped {
                (ms.game.height - y - 1, x)
            } else {
                (x, y)
            };
            let cell_pos = board_offset + Vec2::new(x as f32, y as f32) * cell_size;
            let cell_rect = Rect::from_min_size(cell_pos, cell_size);
            let cell_center_pos = cell_pos + cell_size / 2.0;
            let mut text_style = TextStyle::Monospace.resolve(ui.style().as_ref());
            text_style.size = cell_size.y * 0.8;

            match ms.game.play_state {
                PlayState::Init | PlayState::Playing(_) => match (field.state, field.visibility) {
                    (_, Visibility::Hide) => {
                        painter.rect(cell_rect, 0.0, color_hide, cell_stroke);
                    }
                    (_, Visibility::Hint) => {
                        painter.rect(cell_rect, 0.0, color_hint, cell_stroke);
                    }
                    (FieldState::Free(n), Visibility::Show) => {
                        painter.rect(cell_rect, 0.0, color_show, cell_stroke);
                        if n != 0 {
                            let num_color = colors_nums[n as usize - 1];
                            painter.text(
                                cell_center_pos,
                                Align2::CENTER_CENTER,
                                n,
                                text_style,
                                num_color,
                            );
                        }
                    }
                    (FieldState::Mine, Visibility::Show) => {
                        // Just for debugging
                        painter.rect(cell_rect, 0.0, Color32::GREEN, cell_stroke);
                    }
                },
                PlayState::Won(_) => match (field.state, field.visibility) {
                    (FieldState::Free(n), _) => {
                        painter.rect(cell_rect, 0.0, color_show, cell_stroke);
                        if n != 0 {
                            let num_color = colors_nums[n as usize - 1];
                            painter.text(
                                cell_center_pos,
                                Align2::CENTER_CENTER,
                                n,
                                text_style,
                                num_color,
                            );
                        }
                    }
                    (FieldState::Mine, Visibility::Hint) => {
                        painter.rect(cell_rect, 0.0, color_hint, cell_stroke);
                        painter.text(
                            cell_center_pos,
                            Align2::CENTER_CENTER,
                            "*",
                            text_style,
                            Color32::BLACK,
                        );
                    }
                    (FieldState::Mine, _) => {
                        painter.rect(cell_rect, 0.0, color_show, cell_stroke);
                        painter.text(
                            cell_center_pos,
                            Align2::CENTER_CENTER,
                            "*",
                            text_style,
                            Color32::BLACK,
                        );
                    }
                },
                PlayState::Lost(_) => match (field.state, field.visibility) {
                    (FieldState::Free(_), Visibility::Hide) => {
                        painter.rect(cell_rect, 0.0, color_hide, cell_stroke);
                    }
                    (FieldState::Free(_), Visibility::Hint) => {
                        painter.rect(cell_rect, 0.0, color_hint, cell_stroke);
                        painter.text(
                            cell_center_pos,
                            Align2::CENTER_CENTER,
                            "x",
                            text_style,
                            Color32::RED,
                        );
                    }
                    (FieldState::Free(n), Visibility::Show) => {
                        painter.rect(cell_rect, 0.0, color_show, cell_stroke);
                        if n != 0 {
                            let num_color = colors_nums[n as usize - 1];
                            painter.text(
                                cell_center_pos,
                                Align2::CENTER_CENTER,
                                n,
                                text_style,
                                num_color,
                            );
                        }
                    }
                    (FieldState::Mine, Visibility::Hide) => {
                        painter.rect(cell_rect, 0.0, color_show, cell_stroke);
                        painter.text(
                            cell_center_pos,
                            Align2::CENTER_CENTER,
                            "*",
                            text_style,
                            Color32::BLACK,
                        );
                    }
                    (FieldState::Mine, Visibility::Hint) => {
                        painter.rect(cell_rect, 0.0, color_hint, cell_stroke);
                        painter.text(
                            cell_center_pos,
                            Align2::CENTER_CENTER,
                            "*",
                            text_style,
                            Color32::BLACK,
                        );
                    }
                    (FieldState::Mine, Visibility::Show) => {
                        painter.rect(cell_rect, 0.0, color_lose, cell_stroke);
                        painter.text(
                            cell_center_pos,
                            Align2::CENTER_CENTER,
                            "*",
                            text_style,
                            Color32::BLACK,
                        );
                    }
                },
            }
        }
    }

    // cursor
    if ms.cursor_visible {
        let cursor_idx = if flipped {
            Vec2::new(
                (ms.game.height - ms.cursor_y - 1) as f32,
                ms.cursor_x as f32,
            )
        } else {
            Vec2::new(ms.cursor_x as f32, ms.cursor_y as f32)
        };
        let cursor_pos = board_offset + cursor_idx * cell_size;
        let cursor_rect = Rect::from_min_size(cursor_pos, cell_size);
        painter.rect(
            cursor_rect,
            4.0,
            Color32::TRANSPARENT,
            Stroke::new(2.0, color_cursor),
        );
    }

    if let PlayState::Won(_) | PlayState::Lost(_) = ms.game.play_state {
        let min_dimension = available_size.min_elem();
        let margin = Vec2::splat(min_dimension * 0.05);
        let scoreboard_width = 400.0;
        let scoreboard_offset =
            board_offset + Vec2::new(0.5 * (board_size.x - scoreboard_width), margin.y);
        let scoreboard_size = Vec2::new(scoreboard_width, board_size.y - 2.0 * margin.y);
        let rect = Rect::from_min_size(scoreboard_offset, scoreboard_size);
        painter.rect(
            rect,
            Rounding::same(min_dimension * 0.02),
            Color32::from_black_alpha(0xb0),
            Stroke::NONE,
        );

        let title_pos = scoreboard_offset + Vec2::new(0.5 * scoreboard_size.x, margin.y);
        let unambigous_text = if ms.unambigous {
            "unambigous"
        } else {
            "ambigous"
        };
        let title = format!("{} {}", ms.difficulty, unambigous_text);
        painter.text(
            title_pos,
            Align2::CENTER_TOP,
            title,
            FontId::proportional(30.0),
            Color32::from_white_alpha(0xb0),
        );

        let scores = &ms.highscores[ms.difficulty as usize + (3 * ms.unambigous as usize)];
        let is_same_mode = ms.difficulty == ms.game.difficulty && ms.unambigous == ms.game.unambigous;

        let mut score_y = scoreboard_offset.y + 2.0 * margin.y + 30.0;
        let num_x = scoreboard_offset.x + margin.x;
        let duration_x = scoreboard_offset.x + scoreboard_size.x - margin.x;
        for (i, score) in scores.iter().take(10).enumerate() {
            let mut text_color = Color32::from_white_alpha(0xb0);
            if is_same_mode {
                if let PlayState::Won(d) = ms.game.play_state {
                    if *score == d {
                        text_color = Color32::from_rgba_unmultiplied(0xff, 0xc0, 0x30, 0xb0);
                    }
                }
            }
            painter.text(
                Pos2::new(num_x, score_y),
                Align2::LEFT_TOP,
                format!("{}.", i + 1),
                FontId::proportional(30.0),
                text_color,
            );
            painter.text(
                Pos2::new(duration_x, score_y),
                Align2::RIGHT_TOP,
                format_duration(*score),
                FontId::proportional(30.0),
                text_color,
            );
            score_y += 40.0;
        }
    }
}
