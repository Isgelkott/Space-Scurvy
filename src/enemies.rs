use std::collections::HashMap;

use macroquad::prelude::*;

use crate::{
    assets::ASSETS,
    level::Level,
    utils::{Animation, Play},
};
#[derive(Clone, Copy)]
pub enum PresetEnemies {
    Jetpacker,
    SpikeBall,
}
impl PresetEnemies {
    pub fn spawn(&self, pos: Vec2) -> Box<dyn Enemy> {
        match &self {
            Self::Jetpacker => Jetpacker::spawn(pos),
            _ => todo!(),
        }
    }
}
pub trait Enemy {
    fn spawn(pos: Vec2) -> Box<dyn Enemy>
    where
        Self: Sized;

    fn update(&mut self, map: &Level);
}
struct Jetpacker {
    origin: Vec2,
    clock: f32,
}

impl Enemy for Jetpacker {
    fn spawn(pos: Vec2) -> Box<dyn Enemy>
    where
        Self: Sized,
    {
        Box::new(Self {
            clock: 0.0,
            origin: pos,
        })
    }
    fn update(&mut self, map: &Level) {
        let curve = [
            (0.0, 0.0, &ASSETS.jetpacker.idle),
            (1.0, -100.0, &ASSETS.jetpacker.idle),
            (4.0, -100.0, &ASSETS.jetpacker.idle),
            (5.0, 0.0, &ASSETS.jetpacker.idle),
            (7.0, 0.0, &ASSETS.jetpacker.idle),
        ];
        if self.clock + get_frame_time() > curve.last().unwrap().0 {
            self.clock = 0.0;
        } else {
            self.clock += get_frame_time();
        }
        let mut last = curve.last().unwrap();
        let time = self.clock;
        for p in curve.iter().rev() {
            if time >= p.0 {
                let k = (last.1 - p.1) / (last.0 - p.0);
                let pos = vec2(
                    self.origin.x,
                    self.origin.y + if !k.is_nan() { k } else { 0.0 } * (time - p.0) + p.1,
                );
                p.2.play(pos, None);
                if is_key_down(KeyCode::F) {
                    dbg!((self.clock, self.origin, pos, p, last));
                }
                break;
            } else {
                last = p
            }
        }
    }
}
pub fn update_enemies(enemies: &mut Vec<Box<dyn Enemy>>, map: &Level) {
    for enemy in enemies.iter_mut() {
        enemy.update(map);
    }
}
