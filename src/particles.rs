use macroquad::{
    color::Color, math::Vec2, prelude::animation, prelude::*, rand::gen_range,
    shapes::draw_rectangle, time::get_frame_time,
};

use crate::{
    level::{MAP_SCALE_FACTOR, TILE_SIZE, Tile},
    utils::{Animation, AnimationMethods},
};
pub enum Lifetime {
    ByDistance(f32),
    ByTime(f32),
}
pub struct Particle {
    draw: Box<dyn Fn(Vec2) -> ()>,
    lifetime: Lifetime,
    clock: f32,
    behavior: Option<Box<dyn Fn(f32) -> f32>>,
    origin: Vec2,
    speed: f32,
}
impl Particle {
    pub fn new(
        draw: Box<dyn Fn(Vec2)>,
        lifetime: Lifetime,
        behavior: Option<Box<dyn Fn(f32) -> f32>>,
        origin: Vec2,
    ) -> Self {
        Self {
            draw,
            lifetime,
            clock: 0.0,
            behavior,
            origin,
            speed: 4.0,
        }
    }
    pub fn update(&mut self) {
        self.clock += get_frame_time();
        let mut pos = self.origin;
        if let Some(behavior_fn) = &self.behavior {
            pos.y = behavior_fn(self.clock) + self.origin.y;
        }
        pos.x = self.origin.x + self.clock;
        (self.draw)(pos);
    }
    pub fn should_die(&self) -> bool {
        match self.lifetime {
            Lifetime::ByDistance(distance) => {
                let behaviour = self.behavior.as_ref().unwrap();
                false //(behaviour(0.0) - (behaviour)(self.clock)).abs() > distance
            }
            Lifetime::ByTime(time) => self.clock > time,
        }
    }
}
pub enum ParticleType {
    Acid,
}
pub struct ParticleGenerator {
    pos: Vec2,
    clock: f32,
    interval: f32,
    particle_type: ParticleType,
}
impl ParticleGenerator {
    pub fn new(pos: Vec2, particle_type: ParticleType) -> Self {
        Self {
            pos,
            particle_type,
            clock: 0.0,
            interval: match particle_type {
                ParticleType::Acid => 0.78,
            },
        }
    }
    pub fn update(&mut self, particles: &mut Vec<Particle>) {
        if self.clock > self.interval {
            self.clock = 0.0;
            particles.push(self.gen_particle());
        } else {
            self.clock += get_frame_time();
        }
    }
    fn gen_particle(&self) -> Particle {
        let rand_pos = gen_range(0.0, TILE_SIZE * MAP_SCALE_FACTOR);
        match self.particle_type {
            ParticleType::Acid => Particle::new(
                Box::new(|f| draw_circle(f.x, f.y, 1.5, Color::from_hex(0x99e65f))),
                Lifetime::ByDistance(16.0),
                Some(Box::new(|f| -5.0 * f.powi(2) + 5.0 * f)),
                self.pos + vec2(rand_pos, 0.0),
            ),
        }
    }
}
pub fn update_particle_generators(tiles: &mut Vec<Tile>, particles: &mut Vec<Particle>) {
    for tile in tiles.iter_mut() {
        if let Some(gener) = &mut tile.particle_generator {
            gener.update(particles);
        }
    }
}
pub fn update_particles(particles: &mut Vec<Particle>) {
    particles.retain_mut(|f| {
        f.update();
        !f.should_die()
    });
}
