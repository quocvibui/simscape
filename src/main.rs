// TODO: Link Audio and GPU to create generative art
extern crate nannou;
use nannou::prelude::*;
use nannou::wgpu::BufferInitDescriptor;
use nannou_egui::{self, Egui, egui};

// Audio
use nannou_audio as audio;
use ringbuf::{HeapCons, HeapProd, HeapRb, traits::*};

const BUFFER_SIZE: usize = 1024;

struct Model {
    render_pipeline: wgpu::RenderPipeline, // gpu pipeline
    bind_group: wgpu::BindGroup,           // binding gpu
    uniform_buffer: wgpu::Buffer,          // gpu accessible buffer
    _stream: audio::Stream<CaptureModel>,  // capture live audio
    consumer: HeapCons<f32>,               // grab from heap
    samples: Vec<f32>,                     // sample size
    egui: Egui,                            // egui state lives here
    r: f32,
    g: f32,
    b: f32,
}

// GPU struct
#[repr(C)]
#[derive(Clone, Copy)]
struct Uniforms {
    time: f32,
    amplitude: f32, // for audio
    resolution: [f32; 2],
    color: [f32; 4], // r, g, b, pad
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

    /* Mostly GPU stuff */
    let device = window.device(); // wgpu logical device
    let format = Frame::TEXTURE_FORMAT;
    let sample_count = window.msaa_samples();

    // Shader Module, take in shader.wgsl
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("shader"),
        source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
    });

    // Initialize a uniform buffer to pass data from CPU to shader
    let uniforms = Uniforms {
        time: 0.0,
        amplitude: 0.0,
        resolution: [800.0, 600.0],
        color: [0.0, 0.0, 0.0, 0.0],
    };
    let uniforms_arr = [uniforms];
    let uniforms_bytes = unsafe { wgpu::bytes::from_slice(&uniforms_arr) };
    let uniform_buffer = device.create_buffer_init(&BufferInitDescriptor {
        label: Some("uniforms"),
        contents: uniforms_bytes,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
    });

    // Bind buffer to shader
    let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("bind_group_layout"),
        entries: &[wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }],
    });

    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("bind_group"),
        layout: &bind_group_layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: uniform_buffer.as_entire_binding(),
        }],
    });

    // Init whole pipeline, assembles everything to render pipeline
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("pipeline_layout"),
        bind_group_layouts: &[&bind_group_layout],
        push_constant_ranges: &[],
    });

    let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("render_pipeline"),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: "vs_main",
            buffers: &[],
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: "fs_main",
            targets: &[Some(wgpu::ColorTargetState {
                format,
                blend: Some(wgpu::BlendState::REPLACE),
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        primitive: wgpu::PrimitiveState::default(),
        depth_stencil: None,
        multisample: wgpu::MultisampleState {
            count: sample_count,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
        multiview: None,
    });

    /* No longer GPU stuff */

    // Init ringbuf for audio
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
        render_pipeline,
        bind_group,
        uniform_buffer,
        _stream: stream,
        consumer,
        samples: vec![0.0; BUFFER_SIZE],
        egui: Egui::from_window(&window),
        r: 1.0,
        b: 1.0,
        g: 1.0,
    }
}

fn update(app: &App, model: &mut Model, update: Update) {
    // Floating Setting Window
    model.egui.set_elapsed_time(update.since_start);
    let ctx = model.egui.begin_frame();
    egui::Window::new("Settings").show(&ctx, |ui| {
        ui.add(egui::Slider::new(&mut model.r, 0.0..=1.0).text("Red"));

        ui.add(egui::Slider::new(&mut model.g, 0.0..=1.0).text("Green"));

        ui.add(egui::Slider::new(&mut model.b, 0.0..=1.0).text("Blue"));
    });

    // Taking in audio from microphone, need this for fresh audio
    while let Some(sample) = model.consumer.try_pop() {
        model.samples.remove(0);
        model.samples.push(sample);
    }

    // Calculate Root Mean Square of audio buffer
    let amplitude = (model.samples.iter().map(|s| s * s).sum::<f32>() / BUFFER_SIZE as f32).sqrt();

    // GPU Visualization Update
    let window = app.main_window();
    let queue = window.queue();
    let win = app.window_rect();

    let uniforms = Uniforms {
        time: app.time,
        amplitude: amplitude,
        resolution: [win.w(), win.h()],
        color: [model.r, model.g, model.b, 0.0], // pass in rgb and added paddings
    };
    let uniforms_arr = [uniforms];
    let uniforms_bytes = unsafe { wgpu::bytes::from_slice(&uniforms_arr) };
    queue.write_buffer(&model.uniform_buffer, 0, uniforms_bytes);
}

fn view(_app: &App, model: &Model, frame: Frame) {
    // Gpu Visualization Displayed
    {
        let mut encoder = frame.command_encoder();
        let mut render_pass = wgpu::RenderPassBuilder::new()
            .color_attachment(frame.texture_view(), |color| color)
            .begin(&mut encoder);
        render_pass.set_pipeline(&model.render_pipeline);
        render_pass.set_bind_group(0, &model.bind_group, &[]);
        render_pass.draw(0..180, 0..1); // 60 triangles x 3 vertices
    }

    // Displayed floating setting window
    model.egui.draw_to_frame(&frame).unwrap();
}

// Entry Point
fn main() {
    nannou::app(model).update(update).run();
}
