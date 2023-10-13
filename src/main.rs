use nannou::{
    noise::{
        utils::{NoiseMap, NoiseMapBuilder, PlaneMapBuilder},
        Perlin, Seedable,
    },
    prelude::*,
};

const GRID_WIDTH: usize = 120;
const GRID_HEIGHT: usize = 80;

const CELL_WIDTH: usize = 10;
const CELL_HEIGHT: usize = 10;

const PADDING: usize = 0;

const PARTICLES: usize = 400;

const WINDOW_WIDTH: usize = GRID_WIDTH * CELL_WIDTH + PADDING * 2;
const WINDOW_HEIGHT: usize = GRID_HEIGHT * CELL_HEIGHT + PADDING * 2;

fn main() {
    nannou::app(model)
        .update(update)
        .simple_window(view)
        .size(WINDOW_WIDTH as u32, WINDOW_HEIGHT as u32)
        .run();
}

struct Model {
    map: NoiseMap,
    bounds: Rect,
    particles: Vec<Particle>,
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
}

fn model(app: &App) -> Model {
    let seed = random_range(0, u32::MAX);
    let noise = Perlin::new().set_seed(seed);

    let bounds = app
        .window_rect()
        .pad_top(PADDING as f32)
        .pad_bottom(PADDING as f32)
        .pad_left(PADDING as f32)
        .pad_right(PADDING as f32);

    let map = PlaneMapBuilder::new(&noise)
        .set_size(bounds.w() as usize, bounds.h() as usize)
        .set_x_bounds(-2., 2.)
        .set_y_bounds(-2., 2.)
        .set_is_seamless(true)
        .build();

    let particles = vec![0; PARTICLES];
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

    Model {
        map,
        bounds,
        particles,
    }
}

fn update(_app: &App, model: &mut Model, update: Update) {
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

fn view(app: &App, model: &Model, frame: Frame) {
    let draw = app.draw();

    // Reset background color
    // draw.background().color(BLANCHEDALMOND);

    // Render bounds
    // draw.rect()
    //     .xy(model.bounds.xy())
    //     .wh(model.bounds.wh())
    //     .color(PLUM);

    let origin = model.bounds.bottom_left();

    // Render flow field arrows
    for x in -1..GRID_WIDTH as i32 + 1 {
        for y in -1..GRID_HEIGHT as i32 + 1 {
            let cell = Vec2::new(x as f32 * CELL_WIDTH as f32, y as f32 * CELL_HEIGHT as f32);
            let direction = model.sample_direction(cell.x, cell.y);

            let start = origin + cell;
            let end = start + direction * 8.0;

            draw.arrow().start(start).end(end).color(DARKGRAY);
        }
    }

    // Render all particles
    // for particle in model.particles.iter() {
    //     draw.ellipse()
    //         .xy(origin + particle.position)
    //         .w_h(4.0, 4.0)
    //         .color(PLUM);
    // }

    draw.to_frame(app, &frame).unwrap();
}
