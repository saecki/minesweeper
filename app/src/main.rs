#![windows_subsystem = "windows"]

use eframe::{App, CreationContext, NativeOptions};
use egui::{CentralPanel, Frame};
use minesweeper::Minesweeper;

#[derive(Default)]
struct MinesweeperApp {
    minesweeper: Minesweeper,
}

impl MinesweeperApp {
    fn new(cc: &CreationContext) -> Self {
        let minesweeper = cc
            .storage
            .and_then(|s| eframe::get_value(s, eframe::APP_KEY))
            .unwrap_or_default();
        Self { minesweeper }
    }
}

impl App for MinesweeperApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        CentralPanel::default()
            .frame(Frame::none().fill(ctx.style().visuals.window_fill))
            .show(ctx, |ui| minesweeper::update(ui, &mut self.minesweeper));
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, &self.minesweeper);
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
