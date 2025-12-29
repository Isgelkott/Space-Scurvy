use std::{collections::HashMap, f32::consts::PI, sync::LazyLock};

use macroquad::prelude::*;

use crate::{
    assets::ASSETS,
    level::{Layer, Level, MAP_SCALE_FACTOR, TILE_SIZE},
    player::{self, Player},
    utils::{Animation, AnimationMethods},
};
pub static ENEMY_IDS: LazyLock<HashMap<u8, PresetEnemies>> = LazyLock::new(|| {
    HashMap::from([
        (140, PresetEnemies::Jetpacker),
        (141, PresetEnemies::SpikeBall),
        (142, PresetEnemies::MachineGunner),
    ])
});
#[derive(Clone, Copy)]
pub enum PresetEnemies {
    Jetpacker,
    SpikeBall,
    MachineGunner,
}
impl PresetEnemies {
    pub fn spawn(&self, pos: Vec2, map: &Level) -> Box<dyn Enemy> {
        match &self {
            Self::Jetpacker => Jetpacker::spawn(pos, map),
            Self::SpikeBall => SpikeBall::spawn(pos, map),
            Self::MachineGunner => MachineGunner::spawn(pos, map),
            _ => todo!(),
        }
    }
}
fn check_player_collision(pos: Vec2, size: Vec2, player: &Player) -> bool {
    if pos.x <= player.pos.x + player.size.x
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
    fn on_impact(&mut self, player: &mut Player) -> bool;
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
    fn on_impact(&mut self, player: &mut Player) -> bool {
        if check_player_collision(self.pos, self.size, player) {
            player.hp = player.hp.saturating_sub(20);
            true
        } else {
            false
        }
    }
    fn update(&mut self, player: &mut Player, map: &Level)
    where
        Self: Sized,
    {
        self.animation.play(self.pos, None);
        self.pos += self.velocity * get_frame_time();
        if check_player_collision(self.pos, self.size, player) {
            player.hp = player.hp.saturating_sub(20);
        }
    }
}
struct StandardProjectile {
    pos: Vec2,
    size: Vec2,
    direction: Vec2,
    speed: f32,
    animation: &'static Animation,
}
impl StandardProjectile {
    fn new(
        pos: Vec2,
        size: Vec2,
        direction: Vec2,
        speed: f32,
        animation: &'static Animation,
    ) -> Self {
        Self {
            size,

            pos,
            direction,
            speed,
            animation,
        }
    }
}
impl Projectile for StandardProjectile {
    fn update(&mut self, player: &mut Player, map: &Level) {
        self.pos += self.direction.normalize_or_zero() * self.speed * get_frame_time();
        self.animation.play(
            self.pos,
            Some(DrawTextureParams {
                rotation: self.direction.to_angle(),
                ..Default::default()
            }),
        );
        if check_player_collision(self.pos, self.size, player) {
            player.hp = player.hp.saturating_sub(20);
        }
    }
    fn on_impact(&mut self, player: &mut Player) -> bool {
        if check_player_collision(self.pos, self.size, player) {
            player.hp = player.hp.saturating_sub(20);
            return true;
        } else {
            false
        }
    }
}
pub trait Enemy {
    fn spawn(pos: Vec2, map: &Level) -> Box<dyn Enemy>
    where
        Self: Sized;

    fn update(&mut self, player: &Player, map: &Level, projectiles: &mut Vec<Box<dyn Projectile>>);
}
struct MachineGunner {
    pos: Vec2,
    clock: f32,
    animation: &'static Animation,
    size: Vec2,
}
impl Enemy for MachineGunner {
    fn spawn(pos: Vec2, map: &Level) -> Box<dyn Enemy>
    where
        Self: Sized,
    {
        Box::new(Self {
            size: ASSETS.machine_gunner.get_size(),
            pos,
            clock: 0.0,
            animation: &ASSETS.machine_gunner,
        })
    }
    fn update(&mut self, player: &Player, map: &Level, projectiles: &mut Vec<Box<dyn Projectile>>) {
        let flipped: bool = if player.pos.x > self.pos.x {
            true
        } else {
            false
        };
        if self.clock >= 0.4 {
            projectiles.push(Box::new(StandardProjectile::new(
                self.pos
                    + if flipped {
                        vec2(8.5, 15.0)
                    } else {
                        vec2(-8.5, 15.0)
                    },
                ASSETS.laser.get_size(),
                ((player.pos + player.size / 2.0) - (self.pos + self.size.y / 2.0))
                    .normalize_or_zero(),
                40.0,
                &ASSETS.laser,
            )));
            self.clock = 0.0;
        } else {
            self.clock += get_frame_time();
        }
        self.animation.play(
            self.pos,
            Some(DrawTextureParams {
                flip_x: flipped,
                ..Default::default()
            }),
        );
    }
}
struct SpikeBall {
    pos: Vec2,
}
impl Enemy for SpikeBall {
    fn spawn(pos: Vec2, map: &Level) -> Box<dyn Enemy>
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
struct Jetpacker {
    origin: Vec2,
    clock: f32,
    attacked: bool,
    flipped: bool,
    behavior_curve: [(f32, f32, &'static Animation, bool); 6],
}

impl Enemy for Jetpacker {
    fn spawn(pos: Vec2, map: &Level) -> Box<dyn Enemy>
    where
        Self: Sized,
    {
        let map_pos = pos / (TILE_SIZE * MAP_SCALE_FACTOR);
        let mut tile = (map_pos.y as usize - 1) * map.width as usize + map_pos.x as usize;
        let mut fly_height = 0.0;
        while map.tiles[tile].data.iter().any(|f| f.0 == Layer::Path) {
            println!("path above at tile {}", tile);
            tile = tile - map.width as usize;
            fly_height -= TILE_SIZE * MAP_SCALE_FACTOR;
        }
        let flight_speed = -50.0;
        let flight_time = (fly_height / flight_speed).abs();
        dbg!(flight_time);
        let curve: [(f32, f32, &Animation, bool); 6] = [
            (0.0, 0.0, &ASSETS.jetpacker.fly, false),
            (flight_time, fly_height, &ASSETS.jetpacker.fly, false),
            (flight_time + 2.5, fly_height, &ASSETS.jetpacker.fly, true),
            (flight_time + 4.0, fly_height, &ASSETS.jetpacker.fly, false),
            (flight_time + 5.0, 0.0, &ASSETS.jetpacker.idle, false),
            (flight_time + 7.0, 0.0, &ASSETS.jetpacker.idle, false),
        ];
        Box::new(Self {
            behavior_curve: curve,
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

        if self.clock + get_frame_time() > self.behavior_curve.last().unwrap().0 {
            self.clock = 0.0;
            self.attacked = false;
        } else {
            self.clock += get_frame_time();
        }
        let mut last = self.behavior_curve.last().unwrap();
        let time = self.clock;
        for p in self.behavior_curve.iter().rev() {
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
                        pos + if self.flipped {
                            vec2(10.0, 0.0)
                        } else {
                            vec2(-10.0, 0.0)
                        } + vec2(0.0, 5.0),
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
    projectiles.retain_mut(|f| {
        if f.on_impact(player) {
            return false;
        }
        f.update(player, level);
        return true;
    });
}
