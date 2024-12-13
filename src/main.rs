#![allow(dead_code)]
mod coinbase;
mod orderbook;
mod spinlock;
use coinbase::CoinBaseApiClient;
use eframe::egui::{self, Color32, Pos2, Rect, UiBuilder, Vec2};
use egui_plot::{Bar, BarChart, Plot, PlotBounds};

fn main() {
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "My egui App",
        native_options,
        Box::new(|cc| Ok(Box::new(MyEguiApp::new(cc)))),
    )
    .unwrap();
}

struct MyEguiApp {
    api: CoinBaseApiClient,
    username: String,
    n_bins: u32,
}

impl MyEguiApp {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self {
            api: CoinBaseApiClient::new(),
            username: String::from("Victor"),
            n_bins: 100,
        }
    }

    fn depth_chart_ui(&mut self, ui: &mut egui::Ui) {
        let n_bars = 200;
        let orderbook = {
            let orderbook = self.api.orderbook.lock().clone();
            orderbook
        };
        let Some((prev_bid_coords, prev_bid_amt)) = orderbook.bids.first_key_value() else {
            return;
        };
        let Some((prev_ask_coords, prev_ask_amt)) = orderbook.asks.first_key_value() else {
            return;
        };
        let (mut prev_bid_coords, mut prev_bid_amt): (f64, f64) =
            ((&prev_bid_coords.0).into(), prev_bid_amt.into());
        let (mut prev_ask_coords, mut prev_ask_amt): (f64, f64) =
            (prev_ask_coords.into(), prev_ask_amt.into());
        let mid_point = (prev_bid_coords + prev_ask_coords) / 2.0;
        let chart = BarChart::new(
            orderbook
                .bids
                .iter()
                .skip(1)
                .take(n_bars)
                .map(|(x, f)| {
                    let coords = (&x.0).into();
                    let amt: f64 = f.into();
                    let prev_width = prev_bid_coords - coords;
                    let bar = Bar::new(prev_bid_coords - prev_width / 2.0, prev_bid_amt)
                        .width(prev_width)
                        .fill(Color32::GREEN);
                    prev_bid_coords = coords;
                    prev_bid_amt += amt;
                    bar
                })
                .chain(orderbook.asks.iter().skip(1).take(n_bars).map(|(x, f)| {
                    let coords = x.into();
                    let amt: f64 = f.into();
                    let prev_width = coords - prev_ask_coords;
                    let bar = Bar::new(prev_ask_coords + prev_width / 2.0, prev_ask_amt)
                        .width(prev_width)
                        .fill(Color32::RED);
                    prev_ask_coords = coords;
                    prev_ask_amt += amt;
                    bar
                }))
                .collect(),
        );
        Plot::new("Depth Chart")
            .show_grid(false)
            .show(ui, |plot_ui| {
                plot_ui.set_plot_bounds(PlotBounds::from_min_max(
                    [mid_point - 100.0, 0.0],
                    [mid_point + 60.0, 60.0],
                ));
                plot_ui.bar_chart(chart)
            });
    }

    fn bar_chart_ui(&mut self, ui: &mut egui::Ui) {
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
        ctx.request_repaint();
    }
}
