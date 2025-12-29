use std::{collections::HashMap, sync::LazyLock};

use macroquad::prelude::*;

use crate::{
    assets::ASSETS,
    level::Level,
    player::{self, Player},
    utils::{Animation, Play},
};
pub static ENEMY_IDS: LazyLock<HashMap<u8, PresetEnemies>> = LazyLock::new(|| {
    HashMap::from([
        (140, PresetEnemies::Jetpacker),
        (141, PresetEnemies::SpikeBall),
    ])
});
#[derive(Clone, Copy)]
pub enum PresetEnemies {
    Jetpacker,
    SpikeBall,
}
impl PresetEnemies {
    pub fn spawn(&self, pos: Vec2) -> Box<dyn Enemy> {
        match &self {
            Self::Jetpacker => Jetpacker::spawn(pos),
            Self::SpikeBall => SpikeBall::spawn(pos),
            _ => todo!(),
        }
    }
}
fn check_player_collision(pos: Vec2, size: Vec2, player: &Player) -> bool {
    if pos.x + size.x <= player.pos.x + player.size.x
        && pos.x >= player.pos.x
        && pos.y >= player.pos.y
        && player.size.y + player.pos.y >= pos.y + size.y
    {
        true
    } else {
        false
    }
}
pub trait Projectile {
    fn update(&mut self, player: &mut Player, map: &Level);
}
pub struct EnergyBall {
    animation: &'static Animation,
    velocity: Vec2,
    pos: Vec2,
    size: Vec2,
}
impl EnergyBall {
    fn new(pos: Vec2, x_flipped: bool) -> Self {
        Self {
            animation: &ASSETS.energy_ball,
            velocity: vec2(40.0, 0.0) * if x_flipped { 1.0 } else { -1.0 },
            pos,
            size: vec2(16.0, 16.0),
        }
    }
}
impl Projectile for EnergyBall {
    fn update(&mut self, player: &mut Player, map: &Level)
    where
        Self: Sized,
    {
        self.animation.play(self.pos, None);
        self.pos += self.velocity * get_frame_time();
        if check_player_collision(self.pos, self.size, player) {
            player.hp.saturating_sub(20);
        }
    }
}
struct SpikeBall {
    pos: Vec2,
}
impl Enemy for SpikeBall {
    fn spawn(pos: Vec2) -> Box<dyn Enemy>
    where
        Self: Sized,
    {
        Box::new(Self { pos })
    }
    fn update(&mut self, player: &Player, map: &Level, projectiles: &mut Vec<Box<dyn Projectile>>) {
        self.pos += (player.pos - self.pos).normalize() * 10.0 * get_frame_time();
        let _ = &ASSETS.spike_ball.play(self.pos, None);
    }
}
pub trait Enemy {
    fn spawn(pos: Vec2) -> Box<dyn Enemy>
    where
        Self: Sized;

    fn update(&mut self, player: &Player, map: &Level, projectiles: &mut Vec<Box<dyn Projectile>>);
}
struct Jetpacker {
    origin: Vec2,
    clock: f32,
    attacked: bool,
    flipped: bool,
}

impl Enemy for Jetpacker {
    fn spawn(pos: Vec2) -> Box<dyn Enemy>
    where
        Self: Sized,
    {
        Box::new(Self {
            flipped: false,
            attacked: false,
            clock: 0.0,
            origin: pos,
        })
    }
    fn update(&mut self, player: &Player, map: &Level, projectiles: &mut Vec<Box<dyn Projectile>>) {
        self.flipped = if player.pos.x > self.origin.x {
            true
        } else {
            false
        };

        let curve = [
            (0.0, 0.0, &ASSETS.jetpacker.fly, false),
            (1.0, -50.0, &ASSETS.jetpacker.fly, false),
            (2.5, -50.0, &ASSETS.jetpacker.fly, true),
            (4.0, -50.0, &ASSETS.jetpacker.fly, false),
            (5.0, 0.0, &ASSETS.jetpacker.idle, false),
            (7.0, 0.0, &ASSETS.jetpacker.idle, false),
        ];
        if self.clock + get_frame_time() > curve.last().unwrap().0 {
            self.clock = 0.0;
            self.attacked = false;
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
                    self.origin.y + if !k.is_infinite() { k } else { 0.0 } * (time - p.0) + p.1,
                );
                p.2.play(
                    pos,
                    Some(DrawTextureParams {
                        flip_x: self.flipped,
                        ..Default::default()
                    }),
                );
                if p.3 && !self.attacked {
                    self.attacked = true;
                    projectiles.push(Box::new(EnergyBall::new(
                        pos + if self.flipped { 10.0 } else { -10.0 } + vec2(0.0, 10.0),
                        self.flipped,
                    )));
                }
                break;
            } else {
                last = p
            }
        }
    }
}
pub fn update_enemies(
    player: &Player,
    enemies: &mut Vec<Box<dyn Enemy>>,
    map: &Level,
    projectiles: &mut Vec<Box<dyn Projectile>>,
) {
    for enemy in enemies.iter_mut() {
        enemy.update(player, map, projectiles);
    }
}
pub fn update_projectiles(
    player: &mut Player,
    level: &Level,
    projectiles: &mut Vec<Box<dyn Projectile>>,
) {
    for projectile in projectiles.iter_mut() {
        projectile.update(player, level);
    }
}
