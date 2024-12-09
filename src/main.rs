use eframe::egui::{self, Color32, Pos2, Rect, UiBuilder, Vec2};
use egui_plot::{Bar, BarChart, Plot};

fn main() {
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
    name_top: String,
    age_top: u32,
    name_bot: String,
    age_bot: u32,
}

impl MyEguiApp {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self::default()
    }

    fn depth_chart_ui(&mut self, ui: &mut egui::Ui) {
        let mut chart = BarChart::new(
            (-(self.age_bot as i32)..=self.age_bot as i32)
                .step_by(10)
                .map(|x| x as f64 * 0.01)
                .map(|x| {
                    (
                        x,
                        (-x * x / 2.0).exp() / (2.0 * std::f64::consts::PI).sqrt(),
                    )
                })
                // The 10 factor here is purely for a nice 1:1 aspect ratio
                .map(|(x, f)| Bar::new(x, f * 10.0).width(0.1).fill(Color32::BLUE))
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
            ui.text_edit_singleline(&mut self.name_bot)
                .labelled_by(name_label.id);
        });
        ui.add(egui::Slider::new(&mut self.age_bot, 0..=1000).text("age"));
        if ui.button("Increment").clicked() {
            self.age_bot += 1;
        }
        ui.label(format!("Hello '{}', age {}", self.name_bot, self.age_bot));
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
