use std::{collections::HashMap, sync::LazyLock};

use macroquad::prelude::*;

use crate::{
    assets::ASSETS,
    enemies,
    level::{Layer, Level, MAP_SCALE_FACTOR, TILE_SIZE},
    particles::Particle,
    player::{self, Player},
    utils::{Animation, AnimationMethods, BULLET_MATERIAL, check_collision},
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
        }
    }
}
fn check_player_collision(pos: Vec2, size: Vec2, player: &Player) -> bool {
    if pos.x <= player.pos.x + player.size.x
        && pos.x + size.x >= player.pos.x
        && pos.y >= player.pos.y
        && player.size.y + player.pos.y >= pos.y + size.y
    {
        true
    } else {
        false
    }
}
fn check_map_collision(pos: Vec2, size: Vec2, map: &Level) -> bool {
    let points = [
        vec2(pos.x, pos.y),
        vec2(pos.x + size.x, pos.y),
        vec2(pos.x, pos.y + size.y),
        vec2(pos.x + size.x, pos.y + size.y),
    ];
    return points.iter().any(|f| check_collision(*f, map));
}
pub fn check_collision_with_size(obj1: (Vec2, Vec2), obj2: (Vec2, Vec2)) -> bool {
    let points = [
        vec2(obj1.0.x, obj1.0.y),
        vec2(obj1.0.x + obj1.1.x, obj1.0.y),
        vec2(obj1.0.x, obj1.0.y + obj1.1.y),
        vec2(obj1.0.x + obj1.1.x, obj1.0.y + obj1.1.y),
    ];
    return points.iter().any(|f| {
        f.x >= obj2.0.x
            && f.x <= obj2.0.x + obj2.1.x
            && f.y >= obj2.0.y
            && f.y <= obj2.1.y + obj2.0.y
    });
}
fn check_projectile_collision<'a>(
    pos: Vec2,
    size: Vec2,
    map: &Level,
    player: &Player,
    enemies: &'a mut Vec<Box<dyn Enemy>>,
) -> Option<CollisionType<'a>> {
    if check_player_collision(pos, size, player) {
        return Some(CollisionType::Player);
    } else if check_map_collision(pos, size, map) {
        return Some(CollisionType::Map);
    } else {
        for enemy in enemies {
            let bounds = enemy.get_bounds();
            if check_collision_with_size((pos, size), bounds) {
                println!("wa");
                return Some(CollisionType::Enemy(enemy));
            }
        }

        return None;
    }
}

pub enum CollisionType<'a> {
    Player,
    Map,
    Enemy(&'a mut Box<dyn Enemy>),
}
impl<'a> CollisionType<'a> {
    fn is_same(&self, obj2: &Self) -> bool {
        std::mem::discriminant(self) == std::mem::discriminant(obj2)
    }
}
pub trait Projectile {
    fn update(&mut self, player: &mut Player, map: &Level);
    fn particle(&self) -> Option<Particle> {
        None
    }
    fn collision<'a>(
        &self,
        map: &Level,
        player: &Player,
        enemies: &'a mut Vec<Box<dyn Enemy>>,
    ) -> Option<CollisionType<'a>>;
    fn on_player_impact(&self, player: &mut Player) -> bool {
        true
    }
    fn on_enemy_impact(&self, enemy: &mut dyn Enemy) {}
}
pub struct Bullet {
    pos: Vec2,
    direction: Vec2,
    speed: f32,
    origin: Vec2,
}
impl Bullet {
    pub fn new(pos: Vec2, direction: Vec2) -> Self {
        Self {
            pos,
            direction,
            speed: 500.0,
            origin: pos,
        }
    }
}
impl Projectile for Bullet {
    fn on_enemy_impact(&self, enemy: &mut dyn Enemy) {
        enemy.on_hit_by_player();
    }
    fn on_player_impact(&self, _player: &mut Player) -> bool {
        false
    }
    fn collision<'a>(
        &self,
        map: &Level,
        player: &Player,
        enemies: &'a mut Vec<Box<dyn Enemy>>,
    ) -> Option<CollisionType<'a>> {
        let collision = check_projectile_collision(self.pos, Vec2::ZERO, map, player, enemies);
        return collision;
    }

    fn update(&mut self, _player: &mut Player, map: &Level) {
        self.pos += self.direction * self.speed * get_frame_time();
        gl_use_material(&BULLET_MATERIAL);
        BULLET_MATERIAL.set_uniform("alpha", 1.0 / (self.pos.x - self.origin.x).abs());
        draw_rectangle(
            self.origin.x,
            self.origin.y,
            self.pos.x - self.origin.x,
            2.0,
            BLACK,
        );
        gl_use_default_material();
    }
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
    fn particle(&self) -> Option<Particle> {
        Some(Particle::new(
            Box::new(|f| {
                &ASSETS.energy_ball_shatter.play(f, None);
            }),
            crate::particles::Lifetime::ByTime(ASSETS.energy_ball_shatter.1 as f32 / 1000.0),
            None,
            self.pos,
        ))
    }
    fn collision<'a>(
        &self,
        map: &Level,
        player: &Player,
        enemies: &'a mut Vec<Box<dyn Enemy + 'static>>,
    ) -> Option<CollisionType<'a>> {
        if check_player_collision(self.pos, self.size, player) {
            return Some(CollisionType::Player);
        } else if check_map_collision(self.pos, self.size, map) {
            return Some(CollisionType::Map);
        } else {
            return None;
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
    fn update(&mut self, _player: &mut Player, _map: &Level) {
        self.pos += self.direction.normalize_or_zero() * self.speed * get_frame_time();
        self.animation.play(
            self.pos,
            Some(DrawTextureParams {
                rotation: self.direction.to_angle(),
                ..Default::default()
            }),
        );
    }
    fn collision<'a>(
        &self,
        map: &Level,
        player: &Player,
        enemies: &'a mut Vec<Box<dyn Enemy>>,
    ) -> Option<CollisionType<'a>> {
        check_projectile_collision(self.pos, self.size, map, player, enemies)
    }
}

pub trait Enemy {
    fn get_bounds(&self) -> (Vec2, Vec2);
    fn spawn(pos: Vec2, map: &Level) -> Box<dyn Enemy>
    where
        Self: Sized;

    fn update(&mut self, player: &Player, map: &Level, projectiles: &mut Vec<Box<dyn Projectile>>);
    fn on_hit_by_player(&mut self) {}
}
struct MachineGunner {
    pos: Vec2,
    clock: f32,
    animation: &'static Animation,
    size: Vec2,
}
impl Enemy for MachineGunner {
    fn get_bounds(&self) -> (Vec2, Vec2) {
        (self.pos, self.size)
    }
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
                        vec2(-8.5 + self.size.x, 15.0)
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
    size: Vec2,
}
impl Enemy for SpikeBall {
    fn get_bounds(&self) -> (Vec2, Vec2) {
        (self.pos, self.size)
    }
    fn spawn(pos: Vec2, map: &Level) -> Box<dyn Enemy>
    where
        Self: Sized,
    {
        Box::new(Self {
            pos,
            size: ASSETS.spike_ball.get_size(),
        })
    }
    fn update(&mut self, player: &Player, map: &Level, projectiles: &mut Vec<Box<dyn Projectile>>) {
        self.pos += (player.pos - self.pos).normalize() * 10.0 * get_frame_time();
        let _ = &ASSETS.spike_ball.play(self.pos, None);
    }
}
#[derive(Debug)]
enum JetpackerState {
    Normal,
    Hit,
    Lie,
    Getup,
    Fall,
}
struct Jetpacker {
    size: Vec2,
    origin: Vec2,
    attacked: bool,
    flipped: bool,
    behavior_curve: [(f32, f32, &'static Animation, bool); 6],
    state: (JetpackerState, f32),
    fall_velocity: f32,
    pos: Vec2,
}

impl Enemy for Jetpacker {
    fn get_bounds(&self) -> (Vec2, Vec2) {
        (self.pos, self.size)
    }
    fn on_hit_by_player(&mut self) {
        self.state = (JetpackerState::Hit, 0.0);
    }
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
            (0.0, 0.0, &ASSETS.jetpacker.idle, false),
            (1.5, 0.0, &ASSETS.jetpacker.fly, false),
            (flight_time + 1.5, fly_height, &ASSETS.jetpacker.fly, false),
            (
                flight_time + 2.5 + 1.5,
                fly_height,
                &ASSETS.jetpacker.fly,
                true,
            ),
            (
                flight_time + 4.0 + 1.5,
                fly_height,
                &ASSETS.jetpacker.fly,
                false,
            ),
            (flight_time + 5.0 + 1.5, 0.0, &ASSETS.jetpacker.fly, false),
        ];
        Box::new(Self {
            state: (JetpackerState::Normal, 0.0),
            size: ASSETS.jetpacker.fly.get_size(),
            fall_velocity: 0.0,

            behavior_curve: curve,
            flipped: false,
            attacked: false,
            origin: pos,
            pos: pos,
        })
    }
    fn update(&mut self, player: &Player, map: &Level, projectiles: &mut Vec<Box<dyn Projectile>>) {
        self.state.1 += get_frame_time();
        if self.pos.y > self.origin.y {
            self.pos = self.origin;
            self.fall_velocity = 0.0;
            self.state = (JetpackerState::Lie, 0.0);
        }
        let params = DrawTextureParams {
            flip_x: self.flipped,
            ..Default::default()
        };
        match self.state.0 {
            JetpackerState::Fall => {
                self.fall_velocity += 2.0;
                ASSETS.jetpacker.fall.play(self.pos, Some(params.clone()));
            }
            JetpackerState::Getup => {
                ASSETS.jetpacker.getup.play_with_clock(
                    self.pos,
                    self.state.1,
                    Some(params.clone()),
                );
                if self.state.1 > ASSETS.jetpacker.getup.1 as f32 / 1000.0 {
                    self.state = (JetpackerState::Normal, 0.0)
                }
            }
            JetpackerState::Hit => {
                ASSETS
                    .jetpacker
                    .hit
                    .play_with_clock(self.pos, self.state.1, Some(params.clone()));
                if self.state.1 > ASSETS.jetpacker.hit.1 as f32 / 1000.0 {
                    self.state = (JetpackerState::Fall, 0.0)
                }
            }
            JetpackerState::Normal => {
                self.flipped = if player.pos.x > self.origin.x {
                    true
                } else {
                    false
                };

                if self.state.1 + get_frame_time() > self.behavior_curve.last().unwrap().0 {
                    self.state.1 = 0.0;
                    self.attacked = false;
                }
                let mut last = self.behavior_curve.last().unwrap();
                let time = self.state.1;
                for p in self.behavior_curve.iter().rev() {
                    if time >= p.0 {
                        let k = (last.1 - p.1) / (last.0 - p.0);
                        self.pos = vec2(
                            self.origin.x,
                            self.origin.y
                                + if !k.is_infinite() { k } else { 0.0 } * (time - p.0)
                                + p.1,
                        );
                        p.2.play(
                            self.pos,
                            Some(DrawTextureParams {
                                flip_x: self.flipped,
                                ..Default::default()
                            }),
                        );
                        if p.3 && !self.attacked {
                            self.attacked = true;
                            projectiles.push(Box::new(EnergyBall::new(
                                self.pos
                                    + if self.flipped {
                                        vec2(10.0, 0.0)
                                    } else {
                                        vec2(-10.0, 0.0)
                                    }
                                    + vec2(0.0, 5.0),
                                self.flipped,
                            )));
                        }
                        break;
                    } else {
                        last = p
                    }
                }
            }
            JetpackerState::Lie => {
                ASSETS
                    .jetpacker
                    .fall
                    .play(self.pos + vec2(0.0, 4.0), Some(params.clone()));
                if self.state.1 > ASSETS.jetpacker.fall.1 as f32 / 1000.0 {
                    self.state = (JetpackerState::Getup, 0.0)
                }
                dbg!(&self.state);
            }
        }

        self.pos.y += self.fall_velocity * get_frame_time();
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
    particles: &mut Vec<Particle>,
    enemies: &mut Vec<Box<dyn Enemy>>,
) {
    projectiles.retain_mut(|f| {
        if let Some(collision) = f.collision(level, player, enemies) {
            if let Some(particle) = f.particle() {
                particles.push(particle)
            }
            if collision.is_same(&CollisionType::Player) {
                if f.on_player_impact(player) {
                    return false;
                }
            } else if collision.is_same(&CollisionType::Map) {
                return false;
            } else {
                if let CollisionType::Enemy(enemy) = collision {
                    enemy.on_hit_by_player();
                    dbg!("wa");
                    return false;
                }
            }
        }

        f.update(player, level);
        return true;
    });
}
