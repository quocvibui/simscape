extern crate nannou;
use nannou::prelude::*;
use nannou_egui::{self, egui, Egui};

fn main() {
    nannou::app(model)
        .update(update)
        .run();
}

struct Model {
    egui: Egui, // egui state lives here
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
        egui: Egui::from_window(&window)
    }
}

fn update(_app: &App, model: &mut Model, update: Update) {
    model.egui.set_elapsed_time(update.since_start);
    let ctx = model.egui.begin_frame();
    egui::Window::new("My Panel").show(&ctx, |ui| {
        if ui.button("Click me").clicked() {
            // do something here
        }
    });
}

fn raw_window_event(_app: &App, model: &mut Model, event: &nannou::winit::event::WindowEvent) {
    model.egui.handle_raw_event(event);
}

fn view(_app: &App, model: &Model, frame: Frame){
    frame.clear(PURPLE);
    model.egui.draw_to_frame(&frame).unwrap();
}

