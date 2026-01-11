use std::{collections::HashMap, f32::consts::PI, sync::LazyLock};

use image::imageops::rotate180;
use macroquad::prelude::*;

use crate::{
    assets::ASSETS,
    enemies,
    level::{Layer, Level, MAP_SCALE_FACTOR, TILE_SIZE, TileData},
    particles::Particle,
    player::{self, Player},
    utils::{
        Animation, AnimationMethods, BULLET_MATERIAL, FISH_MATERIAL, check_collision, to_map_pos,
    },
};
pub static ENEMY_IDS: LazyLock<HashMap<usize, PresetEnemies>> = LazyLock::new(|| {
    HashMap::from([
        (141, PresetEnemies::Jetpacker),
        (142, PresetEnemies::SpikeBall),
        (143, PresetEnemies::MachineGunner),
        (144, PresetEnemies::FireWagon),
        (146, PresetEnemies::Fish),
        (161, PresetEnemies::BombChain),
    ])
});
#[derive(Clone, Copy, Debug)]
pub enum PresetEnemies {
    Jetpacker,
    SpikeBall,
    MachineGunner,
    FireWagon,
    BombChain,
    Fish,
}
impl PresetEnemies {
    pub fn spawn(&self, pos: Vec2, map: &Level) -> Box<dyn Enemy> {
        match &self {
            Self::Jetpacker => Jetpacker::spawn(pos, map),
            Self::SpikeBall => SpikeBall::spawn(pos, map),
            Self::MachineGunner => MachineGunner::spawn(pos, map),
            Self::FireWagon => FireWagon::spawn(pos, map),
            Self::BombChain => BombChain::spawn(pos + TILE_SIZE / 2.0, map),
            Self::Fish => Fish::spawn(pos, map),
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
struct Fish {
    pos: Vec2,
    origin: Vec2,

    size: Vec2,
    is_attacking: bool,
    attack_clock: f32,
    attack_cooldown: f32,
    direction: f32,
}
impl Enemy for Fish {
    fn get_bounds(&self) -> (Vec2, Vec2) {
        (self.pos, self.size)
    }
    fn spawn(pos: Vec2, map: &Level) -> Box<dyn Enemy>
    where
        Self: Sized,
    {
        let mut start_x = to_map_pos(pos, map.width);
        let mut end_x = start_x;

        while map.tiles[start_x]
            .data
            .iter()
            .any(|f| f.1 == TileData::ID(80))
        {
            start_x -= 1;
        }
        while map.tiles[end_x]
            .data
            .iter()
            .any(|f| f.1 == TileData::ID(80))
        {
            end_x += 1;
        }
        Box::new(Self {
            origin: pos,
            direction: 1.0,
            attack_clock: 0.0,
            attack_cooldown: 0.0,
            is_attacking: false,
            pos,
            size: ASSETS.fish.get_size(),
        })
    }

    fn on_jumped_on_by_player(&self) -> bool {
        false
    }
    fn update(&mut self, player: &Player, map: &Level, projectiles: &mut Vec<Box<dyn Projectile>>) {
        self.attack_cooldown -= get_frame_time();
        if self.pos.y > self.origin.y {
            self.is_attacking = false;
            self.attack_clock = 0.0;
            self.pos.y = self.origin.y;
        }

        if self.is_attacking {
            self.attack_clock += get_frame_time();

            self.pos.y =
                self.origin.y + 64.0 * self.attack_clock.powi(2) - 128.0 * self.attack_clock;

            let rotation = if (128.0 * self.attack_clock - 128.0).is_sign_positive() {
                PI
            } else {
                0.0
            };
            let shader = self.pos.y + self.size.y > self.origin.y;
            if shader {
                FISH_MATERIAL.set_uniform(
                    "acidy",
                    if rotation == 0.0 { -1.0 } else { 1.0 }
                        - (self.origin.y - self.pos.y) / self.size.y,
                );
                gl_use_material(&FISH_MATERIAL);
            }
            ASSETS.fish.play(
                self.pos,
                Some(DrawTextureParams {
                    rotation,
                    ..Default::default()
                }),
            );
            if shader {
                gl_use_default_material();
            }
        } else {
            'wa: {
                let tile = to_map_pos(
                    self.pos
                        + vec2(self.direction, 0.0)
                        + if self.direction.is_sign_positive() {
                            self.size.x
                        } else {
                            0.0
                        },
                    map.width,
                );
                if (player.pos.x - (self.pos.x + self.size.x)).abs() < 20.0
                    && self.attack_cooldown <= 0.0
                {
                    self.is_attacking = true;
                    self.attack_cooldown = 12.0;
                } else {
                    ASSETS.fish_bubbles.play(self.pos, None);
                    self.pos.x += self.direction;
                }
                if tile + 1 > map.tiles.len() {
                    break 'wa;
                }
                if !map.tiles[tile]
                    .data
                    .iter()
                    .any(|f| f.1 == TileData::Animation(&ASSETS.acid))
                {
                    self.direction *= -1.0;
                }
            }
        }
    }
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
        BULLET_MATERIAL.set_uniform("alpha", 1.0 / (self.pos.x - self.origin.x).abs().powf(1.5));
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
    fn new(pos: Vec2, direction: Vec2) -> Self {
        Self {
            animation: &ASSETS.energy_ball,
            velocity: 40.0 * direction,
            pos,
            size: vec2(16.0, 16.0),
        }
    }
}

impl Projectile for EnergyBall {
    fn on_player_impact(&self, player: &mut Player) -> bool {
        player.damage(25);
        player.knockback(self.pos + self.size / 2.0, 900.0);
        return true;
    }
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
            player.damage(20);
        }
    }
}
struct StandardProjectile {
    pos: Vec2,
    size: Vec2,
    direction: Vec2,
    speed: f32,
    animation: &'static Animation,
    damage: u32,
}
impl StandardProjectile {
    fn new(
        pos: Vec2,
        size: Vec2,
        direction: Vec2,
        speed: f32,
        animation: &'static Animation,
        damage: u32,
    ) -> Self {
        Self {
            size,
            damage,
            pos,
            direction,
            speed,
            animation,
        }
    }
}
impl Projectile for StandardProjectile {
    fn on_player_impact(&self, player: &mut Player) -> bool {
        player.damage(self.damage);
        return true;
    }
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
        let collision = check_projectile_collision(self.pos, self.size, map, player, enemies);
        if let Some(collision) = &collision {
            if let CollisionType::Enemy(enemy) = collision {
                return None;
            }
        }
        return collision;
    }
}

pub trait Enemy {
    fn on_jumped_on_by_player(&self) -> bool {
        return true;
    }
    fn on_player_contact(&mut self, particles: &mut Vec<Particle>) -> Option<(Vec2, f32, u32)> {
        let bounds = self.get_bounds();
        Some((self.get_bounds().0 + self.get_bounds().1 / 2.0, 200.0, 10))
    }
    fn get_bounds(&self) -> (Vec2, Vec2);
    fn spawn(pos: Vec2, map: &Level) -> Box<dyn Enemy>
    where
        Self: Sized;

    fn update(&mut self, player: &Player, map: &Level, projectiles: &mut Vec<Box<dyn Projectile>>);
    fn on_hit_by_player(&mut self) {}
}
struct FireWagon {
    pos: Vec2,
    size: Vec2,
    direction: Vec2,
    speed: f32,
}
impl Enemy for FireWagon {
    fn get_bounds(&self) -> (Vec2, Vec2) {
        (self.pos, self.size)
    }

    fn on_player_contact(&mut self, particles: &mut Vec<Particle>) -> Option<(Vec2, f32, u32)> {
        let bounds = self.get_bounds();
        Some((bounds.0 + bounds.1 / 2.0, 850.0, 20))
    }
    fn on_hit_by_player(&mut self) {}
    fn spawn(pos: Vec2, map: &Level) -> Box<dyn Enemy>
    where
        Self: Sized,
    {
        Box::new(Self {
            pos,
            size: Vec2::ZERO,
            direction: vec2(1.0, 0.0),
            speed: 100.0,
        })
    }
    fn update(
        &mut self,
        player: &Player,
        map: &Level,
        _projectiles: &mut Vec<Box<dyn Projectile>>,
    ) {
        for i in 0..2 {
            let i = i as f32 * 16.0;
            if check_collision(
                self.pos + vec2(i, 0.0) + self.direction * self.speed * get_frame_time(),
                map,
            ) {
                self.direction.x *= -1.0;
            }
        }
        let params = Some(DrawTextureParams {
            flip_x: self.direction.x.is_sign_negative(),
            ..Default::default()
        });
        let diff = (player.pos.x + player.size.x / 2.0) - (self.pos.x + self.size.x / 2.0);
        if diff.signum() == self.direction.x.signum() && diff.abs() < 35.0 {
            ASSETS.fire_wagon_fire.play(self.pos, params.clone());
            self.size = ASSETS.fire_wagon_fire.get_size();
        } else {
            ASSETS.fire_wagon_jiggle.play(self.pos, params.clone());
            self.size = vec2(11.0, 15.0)
        }
        ASSETS.fire_wagon_wheel.play(self.pos, params);
        self.pos += self.direction * self.speed * get_frame_time();
    }
}
struct BombChain {
    origin: Vec2,
    bomb_pos: Vec2,
    rotation: f32,
    chain: &'static Texture2D,
    has_bomb: bool,
    bomb: &'static Texture2D,
}
impl Enemy for BombChain {
    fn get_bounds(&self) -> (Vec2, Vec2) {
        (self.bomb_pos, self.bomb.size())
    }
    fn on_jumped_on_by_player(&self) -> bool {
        return false;
    }
    fn on_player_contact(&mut self, particles: &mut Vec<Particle>) -> Option<(Vec2, f32, u32)> {
        if self.has_bomb {
            self.has_bomb = false;
            particles.push(Particle::new(
                Box::new(|f| ASSETS.bomb_explode.play(f, None)),
                crate::particles::Lifetime::ByTime(ASSETS.bomb_explode.get_duration()),
                None,
                self.bomb_pos + self.bomb.size() / 2.0 - ASSETS.bomb_explode.get_size() / 2.0,
            ));
            let bounds = self.get_bounds();
            Some((bounds.0 + bounds.1 / 2.0, 600., 40))
        } else {
            None
        }
    }
    fn spawn(pos: Vec2, map: &Level) -> Box<dyn Enemy>
    where
        Self: Sized,
    {
        Box::new(Self {
            bomb_pos: Vec2::ZERO,
            bomb: &ASSETS.bomb,
            chain: &ASSETS.bomb_chain,
            origin: pos,
            has_bomb: true,
            rotation: 0.0,
        })
    }

    fn update(&mut self, player: &Player, map: &Level, projectiles: &mut Vec<Box<dyn Projectile>>) {
        self.rotation += 5.0 * get_frame_time();
        self.bomb_pos = self.origin
            + (self.chain.width() + self.bomb.height() * 0.55)
                * vec2((self.rotation).cos(), (self.rotation).sin())
            - self.bomb.size() / 2.0;
        draw_texture_ex(
            self.chain,
            self.origin.x,
            self.origin.y,
            WHITE,
            DrawTextureParams {
                rotation: self.rotation,
                pivot: Some(self.origin + vec2(0.0, 1.0)),
                ..Default::default()
            },
        );
        if self.has_bomb {
            draw_texture_ex(
                self.bomb,
                self.bomb_pos.x,
                self.bomb_pos.y,
                WHITE,
                DrawTextureParams {
                    rotation: self.rotation + PI / 2.0,
                    pivot: (Some(self.bomb_pos + self.bomb.size() / 2.0)),
                    ..Default::default()
                },
            );
        }
    }
}
struct MachineGunner {
    pos: Vec2,
    shoot_clock: f32,
    hit_clock: f32,
    size: Vec2,
    flipped: bool,
    hit: bool,
}
impl Enemy for MachineGunner {
    fn on_hit_by_player(&mut self) {
        self.hit = true;
        self.hit_clock = 3.0;
    }
    fn get_bounds(&self) -> (Vec2, Vec2) {
        (self.pos, self.size)
    }
    fn spawn(pos: Vec2, map: &Level) -> Box<dyn Enemy>
    where
        Self: Sized,
    {
        Box::new(Self {
            flipped: false,
            size: ASSETS.machine_gunner_shoot.get_size(),
            pos,
            shoot_clock: 0.0,
            hit_clock: 0.0,
            hit: false,
        })
    }
    fn update(&mut self, player: &Player, map: &Level, projectiles: &mut Vec<Box<dyn Projectile>>) {
        if !self.hit {
            self.flipped = player.pos.x > self.pos.x;
            if self.shoot_clock >= 0.4 {
                projectiles.push(Box::new(StandardProjectile::new(
                    self.pos
                        + if self.flipped {
                            vec2(-8.5 + self.size.x, 15.0)
                        } else {
                            vec2(-8.5, 15.0)
                        },
                    ASSETS.laser.get_size(),
                    ((player.pos + player.size / 2.0) - (self.pos + self.size.y / 2.0))
                        .normalize_or_zero(),
                    40.0,
                    &ASSETS.laser,
                    2,
                )));
                self.shoot_clock = 0.0;
            } else {
                self.shoot_clock += get_frame_time();
            }
            ASSETS.machine_gunner_shoot.play(
                self.pos,
                Some(DrawTextureParams {
                    flip_x: self.flipped,
                    ..Default::default()
                }),
            );
        } else {
            ASSETS.machine_gunner_inactive.play(
                self.pos,
                Some(DrawTextureParams {
                    flip_x: self.flipped,
                    ..Default::default()
                }),
            );
            self.hit_clock -= get_frame_time();
        }
        if self.hit_clock < 0.0 {
            self.hit = false;
            self.hit_clock = 0.0;
        }
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
#[derive(Debug, PartialEq)]
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
        if self.state.0 != JetpackerState::Lie {
            self.state = (JetpackerState::Hit, 0.0);
        }
    }
    fn spawn(pos: Vec2, map: &Level) -> Box<dyn Enemy>
    where
        Self: Sized,
    {
        let map_pos = pos / (TILE_SIZE * MAP_SCALE_FACTOR);
        let mut tile = (map_pos.y as usize + 2) * map.width as usize + map_pos.x as usize;
        let ground_animation = if map.tiles[tile].data.iter().any(|f| f.0 == Layer::Collision) {
            &ASSETS.jetpacker.idle
        } else {
            &ASSETS.jetpacker.fly
        };
        tile = tile - 3 * map.width as usize;
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
            (0.0, 0.0, ground_animation, false),
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
                self.flipped = if player.pos.x > self.origin.x {
                    true
                } else {
                    false
                };
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
                                ((player.pos + player.size / 2.0) - (self.pos + self.size / 2.0))
                                    .normalize_or_zero(),
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
                    return false;
                }
            }
        }

        f.update(player, level);
        return true;
    });
}
