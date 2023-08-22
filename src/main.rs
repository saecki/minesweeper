use eframe::{App, CreationContext, NativeOptions};
use egui::{Align2, CentralPanel, Color32, Frame, Key, Rect, Stroke, TextStyle, Vec2};
use rand::Rng;

const GAME_WIDTH: i16 = 20;
const GAME_HEIGHT: i16 = 14;
const MINE_PROBABILITY: f64 = 0.18;

struct MinesweeperApp {
    game: Game,
    cursor_x: i16,
    cursor_y: i16,
}

struct Game {
    play_state: PlayState,
    width: i16,
    height: i16,
    fields: Vec<Field>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PlayState {
    Playing,
    Won,
    Lost,
}

impl Game {
    fn new(width: i16, height: i16, probability: f64) -> Self {
        let len = (width * height) as usize;
        let mut game = Game {
            play_state: PlayState::Playing,
            width,
            height,
            fields: vec![Field::free(0); len],
        };

        let mut rng = rand::thread_rng();
        for y in 0..height {
            for x in 0..width {
                if rng.gen_bool(probability) {
                    game[(x, y)] = Field::mine();

                    game.increment_field((x - 1, y - 1));
                    game.increment_field((x - 1, y + 0));
                    game.increment_field((x - 1, y + 1));
                    game.increment_field((x + 0, y - 1));
                    game.increment_field((x + 0, y + 1));
                    game.increment_field((x + 1, y - 1));
                    game.increment_field((x + 1, y + 0));
                    game.increment_field((x + 1, y + 1));
                }
            }
        }

        game
    }

    fn increment_field(&mut self, (x, y): (i16, i16)) {
        if x >= 0 && x < self.width && y >= 0 && y < self.height {
            if let FieldState::Free(neighbors) = &mut self[(x, y)].state {
                *neighbors += 1;
            }
        }
    }

    fn click(&mut self, (x, y): (i16, i16)) {
        let field = &mut self[(x, y)];
        if field.show == ShowState::Hint {
            return;
        }

        match field.state {
            FieldState::Free(_) => {
                self.show_neighbors((x, y));
                self.check_if_won();
            }
            FieldState::Mine => self.lose(),
        }
    }

    fn hint(&mut self, (x, y): (i16, i16)) {
        let field = &mut self[(x, y)];
        if field.show == ShowState::Hint {
            field.show = ShowState::Hide;
        } else if field.show == ShowState::Hide {
            field.show = ShowState::Hint;
        }
    }

    fn lose(&mut self) {
        self.play_state = PlayState::Lost;
        self.show_all();
    }

    fn check_if_won(&mut self) {
        for f in self.fields.iter() {
            if let FieldState::Free(_) = f.state {
                if f.show != ShowState::Show {
                    return;
                }
            }
        }

        self.play_state = PlayState::Won;
        self.show_all();
    }

    fn show_all(&mut self) {
        for f in self.fields.iter_mut() {
            f.show = ShowState::Show;
        }
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
                let available_size = ui.available_size();
                let cells = Vec2::new(self.game.width as f32, self.game.height as f32);
                let ratio = available_size / cells;
                let cell_size = Vec2::splat(ratio.min_elem());
                let board_size = cells * cell_size;
                let board_offset = (available_size - board_size) / 2.0;
                let board_rect = Rect::from_min_size(board_offset.to_pos2(), board_size);

                // input
                // TODO: make it impossible to lose on the click
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
                                let cell_idx = (pos.to_vec2() - board_offset) / cell_size;
                                let (x, y) = (cell_idx.x as i16, cell_idx.y as i16);

                                if x >= 0 && x < self.game.width && y >= 0 && y < self.game.height {
                                    if hint {
                                        self.game.hint((x, y));
                                    } else {
                                        self.game.click((x, y));
                                    }
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
                        let cell_rect = Rect::from_min_size(cell_pos.to_pos2(), cell_size);

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
                                                cell_center_pos.to_pos2(),
                                                Align2::CENTER_CENTER,
                                                c,
                                                text_style,
                                                num_color,
                                            );
                                        }
                                    }
                                    FieldState::Mine => {
                                        let color = match self.game.play_state {
                                            PlayState::Won => Color32::from_gray(0x80),
                                            PlayState::Lost => Color32::from_rgb(0xd0, 0x60, 0x30),
                                            PlayState::Playing => unreachable!("can't show a mine if still playing"),
                                        };
                                        painter.rect(cell_rect, 0.0, color, cell_stroke);
                                        painter.text(
                                            cell_center_pos.to_pos2(),
                                            Align2::CENTER_CENTER,
                                            "#",
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
                let cursor_rect = Rect::from_min_size(cursor_pos.to_pos2(), cell_size);
                painter.rect(
                    cursor_rect,
                    0.0,
                    Color32::TRANSPARENT,
                    Stroke::new(2.0, Color32::from_rgb(0xd0, 0xd0, 0xf0)),
                );
            });
    }
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
