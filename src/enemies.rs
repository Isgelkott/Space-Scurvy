use std::{collections::HashMap, f32::consts::PI, panic, sync::LazyLock};

use macroquad::prelude::*;

use crate::{
    assets::ASSETS,
    level::{Level, SpecialTileData, TILE_SIZE},
    player::Player,
    projectiles::Projectile,
    utils::*,
};
pub static ENEMY_IDS: LazyLock<HashMap<u16, PresetEnemies>> = LazyLock::new(|| {
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
fn draw_path(path: Vec<Vec2>) {
    let mut iter = path.iter();
    while let Some(p) = iter.next() {
        dbg!(p);
        if let Some(p2) = iter.next() {
            for y in 0..((p2.y - p.y) as i8 / 16) {
                for x in 0..((p2.x - p.x) as i8 / 16) {
                    dbg!(x, y);
                    draw_rectangle(x as f32 * 16.0, y as f32 * 16.0, 16.0, 16.0, GREEN);
                }
            }
        }
    }
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
    animations: &'static AnimationGroup,
    beahavior: Box<dyn EnemyBehaviour>,
    enemy: PresetEnemies,
    clock: f32,
}
impl NewEnemy {
    pub fn new(preset: PresetEnemies, pos: Vec2, level: &Level) -> Self {
        let animations;
        let behaviour: Box<dyn EnemyBehaviour>;

        match preset {
            PresetEnemies::Jetpacker => {
                animations = &ASSETS.jetpacker;
                behaviour = Box::new(Jetpacker::new(pos, level));
            }
            PresetEnemies::Fish => {
                animations = &ASSETS.fish;
                behaviour = Box::new(Fish::new(pos, level));
            }
            _ => panic!(),
        };

        Self {
            animations,
            clock: 0.0,
            pos,
            size: animations.get_size(),
            beahavior: behaviour,
            enemy: preset,
        }
    }
    fn update(
        &mut self,
        size: &mut Vec2,
        player: &Player,
        map: &Level,
        projectiles: &mut Vec<Projectile>,
        frame_time: f32,
    ) {
        self.beahavior
            .update(&mut self.pos, size, player, map, projectiles, frame_time);
    }
}
struct Fish {
    origin: Vec2,

    is_attacking: bool,
    attack_clock: f32,
    attack_cooldown: f32,
    direction: f32,
}
impl EnemyBehaviour for Fish {
    fn new(pos: Vec2, map: &Level) -> Self {
        {
            Self {
                origin: pos,
                direction: 60.0,
                attack_clock: 0.0,
                attack_cooldown: 0.0,
                is_attacking: false,
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
            let shader = pos.y + size.y > self.origin.y;
            if shader {
                FISH_MATERIAL.set_uniform(
                    "acidy",
                    if rotation == 0.0 { -1.0 } else { 1.0 } - (self.origin.y - pos.y) / size.y,
                );
                gl_use_material(&FISH_MATERIAL);
            }
            ASSETS.fish.get("attack").play(
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
            if (player.pos.x - (pos.x + size.x)).abs() < 20.0 && self.attack_cooldown <= 0.0 {
                dbg!("attacking");
                self.is_attacking = true;
                self.attack_cooldown = 5.0;
            } else {
                ASSETS.fish.get("bubbles").play(*pos, None);

                if let Some(tile) = map.get_tile(
                    *pos + vec2(
                        if self.direction.is_sign_positive() {
                            size.x
                        } else {
                            0.0
                        },
                        0.0,
                    ),
                ) && tile.collision
                {
                    //dbg!("beep bepp");
                    //dbg!(tile);
                    //dbg!(self.origin.y);
                    self.direction *= -1.0;
                }
                pos.x += self.direction * frame_time;
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
        Self {
            direction: vec2(1.0, 0.0),
            speed: 20.0,
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
    pos: Vec2,
    direction: Option<Vec2>,
    flipped: bool,
    clock: f32,
    activated: bool,
    state: JetpackerState,
}
fn scan_for_path(pos: Vec2, level: &Level) -> Vec2 {
    let mut direction = None;
    for c in &level.chunks {
        for t in &c.tiles {
            if t.has_special(SpecialTileData::Path) {}
        }
    }
    for y in -1..=1 {
        for x in -1..=1 {
            let pos = pos + vec2(x as f32 * TILE_SIZE, y as f32 * TILE_SIZE);
            if let Some(tile) = level.get_tile(pos) {
                if tile.has_special(SpecialTileData::Path) {
                    direction = Some(vec2(x as f32, y as f32));
                    break;
                }
                dbg!(pos);
            }
        }
    }
    direction.unwrap()
}
impl EnemyBehaviour for Jetpacker {
    fn new(pos: Vec2, level: &Level) -> Self {
        Self {
            direction: None,
            state: JetpackerState::Normal,
            clock: 0.0,
            activated: false,
            flipped: false,
            pos,
        }
    }

    fn update(
        &mut self,
        pos: &mut Vec2,
        size: &mut Vec2,
        player: &Player,
        level: &Level,
        projectiles: &mut Vec<Projectile>,
        frame_time: f32,
    ) {
        const SPEED: f32 = 200.0;
        if self.direction.is_none()
            || level
                .get_tile(self.pos + self.direction.unwrap() * TILE_SIZE)
                .is_none()
        {
            self.direction = Some(scan_for_path(*pos, level));
        }
        if DEBUG_FLAGS.show_path {
            // for y in ((self.origin.y) / TILE_SIZE) as i8..=(self.height + self.origin.y) as i8 {
            //     print!("wa");
            //     draw_rectangle(self.origin.x, y as f32 * TILE_SIZE, 8., 8., ORANGE);
            // }
        }
        match &self.state {
            JetpackerState::Normal => {
                self.pos += self.direction.unwrap() * SPEED * frame_time;
            }
            _ => panic!(),
        }
    }
}
