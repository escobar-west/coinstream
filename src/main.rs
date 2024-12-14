mod coinbase;
mod orderbook;
mod spinlock;
use coinbase::CoinBaseApiClient;
use eframe::egui::{self, Color32, Visuals};
use egui_plot::{Line, Plot, PlotBounds};
use std::time::Duration;

fn main() {
    eframe::run_native(
        "Bitcoin Depth Chart",
        eframe::NativeOptions::default(),
        Box::new(|cc| Ok(Box::new(MyEguiApp::new(cc)))),
    )
    .unwrap();
}

struct MyEguiApp {
    api: CoinBaseApiClient,
}

impl MyEguiApp {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self {
            api: CoinBaseApiClient::new(),
        }
    }

    fn depth_chart_ui(&mut self, ui: &mut egui::Ui) {
        let x_range = 100;
        let (bid_points, ask_points) = {
            let orderbook = self.api.orderbook.lock();
            if orderbook.bids.is_empty() || orderbook.asks.is_empty() {
                return;
            }

            let mut iter_flag = true;
            let mut bid_total_amt = 0.0;
            let min_bid = orderbook.bids.first_key_value().unwrap().0 .0.int - x_range;
            let bid_points: Vec<[f64; 2]> = orderbook
                .bids
                .iter()
                .take_while(|(x, _)| std::mem::replace(&mut iter_flag, min_bid <= x.0.int))
                .flat_map(|(x, f)| {
                    let coords = (&x.0).into();
                    let amt: f64 = f.into();
                    let point0 = [coords, bid_total_amt];
                    bid_total_amt += amt;
                    let point1 = [coords, bid_total_amt];
                    [point0, point1]
                })
                .collect();

            iter_flag = true;
            let mut ask_total_amt = 0.0;
            let max_ask = orderbook.asks.first_key_value().unwrap().0.int + x_range + 1;
            let ask_points: Vec<[f64; 2]> = orderbook
                .asks
                .iter()
                .take_while(|(x, _)| std::mem::replace(&mut iter_flag, x.int <= max_ask))
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
            .allow_zoom(false)
            .allow_drag(false)
            .show(ui, |plot_ui| {
                plot_ui.set_plot_bounds(PlotBounds::from_min_max(
                    [mid_point - x_range as f64, 0.0],
                    [mid_point + x_range as f64, 60.0],
                ));
                plot_ui.line(bid_line);
                plot_ui.line(ask_line);
            });
    }
}

impl eframe::App for MyEguiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.set_visuals(Visuals::dark());
        egui::CentralPanel::default().show(ctx, |ui| {
            self.depth_chart_ui(ui);
        });
        ctx.request_repaint_after(Duration::from_millis(50));
    }
}
