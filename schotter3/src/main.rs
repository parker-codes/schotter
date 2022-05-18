use nannou::prelude::*;
use nannou::rand::rngs::StdRng;
use nannou::rand::{Rng, SeedableRng};
use nannou_egui::egui::{Area, Slider, TextEdit};
use nannou_egui::Egui;

const ROWS: u32 = 22;
const COLS: u32 = 12;
const SIZE: u32 = 30;
const LINE_WIDTH: f32 = 0.06;
const MARGIN: u32 = 35;
const WIDTH: u32 = COLS * SIZE + 2 * MARGIN;
const HEIGHT: u32 = ROWS * SIZE + 2 * MARGIN;

fn main() {
    nannou::app(model)
        .update(update)
        .loop_mode(LoopMode::wait())
        .run();
}

struct Model {
    main_window: WindowId,

    random_seed: u64,
    displacement: f32,
    rotation: f32,
    color: Rgb<f32>,
    gravel: Vec<Stone>,

    controls_ui: Egui,
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

    let random_seed = gen_random_seed();
    let displacement = 1.0;
    let rotation = 1.0;
    let color = Rgb::new(0.0, 0.0, 0.0);

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

    Model {
        main_window,
        random_seed,
        displacement,
        rotation,
        color,
        gravel,

        controls_ui,
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
            .stroke(model.color)
            .stroke_weight(LINE_WIDTH)
            .w_h(1.0, 1.0)
            .x_y(stone.x_offset, stone.y_offset)
            .rotate(stone.rotation);
    }

    gdraw.to_frame(app, &frame).unwrap();
}

fn update(app: &App, model: &mut Model, update: Update) {
    update_controls_ui(app, model, update);

    let mut rng = StdRng::seed_from_u64(model.random_seed);

    for stone in &mut model.gravel {
        let factor = stone.y / ROWS as f32;

        let displacement_factor = factor * model.displacement;
        let rotation_factor = factor * model.rotation;

        stone.x_offset = displacement_factor * rng.gen_range(-0.5..0.5);
        stone.y_offset = displacement_factor * rng.gen_range(-0.5..0.5);
        stone.rotation = rotation_factor * rng.gen_range(-PI / 4.0..PI / 4.0);
    }
}

fn key_pressed(app: &App, model: &mut Model, key: Key) {
    match key {
        Key::R => {
            model.random_seed = gen_random_seed();
        }
        Key::S => {
            if let Some(window) = app.window(model.main_window) {
                let app_name = app.exe_name().unwrap().to_string();
                let Model {
                    random_seed,
                    displacement,
                    rotation,
                    ..
                } = model;
                let image_path =
                    format!("snapshots/{app_name}-s{random_seed}-d{displacement}-r{rotation}.png");
                window.capture_frame(image_path);
            }
        }
        Key::C => {
            model.color = gen_random_color();
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
}

impl Stone {
    fn new(x: f32, y: f32) -> Self {
        let x_offset = 0.0;
        let y_offset = 0.0;
        let rotation = 0.0;

        Stone {
            x,
            y,
            x_offset,
            y_offset,
            rotation,
        }
    }
}

fn gen_random_seed() -> u64 {
    random_range(0, 1_000_000)
}

fn gen_random_color() -> Rgb {
    let r = random_range(0, 255) as f32 / 255.0;
    let g = random_range(0, 255) as f32 / 255.0;
    let b = random_range(0, 255) as f32 / 255.0;

    Rgb::new(r, g, b)
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
            ui.add(Slider::new(&mut model.displacement, (0.0)..=(5.0)).text("Displacement"));

            ui.add(Slider::new(&mut model.rotation, (0.0)..=(5.0)).text("Rotation"));

            ui.horizontal(|ui| {
                ui.label("Seed");

                let mut seed_string = model.random_seed.to_string();
                let seed_string_input = ui.add(
                    TextEdit::singleline(&mut seed_string)
                        .desired_width(100.0)
                        .cursor_at_end(true),
                );
                if seed_string_input.changed() {
                    let seed = seed_string.parse::<u64>().unwrap_or(0);
                    model.random_seed = seed;
                }

                if ui.button("Randomize").clicked() {
                    model.random_seed = gen_random_seed();
                }
            });
        });
    });
}
