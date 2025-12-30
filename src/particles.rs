use macroquad::{math::Vec2, prelude::animation, time::get_frame_time};

use crate::utils::{Animation, AnimationMethods};
pub trait Particle {
    fn update(&mut self) -> bool;
}
pub struct StandardParticle {
    pos: Vec2,
    clock: f32,
    lifetime: f32,
    animation: &'static Animation,
    size: Vec2,
}
impl StandardParticle {
    pub fn from_animation(pos: Vec2, animation: &'static Animation) -> Self {
        Self {
            pos,
            clock: 0.0,
            lifetime: animation.1 as f32 / 1000.0,
            animation,
            size: animation.get_size(),
        }
    }
    pub fn new(
        pos: Vec2,
        clock: f32,
        lifetime: f32,
        animation: &'static Animation,
        size: Vec2,
    ) -> Self {
        Self {
            pos,
            clock,
            lifetime,
            animation,
            size,
        }
    }
}
impl Particle for StandardParticle {
    fn update(&mut self) -> bool {
        if self.clock >= self.lifetime {
            true
        } else {
            self.animation
                .play_with_clock(self.pos - self.size / 2.0, &mut self.clock, None);
            false
        }
    }
}
pub fn update_particles(particles: &mut Vec<Box<dyn Particle>>) {
    particles.retain_mut(|f| {
        let is_dead = f.update();
        !is_dead
    });
}
