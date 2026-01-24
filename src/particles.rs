use macroquad::{color::Color, math::Vec2, prelude::*, rand::gen_range};

use crate::{
    assets::ASSETS,
    level::{MAP_SCALE_FACTOR, TILE_SIZE, Tile},
    utils::*,
};
pub enum Lifetime {
    ByDistance(f32),
    ByTime(f32),
}
pub enum Particles {
    Explosion,
    EnergyBallShatter,
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
    pub fn from(particle: Particles, pos: Vec2) -> Self {
        match &particle {
            Particles::Explosion => {
                let animation = ASSETS.rocket.get("explode");
                Self::new(
                    Box::new(|f| ASSETS.rocket.get("explode").play(f, None)),
                    Lifetime::ByTime((animation.get_duration())),
                    None,
                    pos,
                )
            }
            Particles::EnergyBallShatter => Particle::new(
                Box::new(|f| {
                    ASSETS.energy_ball_shatter.play(f, None);
                }),
                crate::particles::Lifetime::ByTime(ASSETS.energy_ball_shatter.1 as f32 / 1000.0),
                None,
                pos,
            ),
        }
    }
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
    pub fn update(&mut self, frame_time: f32) {
        self.clock += frame_time;
        let mut pos = self.origin;
        if let Some(behavior_fn) = &self.behavior {
            pos.y = behavior_fn(self.clock) + self.origin.y;
        }
        pos.x = self.origin.x + self.clock;
        (self.draw)(pos);
    }
    pub fn should_die(&self) -> bool {
        match self.lifetime {
            Lifetime::ByTime(time) => self.clock > time,
            _ => panic!(),
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
    pub fn update(&mut self, particles: &mut Vec<Particle>, frame_time: f32) {
        if self.clock > self.interval {
            self.clock = 0.0;
            particles.push(self.gen_particle());
        } else {
            self.clock += frame_time;
        }
    }
    fn gen_particle(&self) -> Particle {
        let rand_pos = gen_range(0.0, TILE_SIZE * MAP_SCALE_FACTOR);
        match self.particle_type {
            ParticleType::Acid => Particle::new(
                Box::new(|f| draw_circle(f.x, f.y, 1.5, Color::from_hex(0x5ac54f))),
                Lifetime::ByDistance(16.0),
                Some(Box::new(|f| -5.0 * f.powi(2) + 5.0 * f)),
                self.pos + vec2(rand_pos, 0.0),
            ),
        }
    }
}
pub fn update_particle_generators(
    tiles: &mut Vec<Tile>,
    particles: &mut Vec<Particle>,
    frame_time: f32,
) {
    for tile in tiles.iter_mut() {
        if let Some(gener) = &mut tile.particle_generator {
            gener.update(particles, frame_time);
        }
    }
}
pub fn update_particles(particles: &mut Vec<Particle>, frame_time: f32) {
    particles.retain_mut(|f| {
        f.update(frame_time);
        !f.should_die()
    });
}
