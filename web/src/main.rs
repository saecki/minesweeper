use eframe::{App, CreationContext, WebOptions};
use egui::{CentralPanel, Frame};
use minesweeper::Minesweeper;

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
    eframe::WebLogger::init(log::LevelFilter::Debug).ok();

    let options = WebOptions::default();
    wasm_bindgen_futures::spawn_local(async {
        let res = eframe::WebRunner::new()
            .start(
                "minesweeper_canvas_id",
                options,
                Box::new(|c| Box::new(MinesweeperApp::new(c))),
            )
            .await;
        if let Err(e) = res {
            println!("error running app: {e:?}");
        }
    });
}
