#![allow(dead_code)]
mod coinbase;
mod orderbook;
mod spinlock;
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
        let n_orders = 500;
        let orderbook = {
            let orderbook = self.api.orderbook.lock();
            if orderbook.bids.is_empty() || orderbook.asks.is_empty() {
                return;
            }
            orderbook.clone()
        };
        let mut bid_total_amt = 0.0;
        let mut bid_points = Vec::with_capacity(2 * n_orders);
        for (x, f) in orderbook.bids.iter().take(n_orders) {
            let coords = (&x.0).into();
            let amt: f64 = f.into();
            bid_points.push([coords, bid_total_amt]);
            bid_total_amt += amt;
            bid_points.push([coords, bid_total_amt]);
        }
        let mut ask_total_amt = 0.0;
        let mut ask_points = Vec::with_capacity(2 * n_orders);
        for (x, f) in orderbook.asks.iter().take(n_orders) {
            let coords = x.into();
            let amt: f64 = f.into();
            ask_points.push([coords, ask_total_amt]);
            ask_total_amt += amt;
            ask_points.push([coords, ask_total_amt]);
        }
        let mid_point = (bid_points[0][0] + ask_points[0][0]) / 2.0;
        let bid_line = Line::new(bid_points).fill(0.0).color(Color32::GREEN);
        let ask_line = Line::new(ask_points).fill(0.0).color(Color32::RED);
        Plot::new("Depth Chart")
            .show_grid(false)
            .show(ui, |plot_ui| {
                plot_ui.set_plot_bounds(PlotBounds::from_min_max(
                    [mid_point - 100.0, 0.0],
                    [mid_point + 100.0, 60.0],
                ));
                plot_ui.line(bid_line);
                plot_ui.line(ask_line);
            });
    }

    fn depth_chart_ui_new(&mut self, ui: &mut egui::Ui) {
        let x_range = 150;
        let (bid_points, ask_points) = {
            let orderbook = self.api.orderbook.lock();
            if orderbook.bids.is_empty() || orderbook.asks.is_empty() {
                return;
            }
            let first_bid = orderbook.bids.first_key_value().unwrap().0 .0.int;
            let first_ask = orderbook.asks.first_key_value().unwrap().0.int;
            let mid_point = (first_bid + first_ask) / 2;
            let mut bid_total_amt = 0.0;
            let bid_points: Vec<[f64; 2]> = orderbook
                .bids
                .iter()
                .take_while(|(x, _)| x.0.int >= mid_point - x_range)
                .flat_map(|(x, f)| {
                    let coords = (&x.0).into();
                    let amt: f64 = f.into();
                    let point0 = [coords, bid_total_amt];
                    bid_total_amt += amt;
                    let point1 = [coords, bid_total_amt];
                    [point0, point1]
                })
                .collect();
            let mut ask_total_amt = 0.0;
            let ask_points: Vec<[f64; 2]> = orderbook
                .asks
                .iter()
                .take_while(|(x, _)| x.int <= mid_point + x_range)
                .flat_map(|(x, f)| {
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
                    [mid_point - (x_range - 50) as f64, 0.0],
                    [mid_point + (x_range - 50) as f64, 60.0],
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
            self.depth_chart_ui_new(ui);
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
