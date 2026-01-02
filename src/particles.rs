use macroquad::{math::Vec2, prelude::animation, time::get_frame_time};

use crate::utils::{Animation, AnimationMethods};
pub enum Lifetime {
    ByDistance(f32),
    ByTime(f32),
}
pub struct Particle {
    draw: Box<dyn Fn(Vec2) -> ()>,
    lifetime: Lifetime,
    clock: f32,
    behavior: Option<Box<dyn Fn(f32) -> f32>>,
    pos: Vec2,
}
impl Particle {
    pub fn new(
        draw: Box<dyn Fn(Vec2)>,
        lifetime: Lifetime,
        behavior: Option<Box<dyn Fn(f32) -> f32>>,
        pos: Vec2,
    ) -> Self {
        Self {
            draw,
            lifetime,
            clock: 0.0,
            behavior,
            pos,
        }
    }
    pub fn update(&mut self) {
        self.clock += get_frame_time();
        if let Some(behavior_fn) = &self.behavior {
            self.pos.y = behavior_fn(self.clock);
            self.pos.x = self.clock;
        }
        (self.draw)(self.pos);
    }
    pub fn should_die(&self) -> bool {
        match self.lifetime {
            Lifetime::ByDistance(distance) => {
                let behaviour = self.behavior.as_ref().unwrap();
                (behaviour(0.0) - (behaviour)(self.clock)).abs() > distance
            }
            Lifetime::ByTime(time) => self.clock > time,
        }
    }
}
pub fn update_particles(particles: &mut Vec<Particle>) {
    particles.retain_mut(|f| {
        f.update();
        !f.should_die()
    });
}
