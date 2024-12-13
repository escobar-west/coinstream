#![allow(dead_code)]
mod coinbase;
mod orderbook;
mod spinlock;
use std::cell::Cell;

use coinbase::CoinBaseApiClient;
use eframe::egui::{self, Color32, Pos2, Rect, UiBuilder, Vec2, Visuals};
use egui_plot::{Line, Plot, PlotBounds};

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
        let x_range = 100;
        let (bid_points, ask_points) = {
            let orderbook = self.api.orderbook.lock();
            if orderbook.bids.is_empty() || orderbook.asks.is_empty() {
                return;
            }
            let should_cont = Cell::new(true);
            let min_bid = orderbook.bids.first_key_value().unwrap().0 .0.int - x_range;
            let mut bid_total_amt = 0.0;
            let bid_points: Vec<[f64; 2]> = orderbook
                .bids
                .iter()
                .take_while(|_| should_cont.get())
                .flat_map(|(x, f)| {
                    if x.0.int < min_bid {
                        should_cont.set(false);
                    }
                    let coords = (&x.0).into();
                    let amt: f64 = f.into();
                    let point0 = [coords, bid_total_amt];
                    bid_total_amt += amt;
                    let point1 = [coords, bid_total_amt];
                    [point0, point1]
                })
                .collect();
            should_cont.set(true);
            let max_ask = orderbook.asks.first_key_value().unwrap().0.int + x_range + 1;
            let mut ask_total_amt = 0.0;
            let ask_points: Vec<[f64; 2]> = orderbook
                .asks
                .iter()
                .take_while(|_| should_cont.get())
                .flat_map(|(x, f)| {
                    if x.int > max_ask {
                        should_cont.set(false);
                    }
                    let coords = x.into();
                    let amt: f64 = f.into();
                    let point0 = [coords, ask_total_amt];
                    ask_total_amt += amt;
                    let point1 = [coords, ask_total_amt];
                    [point0, point1]
                })
                .collect();
            (bid_points, ask_points)
        };
        let mid_point = (bid_points[0][0] + ask_points[0][0]) / 2.0;
        let bid_line = Line::new(bid_points).fill(0.0).color(Color32::GREEN);
        let ask_line = Line::new(ask_points).fill(0.0).color(Color32::RED);
        Plot::new("Depth Chart")
            .show_grid(false)
            .show(ui, |plot_ui| {
                plot_ui.set_plot_bounds(PlotBounds::from_min_max(
                    [mid_point - x_range as f64, 0.0],
                    [mid_point + x_range as f64, 60.0],
                ));
                plot_ui.line(bid_line);
                plot_ui.line(ask_line);
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
        ctx.set_visuals(Visuals::dark());
        egui::CentralPanel::default().show(ctx, |ui| {
            self.depth_chart_ui(ui);
            //let Vec2 { x, y } = ui.available_size();
            //let top_rect = Rect::from_min_size(Pos2::default(), Vec2 { x, y: y / 2.0 });
            //let ui_builder = UiBuilder::default().max_rect(top_rect);
            //ui.scope_builder(ui_builder, |ui| self.depth_chart_ui(ui));

            //let bot_rect = Rect::from_min_size(Pos2 { x: 0.0, y: y / 2.0 }, Vec2 { x, y: y / 2.0 });
            //let ui_builder = UiBuilder::default().max_rect(bot_rect);
            //ui.scope_builder(ui_builder, |ui| self.bar_chart_ui(ui));
        });
        ctx.request_repaint();
    }
}
