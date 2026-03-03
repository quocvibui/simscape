extern crate nannou;
use nannou::prelude::*;
use nannou_egui::{self, Egui, egui};

fn main() {
    nannou::app(model).update(update).run();
}

struct Model {
    egui: Egui, // egui state lives here
    r: f64,
    g: f64,
    b: f64,
}

fn model(app: &App) -> Model {
    let window_id = app
        .new_window()
        .view(view)
        .raw_event(raw_window_event)
        .build()
        .unwrap();
    let window = app.window(window_id).unwrap();

    Model {
        egui: Egui::from_window(&window),
        r: 0.0,
        b: 0.0,
        g: 0.0,
    }
}

fn update(_app: &App, model: &mut Model, update: Update) {
    model.egui.set_elapsed_time(update.since_start);
    let ctx = model.egui.begin_frame();

    // Floating Setting Window
    egui::Window::new("Settings").show(&ctx, |ui| {
        ui.add(egui::Slider::new(&mut model.r, 0.0..=1.0).text("Red"));

        ui.add(egui::Slider::new(&mut model.g, 0.0..=1.0).text("Green"));

        ui.add(egui::Slider::new(&mut model.b, 0.0..=1.0).text("Blue"));
    });
}

fn raw_window_event(_app: &App, model: &mut Model, event: &nannou::winit::event::WindowEvent) {
    model.egui.handle_raw_event(event);
}

fn view(_app: &App, model: &Model, frame: Frame) {
    frame.clear(nannou::color::rgb(model.r, model.g, model.b));
    model.egui.draw_to_frame(&frame).unwrap();
}
