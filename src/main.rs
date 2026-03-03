// TODO: Link Audio and GPU to create generative art

extern crate nannou;
use nannou::prelude::*;
use nannou_egui::{self, Egui, egui};

// Audio
use nannou_audio as audio;
use ringbuf::{HeapCons, HeapProd, HeapRb, traits::*};

const BUFFER_SIZE: usize = 1024;

struct Model {
    _stream: audio::Stream<CaptureModel>, // capture live audio
    consumer: HeapCons<f32>,              // grab from heap
    samples: Vec<f32>,                    // sample size
    egui: Egui,                           // egui state lives here
    r: f32,
    g: f32,
    b: f32,
}

// Insert items into the ringbuf, or basically take in audio
struct CaptureModel {
    producer: HeapProd<f32>,
}

// Capture input events such as mouse and keyboard
fn raw_window_event(_app: &App, model: &mut Model, event: &nannou::winit::event::WindowEvent) {
    model.egui.handle_raw_event(event);
}

// Capture from microphone
fn capture(model: &mut CaptureModel, buffer: &audio::Buffer) {
    // Take in audio snippets as buffer and push to ringbuf
    for frame in buffer.frames() {
        let _ = model.producer.try_push(frame[0]);
    }
}

fn model(app: &App) -> Model {
    let window_id = app
        .new_window()
        .view(view)
        .raw_event(raw_window_event)
        .build()
        .unwrap();
    let window = app.window(window_id).unwrap();

    // Init ringbuf
    let ring = HeapRb::<f32>::new(BUFFER_SIZE * 2); // to be safe
    let (producer, consumer) = ring.split();

    // Init microphone
    let audio_host = audio::Host::new();
    let capture_model = CaptureModel { producer };
    let stream = audio_host
        .new_input_stream(capture_model)
        .capture(capture)
        .build()
        .unwrap();
    stream.play().unwrap();

    Model {
        _stream: stream,
        consumer,
        samples: vec![0.0; BUFFER_SIZE],
        egui: Egui::from_window(&window),
        r: 1.0,
        b: 1.0,
        g: 1.0,
    }
}

fn update(_app: &App, model: &mut Model, update: Update) {
    // Floating Setting Window
    model.egui.set_elapsed_time(update.since_start);
    let ctx = model.egui.begin_frame();
    egui::Window::new("Settings").show(&ctx, |ui| {
        ui.add(egui::Slider::new(&mut model.r, 0.0..=1.0).text("Red"));

        ui.add(egui::Slider::new(&mut model.g, 0.0..=1.0).text("Green"));

        ui.add(egui::Slider::new(&mut model.b, 0.0..=1.0).text("Blue"));
    });

    // Taking in audio from microphone
    while let Some(sample) = model.consumer.try_pop() {
        model.samples.remove(0);
        model.samples.push(sample);
    }
}

fn view(app: &App, model: &Model, frame: Frame) {
    // Draw it visually on screen
    let draw = app.draw();
    draw.background().rgb(model.r, model.g, model.b); // background color

    let win = app.window_rect(); // retrieve boundaries of window
    let points: Vec<Point2> = model // construct points from the window
        .samples
        .iter()
        .enumerate()
        .map(|(i, &s)| {
            let x = map_range(i, 0, model.samples.len(), win.left(), win.right());
            let y = s * win.h() * 0.5;
            pt2(x, y)
        })
        .collect();

    // draw polyline with thickness of 2.0
    draw.polyline()
        .weight(2.0)
        .rgb(0.0, 0.0, 0.0)
        .points(points.clone());
    draw.to_frame(app, &frame).unwrap();

    // Displayed floating setting window
    model.egui.draw_to_frame(&frame).unwrap();
}

// Entry Point
fn main() {
    nannou::app(model).update(update).run();
}
