#![windows_subsystem = "windows"]

use eframe::{App, CreationContext, NativeOptions};
use egui::{CentralPanel, Frame};
use minesweeper::Minesweeper;

struct MinesweeperApp {
    minesweeper: Minesweeper,
}

impl MinesweeperApp {
    fn new(_cc: &CreationContext) -> Self {
        Self {
            minesweeper: Minesweeper::new(),
        }
    }
}

impl App for MinesweeperApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        CentralPanel::default()
            .frame(Frame::none())
            .show(ctx, |ui| minesweeper::update(ui, &mut self.minesweeper));
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
