use std::{collections::HashMap, f32::consts::PI, sync::LazyLock};

use macroquad::prelude::*;

use crate::{
    assets::ASSETS,
    enemies,
    level::{self, Level, MAP_SCALE_FACTOR, SpecialTileData, TILE_SIZE, VisualData},
    particles::{Particle, Particles},
    player::{DeathCause, Player},
    projectiles::Projectile,
    utils::*,
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
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub enum PresetEnemies {
    Jetpacker,
    SpikeBall,
    MachineGunner,
    FireWagon,
    BombChain,
    Fish,
}
impl PresetEnemies {
    pub fn default_texture(&self) -> &Texture2D {
        match &self {
            Self::Jetpacker => ASSETS.jetpacker.default(),
            Self::FireWagon => ASSETS.fire_wagon.default(),
            _ => panic!(),
        }
    }
}

fn check_player_collision(pos: Vec2, size: Vec2, player: &Player) -> bool {
    let points = [(0.0, 0.0), (size.x, 0.0), (0.0, size.y), (size.x, size.y)];

    points.iter().any(|f| {
        let x = pos.x + f.0;
        let y = pos.y + f.1;
        x <= player.pos.x + player.size.x
            && x >= player.pos.x
            && y >= player.pos.y
            && player.size.y + player.pos.y >= y
    })
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
    enemies: &'a mut Vec<NewEnemy>,
) -> Option<CollisionType<'a>> {
    if check_player_collision(pos, size, player) {
        return Some(CollisionType::Player);
    } else if check_map_collision(pos, size, map) {
        return Some(CollisionType::Map);
    } else {
        for enemy in enemies {
            let bounds = (enemy.pos, enemy.size);
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
    Enemy(&'a mut NewEnemy),
}
impl<'a> CollisionType<'a> {
    fn is_same(&self, obj2: &Self) -> bool {
        std::mem::discriminant(self) == std::mem::discriminant(obj2)
    }
}
pub fn update_enemies(
    enemies: &mut Vec<NewEnemy>,
    player: &Player,
    map: &Level,
    projectiles: &mut Vec<Projectile>,
    frame_time: f32,
) {
    for enemy in enemies.iter_mut() {
        enemy.beahavior.update(
            &mut enemy.pos,
            &mut enemy.size,
            player,
            map,
            projectiles,
            frame_time,
        );
    }
}
struct Fish {
    origin: Vec2,

    size: Vec2,
    is_attacking: bool,
    attack_clock: f32,
    attack_cooldown: f32,
    direction: f32,
}

pub trait EnemyBehaviour {
    fn update(
        &mut self,
        pos: &mut Vec2,
        size: &mut Vec2,
        player: &Player,
        map: &Level,
        projectiles: &mut Vec<Projectile>,
        frame_time: f32,
    );
    fn new(pos: Vec2, level: &Level) -> Self
    where
        Self: Sized;
}
pub struct NewEnemy {
    pub pos: Vec2,
    pub size: Vec2,
    beahavior: Box<dyn EnemyBehaviour>,
}
impl NewEnemy {
    pub fn from(preset: PresetEnemies, pos: Vec2, level: &Level) -> Self {
        let behaviour;
        let size;
        match preset {
            PresetEnemies::Jetpacker => {
                size = ASSETS.jetpacker.get_size();
                behaviour = Box::new(Jetpacker::new(pos, level));
            }
            _ => panic!(),
        };
        Self {
            pos,
            size,
            beahavior: behaviour,
        }
    }
}
impl EnemyBehaviour for Fish {
    fn new(pos: Vec2, map: &Level) -> Self {
        {
            let mut start_x = to_world_pos(pos, map.width);
            let mut end_x = start_x;

            while map.tiles[start_x]
                .special_data
                .iter()
                .any(|f| *f == SpecialTileData::Path)
            {
                start_x -= 1;
            }
            while map.tiles[end_x]
                .special_data
                .iter()
                .any(|f| *f == SpecialTileData::Path)
            {
                end_x += 1;
            }
            Self {
                origin: pos,
                direction: 60.0,
                attack_clock: 0.0,
                attack_cooldown: 0.0,
                is_attacking: false,

                size: ASSETS.fish.get_size(),
            }
        }
    }

    fn update(
        &mut self,
        pos: &mut Vec2,
        size: &mut Vec2,
        player: &Player,
        map: &Level,
        projectiles: &mut Vec<Projectile>,
        frame_time: f32,
    ) {
        self.attack_cooldown -= frame_time;
        if pos.y > self.origin.y {
            self.is_attacking = false;
            self.attack_clock = 0.0;
            pos.y = self.origin.y;
        }

        if self.is_attacking {
            self.attack_clock += frame_time;

            pos.y = self.origin.y + 64.0 * self.attack_clock.powi(2) - 128.0 * self.attack_clock;

            let rotation = if (128.0 * self.attack_clock - 128.0).is_sign_positive() {
                PI
            } else {
                0.0
            };
            let shader = pos.y + self.size.y > self.origin.y;
            if shader {
                FISH_MATERIAL.set_uniform(
                    "acidy",
                    if rotation == 0.0 { -1.0 } else { 1.0 }
                        - (self.origin.y - pos.y) / self.size.y,
                );
                gl_use_material(&FISH_MATERIAL);
            }
            ASSETS.fish.get("jump").play(
                *pos,
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
                let tile = to_world_pos(
                    vec2(
                        pos.x
                            + self.direction * frame_time
                            + if self.direction.is_sign_positive() {
                                self.size.x
                            } else {
                                0.0
                            },
                        self.origin.y,
                    ),
                    map.width,
                );

                if (player.pos.x - (pos.x + self.size.x)).abs() < 20.0
                    && self.attack_cooldown <= 0.0
                {
                    dbg!("attacking");
                    self.is_attacking = true;
                    self.attack_cooldown = 5.0;
                } else {
                    ASSETS.fish.get("bubbles").play(*pos, None);
                    if tile + 1 > map.tiles.len() {
                        dbg!("out of bounds fish");
                        break 'wa;
                    }
                    if !map.tiles[tile]
                        .special_data
                        .iter()
                        .any(|f| *f == SpecialTileData::Acid)
                    {
                        dbg!("beep bepp");
                        self.direction *= -1.0;
                    } else {
                        pos.x += self.direction * frame_time;
                    }
                }
            }
        }
    }
}

struct FireWagon {
    direction: Vec2,
    speed: f32,
}
impl EnemyBehaviour for FireWagon {
    fn new(pos: Vec2, level: &Level) -> Self {
        panic!()
    }

    fn update(
        &mut self,
        pos: &mut Vec2,
        size: &mut Vec2,
        player: &Player,
        map: &Level,
        projectiles: &mut Vec<Projectile>,
        frame_time: f32,
    ) {
        for i in 0..2 {
            let i = i as f32 * 16.0;
            if check_collision(
                *pos + vec2(i, 0.0) + self.direction * self.speed * frame_time,
                map,
            ) {
                self.direction.x *= -1.0;
            }
        }
        let params = Some(DrawTextureParams {
            flip_x: self.direction.x.is_sign_negative(),
            ..Default::default()
        });
        let diff = (player.pos.x + player.size.x / 2.0) - (pos.x + size.x / 2.0);
        if diff.signum() == self.direction.x.signum() && diff.abs() < 35.0 {
            let animation = ASSETS.fire_wagon.get("fire");
            animation.play(*pos, params.clone());
            *size = animation.get_size();
        } else {
            ASSETS.fire_wagon.get("jiggle").play(*pos, params.clone());
            *size = vec2(11.0, 15.0)
        }
        ASSETS.fire_wagon.get("drive").play(*pos, params);
        *pos += self.direction * self.speed * frame_time;
    }
}
struct BombChain {
    bomb_pos: Vec2,
    rotation: f32,
    has_bomb: bool,
    origin: Vec2,
}
impl EnemyBehaviour for BombChain {
    fn new(pos: Vec2, level: &Level) -> Self
    where
        Self: Sized,
    {
        Self {
            rotation: 0.0,
            bomb_pos: Vec2::ZERO,
            has_bomb: true,
            origin: pos,
        }
    }

    fn update(
        &mut self,
        pos: &mut Vec2,
        size: &mut Vec2,
        player: &Player,
        map: &Level,
        projectiles: &mut Vec<Projectile>,
        frame_time: f32,
    ) {
        self.rotation += 5.0 * frame_time;
        let bomb = &ASSETS.bomb_chain;
        self.bomb_pos = self.origin
            + (ASSETS.bomb_chain.width() + bomb.height() * 0.55)
                * vec2((self.rotation).cos(), (self.rotation).sin())
            - *size / 2.0;
        draw_texture_ex(
            &ASSETS.bomb_chain,
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
                &ASSETS.bomb,
                self.bomb_pos.x,
                self.bomb_pos.y,
                WHITE,
                DrawTextureParams {
                    rotation: self.rotation + PI / 2.0,
                    pivot: (Some(self.bomb_pos + ASSETS.bomb.size() / 2.0)),
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
impl EnemyBehaviour for MachineGunner {
    fn new(pos: Vec2, level: &Level) -> Self
    where
        Self: Sized,
    {
        panic!()
    }
    fn update(
        &mut self,
        pos: &mut Vec2,
        size: &mut Vec2,
        player: &Player,
        map: &Level,
        projectiles: &mut Vec<Projectile>,
        frame_time: f32,
    ) {
        if !self.hit {
            self.flipped = player.pos.x > self.pos.x;
            if self.shoot_clock >= 0.4 {
                // shoot
                self.shoot_clock = 0.0;
            } else {
                self.shoot_clock += frame_time;
            }
            ASSETS.machine_gunner.get(&"shoot").play(
                self.pos,
                Some(DrawTextureParams {
                    flip_x: self.flipped,
                    ..Default::default()
                }),
            );
        } else {
            ASSETS.machine_gunner.get("inactive").play(
                self.pos,
                Some(DrawTextureParams {
                    flip_x: self.flipped,
                    ..Default::default()
                }),
            );
            self.hit_clock -= frame_time;
        }
        if self.hit_clock < 0.0 {
            self.hit = false;
            self.hit_clock = 0.0;
        }
    }
}

// impl EnemyBehaviour for SpikeBall {
//     fn new(pos: Vec2, level: &Level) -> Self {}
//     fn update(
//         &mut self,
//         player: &Player,
//         map: &Level,
//         projectiles: &mut Vec<Projectile>,
//         frame_time: f32,
//     ) {
//         self.pos += (player.pos - self.pos).normalize() * 10.0 * frame_time;
//         let _ = &ASSETS.spike_ball.get("4").play(self.pos, None);
//     }
// }
#[derive(Debug, PartialEq)]
enum JetpackerState {
    Normal,
    Hit,
    Lie,
    Getup,
    Fall,
}
struct Jetpacker {
    origin: Vec2,
    attacked: bool,
    flipped: bool,
    behavior_curve: [(f32, f32, &'static Animation, bool); 6],
    state: (JetpackerState, f32),
    fall_velocity: f32,
}

impl EnemyBehaviour for Jetpacker {
    fn new(pos: Vec2, map: &Level) -> Self {
        let map_pos = pos / (TILE_SIZE * MAP_SCALE_FACTOR);
        let mut tile = (map_pos.y as usize + 2) * map.width as usize + map_pos.x as usize;
        let ground_animation = if map.tiles[tile].collision {
            &ASSETS.jetpacker.get("idle")
        } else {
            &ASSETS.jetpacker.get("fly")
        };
        tile = tile - 3 * map.width as usize;
        let mut fly_height = 0.0;
        while map.tiles[tile]
            .visual
            .iter()
            .any(|f| *f == VisualData::ID(240))
        {
            println!("path above at tile {}", tile);
            tile = tile - map.width as usize;
            fly_height -= TILE_SIZE * MAP_SCALE_FACTOR;
        }
        let flight_speed = -50.0;
        let flight_time = (fly_height / flight_speed).abs();

        dbg!(flight_time);
        let animation = ASSETS.jetpacker.get("fly");
        let curve: [(f32, f32, &Animation, bool); 6] = [
            (0.0, 0.0, ground_animation, false),
            (1.5, 0.0, animation, false),
            (flight_time + 1.5, fly_height, animation, false),
            (flight_time + 2.5 + 1.5, fly_height, animation, true),
            (flight_time + 4.0 + 1.5, fly_height, animation, false),
            (flight_time + 5.0 + 1.5, 0.0, animation, false),
        ];
        Self {
            state: (JetpackerState::Normal, 0.0),
            fall_velocity: 0.0,

            behavior_curve: curve,
            flipped: false,
            attacked: false,
            origin: pos,
        }
    }

    fn update(
        &mut self,
        pos: &mut Vec2,
        size: &mut Vec2,
        player: &Player,
        map: &Level,
        projectiles: &mut Vec<Projectile>,
        frame_time: f32,
    ) {
        self.state.1 += frame_time;
        if pos.y > self.origin.y {
            *pos = self.origin;
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
                ASSETS
                    .jetpacker
                    .get("fall")
                    .play(*pos, Some(params.clone()));
            }
            JetpackerState::Getup => {
                let animation = ASSETS.jetpacker.get("getup");
                animation.play_with_clock(*pos, self.state.1, Some(params.clone()));
                if self.state.1 > animation.1 as f32 / 1000.0 {
                    self.state = (JetpackerState::Normal, 0.0)
                }
            }
            JetpackerState::Hit => {
                self.flipped = if pos.x > self.origin.x { true } else { false };
                let animation = ASSETS.jetpacker.get("hit");
                animation.play_with_clock(*pos, self.state.1, Some(params.clone()));
                if self.state.1 > animation.1 as f32 / 1000.0 {
                    self.state = (JetpackerState::Fall, 0.0)
                }
            }
            JetpackerState::Normal => {
                self.flipped = pos.x > self.origin.x;

                if self.state.1 + frame_time > self.behavior_curve.last().unwrap().0 {
                    self.state.1 = 0.0;
                    self.attacked = false;
                }
                let mut last = self.behavior_curve.last().unwrap();
                let time = self.state.1;
                for p in self.behavior_curve.iter().rev() {
                    if time >= p.0 {
                        let k = (last.1 - p.1) / (last.0 - p.0);
                        *pos = vec2(
                            self.origin.x,
                            self.origin.y
                                + if !k.is_infinite() { k } else { 0.0 } * (time - p.0)
                                + p.1,
                        );
                        p.2.play(
                            *pos,
                            Some(DrawTextureParams {
                                flip_x: self.flipped,
                                ..Default::default()
                            }),
                        );
                        if p.3 && !self.attacked {
                            self.attacked = true;
                            // projectiles.push(Box::new(EnergyBall::new(
                            //     *pos
                            //         + if self.flipped {
                            //             vec2(10.0, 0.0)
                            //         } else {
                            //             vec2(-10.0, 0.0)
                            //         }
                            //         + vec2(0.0, 5.0),
                            //     ((*pos + player.size / 2.0) - (*pos + self.size / 2.0))
                            //         .normalize_or_zero(),
                            // )));
                        }
                        break;
                    } else {
                        last = p
                    }
                }
            }
            JetpackerState::Lie => {
                let animation = ASSETS.jetpacker.get("fall");

                animation.play(*pos + vec2(0.0, 4.0), Some(params.clone()));
                if self.state.1 > animation.1 as f32 / 1000.0 {
                    self.state = (JetpackerState::Getup, 0.0)
                }
            }
        };
        pos.y += self.fall_velocity * frame_time;
    }
}
