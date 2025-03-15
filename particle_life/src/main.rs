use cgmath::{Point2, Vector2, InnerSpace};
use minifb::{Key, Window, WindowOptions};
use rand::Rng;
use rayon::prelude::*;

const WIDTH: usize = 800;
const HEIGHT: usize = 600;
const NUM_PARTICLES: usize = 5000;
const NUM_PARTICLE_TYPES: usize = 5;
const INTERACTION_RADIUS: f32 = 50.0;
const DT: f32 = 0.1;
const FORCE_MULTIPLIER: f32 = 0.5;

struct Particle {
    position: [f32; 2],
    velocity: [f32; 2],
    particle_type: u32,
}

struct World {
    particles: Vec<Particle>,
    attraction_matrix: [[f32; NUM_PARTICLE_TYPES]; NUM_PARTICLE_TYPES],
    buffer: Vec<u32>,
}

impl World {
    fn new() -> Self {
        let mut rng = rand::thread_rng();
        let mut particles = Vec::with_capacity(NUM_PARTICLES);
        
        for _ in 0..NUM_PARTICLES {
            particles.push(Particle {
                position: [
                    rng.gen_range(0.0..WIDTH as f32),
                    rng.gen_range(0.0..HEIGHT as f32),
                ],
                velocity: [0.0, 0.0],
                particle_type: rng.gen_range(0..NUM_PARTICLE_TYPES as u32),
            });
        }

        let mut attraction_matrix = [[0.0; NUM_PARTICLE_TYPES]; NUM_PARTICLE_TYPES];
        for i in 0..NUM_PARTICLE_TYPES {
            for j in 0..NUM_PARTICLE_TYPES {
                attraction_matrix[i][j] = rng.gen_range(-1.0..1.0);
            }
        }

        Self {
            particles,
            attraction_matrix,
            buffer: vec![0; WIDTH * HEIGHT],
        }
    }

    fn update(&mut self) {
        // Create a temporary copy of positions for reading
        let positions: Vec<_> = self.particles
            .iter()
            .map(|p| (Point2::new(p.position[0], p.position[1]), p.particle_type))
            .collect();

        self.particles.par_chunks_mut(NUM_PARTICLES / rayon::current_num_threads())
            .for_each(|chunk| {
                for particle in chunk {
                    let mut force = Vector2::new(0.0, 0.0);
                    let pos = Point2::new(particle.position[0], particle.position[1]);

                    for (other_pos, other_type) in &positions {
                        let delta = *other_pos - pos;
                        let distance = delta.magnitude();

                        if distance > 0.0 && distance < INTERACTION_RADIUS {
                            let strength = self.attraction_matrix[particle.particle_type as usize][*other_type as usize];
                            force += delta.normalize() * strength * (1.0 - distance / INTERACTION_RADIUS);
                        }
                    }

                    let velocity = Vector2::new(particle.velocity[0], particle.velocity[1]);
                    let new_velocity = velocity + force * FORCE_MULTIPLIER * DT;
                    particle.velocity = [new_velocity.x, new_velocity.y];
                    
                    let new_pos = pos + new_velocity * DT;
                    particle.position[0] = new_pos.x.rem_euclid(WIDTH as f32);
                    particle.position[1] = new_pos.y.rem_euclid(HEIGHT as f32);
                }
            });
    }

    fn draw(&mut self) -> &[u32] {
        // Clear the buffer
        self.buffer.fill(0);

        // Draw particles
        for particle in &self.particles {
            let x = particle.position[0] as usize;
            let y = particle.position[1] as usize;
            
            if x < WIDTH && y < HEIGHT {
                let pixel_index = y * WIDTH + x;
                
                // Assign different colors based on particle type
                let color = match particle.particle_type {
                    0 => 0xFF0000,    // Red
                    1 => 0x00FF00,    // Green
                    2 => 0x0000FF,    // Blue
                    3 => 0xFFFF00,    // Yellow
                    _ => 0xFF00FF,    // Purple
                };

                if pixel_index < self.buffer.len() {
                    self.buffer[pixel_index] = color;
                }
            }
        }

        &self.buffer
    }
}

fn main() {
    let mut window = Window::new(
        "Particle Life",
        WIDTH,
        HEIGHT,
        WindowOptions::default(),
    ).unwrap();

    window.limit_update_rate(Some(std::time::Duration::from_micros(16600)));

    let mut world = World::new();

    while window.is_open() && !window.is_key_down(Key::Escape) {
        world.update();
        window.update_with_buffer(world.draw(), WIDTH, HEIGHT).unwrap();
    }
}
