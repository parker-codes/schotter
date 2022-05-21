use nannou::prelude::*;
use nannou_egui::egui::{Area, Slider};
use nannou_egui::Egui;
use std::fs;
use std::io::ErrorKind;

const ROWS: u32 = 22;
const COLS: u32 = 12;
const SIZE: u32 = 30;
const LINE_WIDTH: f32 = 0.06;
const MARGIN: u32 = 35;
const WIDTH: u32 = COLS * SIZE + 2 * MARGIN;
const HEIGHT: u32 = ROWS * SIZE + 2 * MARGIN;

const ASSETS_DIR: &'static str = "assets";

fn main() {
    nannou::app(model)
        .update(update)
        .loop_mode(LoopMode::refresh_sync())
        .run();
}

struct Model {
    main_window: WindowId,
    displacement: f32,
    rotation: f32,
    motion: f32,
    gravel: Vec<Stone>,

    controls_ui: Egui,

    // for recording
    current_recording_name: String,
    current_frame: u32,
    recording: bool,
}

fn model(app: &App) -> Model {
    let main_window = app
        .new_window()
        .title(app.exe_name().unwrap())
        .size(WIDTH, HEIGHT)
        .view(view)
        .key_pressed(key_pressed)
        .build()
        .unwrap();

    let displacement = 1.0;
    let rotation = 1.0;
    let motion = 0.5;

    let mut gravel = Vec::new();
    for y in 0..ROWS {
        for x in 0..COLS {
            let stone = Stone::new(x as f32, y as f32);
            gravel.push(stone);
        }
    }

    let controls_window_id = app
        .new_window()
        .title(app.exe_name().unwrap() + " Control Panel")
        .size(300, 200)
        .view(controls_view)
        .raw_event(raw_controls_event)
        .key_pressed(key_pressed)
        .build()
        .unwrap();
    let controls_window = app.window(controls_window_id).unwrap();
    let controls_ui = Egui::from_window(&controls_window);

    let current_recording_name = String::new();
    let current_frame = 0;
    let recording = false;

    Model {
        main_window,
        displacement,
        rotation,
        motion,
        gravel,

        controls_ui,

        current_recording_name,
        current_frame,
        recording,
    }
}

/* -------------------------------------------
 * Main
 * ----------------------------------------
 */

fn view(app: &App, model: &Model, frame: Frame) {
    let draw = app.draw();
    let gdraw = draw
        .scale(SIZE as f32)
        .scale_y(-1.0)
        .x_y(COLS as f32 / -2.0 + 0.5, ROWS as f32 / -2.0 + 0.5);

    gdraw.background().color(WHITE);

    for stone in &model.gravel {
        let cdraw = gdraw.x_y(stone.x, stone.y);

        cdraw
            .rect()
            .no_fill()
            .stroke(stone.color)
            .stroke_weight(LINE_WIDTH)
            .w_h(1.0, 1.0)
            .x_y(stone.x_offset, stone.y_offset)
            .rotate(stone.rotation);
    }

    gdraw.to_frame(app, &frame).unwrap();
}

fn update(app: &App, model: &mut Model, update: Update) {
    update_controls_ui(app, model, update);

    for stone in &mut model.gravel {
        if stone.cycles == 0 {
            if random_f32() > model.motion {
                stone.x_velocity = 0.0;
                stone.y_velocity = 0.0;
                stone.rotation_velocity = 0.0;
                stone.cycles = random_range(50, 300);
            } else {
                let factor = stone.y / ROWS as f32;
                let displacement_factor = factor * model.displacement;
                let rotation_factor = factor * model.rotation;

                let new_x = displacement_factor * random_range(-0.5, 0.5);
                let new_y = displacement_factor * random_range(-0.5, 0.5);
                let new_rotation = rotation_factor * random_range(-PI / 4.0, PI / 4.0);
                let new_cycles = random_range(50, 300);

                stone.x_velocity = (new_x - stone.x_offset) / new_cycles as f32;
                stone.y_velocity = (new_y - stone.y_offset) / new_cycles as f32;
                stone.rotation_velocity = (new_rotation - stone.rotation) / new_cycles as f32;
                stone.cycles = new_cycles;

                // stone.start_cycles();
            }
        } else {
            stone.x_offset += stone.x_velocity;
            stone.y_offset += stone.y_velocity;
            stone.rotation += stone.rotation_velocity;
            stone.cycles -= 1;

            // stone.run_cycle();
        }
    }

    // If recording (and only ever other frame)
    if model.recording && app.elapsed_frames() % 2 == 0 {
        model.current_frame += 1;

        // Stop if too long
        if model.current_frame > 9999 {
            model.recording = false;
        } else {
            if let Some(main_window) = app.window(model.main_window) {
                let filename = format!(
                    "{ASSETS_DIR}/recordings/{}/{:>04}.png",
                    model.current_recording_name, model.current_frame
                );

                main_window.capture_frame(filename);
            }
        }
    }
}

fn key_pressed(app: &App, model: &mut Model, key: Key) {
    match key {
        Key::S => {
            if let Some(window) = app.window(model.main_window) {
                let app_name = app.exe_name().unwrap().to_string();
                let Model {
                    displacement,
                    rotation,
                    ..
                } = model;
                let image_path =
                    format!("{ASSETS_DIR}/snapshots/{app_name}-d{displacement}-r{rotation}.png");
                window.capture_frame(image_path);
            }
        }
        Key::R => {
            if model.recording {
                model.recording = false;
            } else {
                model.current_recording_name = app.time.to_string();

                let path = format!("{ASSETS_DIR}/recordings/{}", model.current_recording_name);
                fs::create_dir_all(&path).unwrap_or_else(|error| {
                    if error.kind() != ErrorKind::AlreadyExists {
                        panic!("Problem creating directory {:?}", path);
                    }
                });

                model.current_frame = 0;
                model.recording = true;
            }
        }
        Key::Up => {
            model.displacement += 0.1;
        }
        Key::Down => {
            if model.displacement > 0.0 {
                model.displacement -= 0.1;
            }
        }
        Key::Right => {
            model.rotation += 0.1;
        }
        Key::Left => {
            if model.rotation > 0.0 {
                model.rotation -= 0.1;
            }
        }
        _other_key => {}
    }
}

struct Stone {
    x: f32,
    y: f32,
    x_offset: f32,
    y_offset: f32,
    rotation: f32,
    x_velocity: f32,
    y_velocity: f32,
    rotation_velocity: f32,
    cycles: u32,
    color: Rgb<u8>,
}

impl Stone {
    fn new(x: f32, y: f32) -> Self {
        let x_offset = 0.0;
        let y_offset = 0.0;
        let rotation = 0.0;
        let x_velocity = 0.0;
        let y_velocity = 0.0;
        let rotation_velocity = 0.0;
        let cycles = 0;
        let color = random_color();

        Stone {
            x,
            y,
            x_offset,
            y_offset,
            rotation,
            x_velocity,
            y_velocity,
            rotation_velocity,
            cycles,
            color,
        }
    }
}

fn random_color() -> Rgb<u8> {
    let colors = [
        BLUE,
        BLUEVIOLET,
        CHARTREUSE,
        CORAL,
        CRIMSON,
        DARKBLUE,
        DARKORANGE,
        DEEPPINK,
        FORESTGREEN,
        GOLD,
        ORANGERED,
        RED,
        SLATEBLUE,
        YELLOW,
    ];
    let random_index = random_range(0, colors.len() - 1);
    colors[random_index]
}

/* -------------------------------------------
 * Controls
 * ----------------------------------------
 */

fn controls_view(_app: &App, model: &Model, frame: Frame) {
    model.controls_ui.draw_to_frame(&frame).unwrap();
}

fn raw_controls_event(_app: &App, model: &mut Model, event: &nannou::winit::event::WindowEvent) {
    model.controls_ui.handle_raw_event(event);
}

fn update_controls_ui(app: &App, model: &mut Model, update: Update) {
    let ui = &mut model.controls_ui;
    ui.set_elapsed_time(update.since_start);
    let proxy = app.create_proxy();

    ui.do_frame_with_epi_frame(proxy, |ctx, _frame| {
        Area::new("controls").show(&ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("Displacement");

                ui.add(Slider::new(&mut model.displacement, (0.0)..=(5.0)));
            });

            ui.horizontal(|ui| {
                ui.label("Rotation");

                ui.add(Slider::new(&mut model.rotation, (0.0)..=(5.0)));
            });

            ui.horizontal(|ui| {
                ui.label("Motion");

                ui.add(Slider::new(&mut model.motion, (0.0)..=(1.0)));
            });
        });
    });
}
