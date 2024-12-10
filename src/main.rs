#![feature(adt_const_params)]
mod coinbase;
mod spinlock;
use coinbase::CoinBaseApiClient;
use eframe::egui::{self, Color32, Pos2, Rect, UiBuilder, Vec2};
use egui_plot::{Bar, BarChart, Plot};

fn main() {
    std::thread::spawn(|| {
        CoinBaseApiClient::new().connect_to_api();
    });
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "My egui App",
        native_options,
        Box::new(|cc| Ok(Box::new(MyEguiApp::new(cc)))),
    )
    .unwrap();
}

#[derive(Default)]
struct MyEguiApp {
    username: String,
    n_bins: u32,
}

impl MyEguiApp {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self {
            username: String::from("Victor"),
            n_bins: 100,
        }
    }

    fn depth_chart_ui(&mut self, ui: &mut egui::Ui) {
        let width = 2.0 * std::f64::consts::PI / self.n_bins as f64;
        let chart = BarChart::new(
            (0..=self.n_bins)
                .map(|x| width * x as f64)
                .map(|x| (x, (x * x).sin()))
                // The 10 factor here is purely for a nice 1:1 aspect ratio
                .map(|(x, f)| Bar::new(x, f * 10.0).width(width).fill(Color32::BLUE))
                .collect(),
        );

        Plot::new("Normal Distribution Demo")
            .clamp_grid(true)
            .show(ui, |plot_ui| plot_ui.bar_chart(chart));
    }

    fn bar_chart_ui(&mut self, ui: &mut egui::Ui) {
        ui.heading("My egui Application");
        ui.horizontal(|ui| {
            let name_label = ui.label("Your name: ");
            ui.text_edit_singleline(&mut self.username)
                .labelled_by(name_label.id);
        });
        ui.add(egui::Slider::new(&mut self.n_bins, 0..=1000).text("n_bins"));
        if ui.button("Increment").clicked() {
            self.n_bins += 1;
        }
        ui.label(format!("Hello '{}', n_bars {}", self.username, self.n_bins));
    }
}

impl eframe::App for MyEguiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            let Vec2 { x, y } = ui.available_size();
            let top_rect = Rect::from_min_size(Pos2::default(), Vec2 { x, y: y / 2.0 });
            let ui_builder = UiBuilder::default().max_rect(top_rect);
            ui.scope_builder(ui_builder, |ui| self.depth_chart_ui(ui));

            let bot_rect = Rect::from_min_size(Pos2 { x: 0.0, y: y / 2.0 }, Vec2 { x, y: y / 2.0 });
            let ui_builder = UiBuilder::default().max_rect(bot_rect);
            ui.scope_builder(ui_builder, |ui| self.bar_chart_ui(ui));
        });
    }
}
