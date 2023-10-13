use nannou::{
    noise::{
        utils::{NoiseMap, NoiseMapBuilder, PlaneMapBuilder},
        Perlin, Seedable,
    },
    prelude::*,
};

use nannou_egui::{egui, Egui};

const GRID_WIDTH: usize = 120;
const GRID_HEIGHT: usize = 80;

const CELL_WIDTH: usize = 10;
const CELL_HEIGHT: usize = 10;

const PADDING: usize = 0;

const WINDOW_WIDTH: usize = GRID_WIDTH * CELL_WIDTH + PADDING * 2;
const WINDOW_HEIGHT: usize = GRID_HEIGHT * CELL_HEIGHT + PADDING * 2;

fn main() {
    nannou::app(model).update(update).run();
}

struct Model {
    egui: Egui,
    settings: Settings,
    map: NoiseMap,
    bounds: Rect,
    particles: Vec<Particle>,
}

struct Settings {
    noise_seed: u32,
    draw_background: bool,
    draw_particles: bool,
    draw_flowfield: bool,
    particle_count: usize,
    particle_size: f32,
}

struct Particle {
    position: Vec2,
    acceleration: Vec2,
}

impl Model {
    fn sample_direction(&self, x: f32, y: f32) -> Vec2 {
        let angle = self.map.get_value(x as usize, y as usize) as f32;
        let angle = angle * 2.0 * PI;

        Vec2::X.rotate(angle)
    }

    fn generate_map(seed: u32, bounds: &Rect) -> NoiseMap {
        let noise = Perlin::new().set_seed(seed);
        let map = PlaneMapBuilder::new(&noise)
            .set_size(bounds.w() as usize, bounds.h() as usize)
            .set_x_bounds(-2., 2.)
            .set_y_bounds(-2., 2.)
            .set_is_seamless(true)
            .build();

        map
    }

    fn generate_particles(count: usize, bounds: &Rect) -> Vec<Particle> {
        let particles = vec![0; count];
        let particles = particles
            .iter()
            .map(|_| {
                let origin = bounds.xy();

                let x = random_range(0, bounds.w() as i32) as f32;
                let y = random_range(0, bounds.h() as i32) as f32;

                let x = Vec2::X * x;
                let y = Vec2::Y * y;

                Particle {
                    position: origin + x + y,
                    acceleration: Vec2::ZERO,
                }
            })
            .collect();

        particles
    }
}

fn model(app: &App) -> Model {
    let window_id = app
        .new_window()
        .view(view)
        .size(WINDOW_WIDTH as u32, WINDOW_HEIGHT as u32)
        .raw_event(raw_window_event)
        .build()
        .unwrap();

    let window = app.window(window_id).unwrap();
    let egui = Egui::from_window(&window);

    let settings = Settings {
        noise_seed: random_range(0, u32::MAX),
        draw_background: true,
        draw_particles: true,
        draw_flowfield: false,
        particle_count: 400,
        particle_size: 1.0,
    };

    let bounds = app
        .window_rect()
        .pad_top(PADDING as f32)
        .pad_bottom(PADDING as f32)
        .pad_left(PADDING as f32)
        .pad_right(PADDING as f32);

    let map = Model::generate_map(settings.noise_seed, &bounds);
    let particles = Model::generate_particles(settings.particle_count, &bounds);

    Model {
        egui,
        settings,
        map,
        bounds,
        particles,
    }
}

fn update(_app: &App, model: &mut Model, update: Update) {
    // Update UI
    let frame = model.egui.begin_frame();
    let ctx = frame.context();

    egui::Window::new("Settings").show(&ctx, |ui| {
        ui.label("Noise:");
        if ui
            .add(egui::Slider::new(
                &mut model.settings.noise_seed,
                0..=u32::MAX,
            ))
            .changed()
        {
            model.map = Model::generate_map(model.settings.noise_seed, &model.bounds);
        }

        ui.label("Particles:");
        if ui
            .add(
                egui::Slider::new(&mut model.settings.particle_count, 0..=1000)
                    .integer()
                    .text("count"),
            )
            .changed()
        {
            model.particles =
                Model::generate_particles(model.settings.particle_count, &model.bounds);
        }

        ui.add(
            egui::Slider::new(&mut model.settings.particle_size, 0.1..=100.0)
                .text("size")
                .logarithmic(true),
        );

        ui.label("Rendering:");
        ui.add(egui::Checkbox::new(
            &mut model.settings.draw_background,
            "background",
        ));
        ui.add(egui::Checkbox::new(
            &mut model.settings.draw_particles,
            "particles",
        ));
        ui.add(egui::Checkbox::new(
            &mut model.settings.draw_flowfield,
            "flowfield",
        ));
    });

    frame.end();

    // Update particles
    let particles: Vec<Particle> = model
        .particles
        .iter()
        .map(|particle| {
            let deltatime = update.since_last.as_secs_f32() * 100.;

            let direction = model.sample_direction(particle.position.x, particle.position.y);
            let direction = direction.clamp_length_max(0.04); // Limit current flow field direction influence

            let acceleration = particle.acceleration + direction;
            let acceleration = acceleration.clamp_length_max(1.); // Limit max acceleration

            let mut particle = Particle {
                position: particle.position + acceleration * deltatime,
                acceleration,
            };

            let (width, height) = model.bounds.w_h();

            if particle.position.x < 0. {
                particle.position.x += width;
            } else if particle.position.x > width {
                particle.position.x -= width;
            }

            if particle.position.y < 0. {
                particle.position.y += height;
            } else if particle.position.y > height {
                particle.position.y -= height;
            }

            particle
        })
        .collect();

    model.particles = particles;
}

fn raw_window_event(_app: &App, model: &mut Model, event: &nannou::winit::event::WindowEvent) {
    model.egui.handle_raw_event(event);
}

fn view(app: &App, model: &Model, frame: Frame) {
    let draw = app.draw();
    let origin = model.bounds.bottom_left();

    if model.settings.draw_background {
        draw.background().color(BLACK);
    }

    if model.settings.draw_flowfield {
        for x in -1..GRID_WIDTH as i32 + 1 {
            for y in -1..GRID_HEIGHT as i32 + 1 {
                let cell = Vec2::new(x as f32 * CELL_WIDTH as f32, y as f32 * CELL_HEIGHT as f32);
                let direction = model.sample_direction(cell.x, cell.y);

                let start = origin + cell;
                let end = start + direction * 8.0;

                draw.arrow()
                    .start(start)
                    .end(end)
                    .color(DARKGRAY)
                    .stroke_weight(0.5);
            }
        }
    }

    if model.settings.draw_particles {
        for particle in model.particles.iter() {
            draw.ellipse()
                .xy(origin + particle.position)
                .w_h(model.settings.particle_size, model.settings.particle_size)
                .color(PLUM);
        }
    }

    draw.to_frame(app, &frame).unwrap();
    model.egui.draw_to_frame(&frame).unwrap();
}
