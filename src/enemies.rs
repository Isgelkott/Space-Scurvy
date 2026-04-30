use std::{collections::HashMap, f32::consts::PI, panic, sync::LazyLock};

use macroquad::prelude::*;

use crate::{
    assets::ASSETS,
    level::{self, Level, SpecialTileData, TILE_SIZE, floored_pos},
    particles::Particle,
    player::{Bullet, GRAVITY, Player},
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
fn is_grounded_and_check_bounds(
    pos: &mut Vec2,
    size: &mut Vec2,
    velocity: &mut Vec2,
    frame_time: f32,
    level: &Level,
) -> (bool, bool) {
    let mut grounded = false;
    let mut collid_with_wall = false;
    for y in (0..(size.y / 16.0) as i16 + 2).rev() {
        let y = ((y * 16) as f32).min(size.y);
        for x in 0..((size.x / 16.0).ceil()) as i16 + 2 {
            let x = ((x * 16) as f32).min(size.x - 1.0);
            let point = (x, y);
            let mut map_pos =
                *pos + vec2(1.0, 0.0) + vec2(0.0, velocity.y) * frame_time + vec2(point.0, point.1);
            // if x != 0.0 && map_pos.x.fract() == 0.0 {
            //     map_pos.x -= 1.0;
            // }
            let tile = level.get_tile(map_pos);
            if let Some(tile) = tile {
                let tile_pos = floored_pos(map_pos);
                if tile.collision {
                    if DEBUG_FLAGS.show_collisions {
                        draw_rectangle(tile_pos.x, tile_pos.y, 5.0, 5.0, BLUE);
                    }

                    pos.y = pos
                        .y
                        .clamp(tile_pos.y - point.1, tile_pos.y + TILE_SIZE - point.1);
                    if pos.y == (tile_pos.y - point.1) || pos.y == tile_pos.y + TILE_SIZE - point.1
                    {
                        grounded = true;

                        velocity.y = 0.;
                    }
                } else {
                    if DEBUG_FLAGS.show_collisions {
                        draw_rectangle(tile_pos.x, tile_pos.y, 5.0, 5.0, RED);
                    }
                }
            } else {
                dbg!("out of bounds :(");
            }
        }
    }
    for y in (0..(size.y / 16.0) as i16 + 2).rev() {
        let y = ((y * 16) as f32).min(size.y - 1.0);
        for x in 0..((size.x / 16.0).ceil()) as i16 + 1 {
            let x = ((x * 16) as f32).min(size.x);
            let point = (x, y);
            let mut map_pos = *pos + vec2(velocity.x, 0.0) * frame_time + vec2(point.0, point.1);
            if x != 0.0 && map_pos.x.fract() == 0.0 {
                map_pos.x -= 1.0;
            }
            let tile = level.get_tile(map_pos);

            if let Some(tile) = tile {
                let tile_pos = floored_pos(map_pos);

                if tile.collision {
                    if DEBUG_FLAGS.show_collisions {
                        draw_rectangle(tile_pos.x, tile_pos.y, 5.0, 5.0, YELLOW);
                    }

                    let x1 = tile_pos.x - point.0;
                    let x2 = tile_pos.x + TILE_SIZE - point.0;
                    // pos.x = if velocity.x.is_sign_positive() {
                    //     x1
                    // } else {
                    //     x2
                    // };
                    // velocity.x = 0.;

                    collid_with_wall = true;
                }
            } else {
                dbg!("out of bounds :(");
            }
        }
    }

    return (grounded, collid_with_wall);
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
    // return points.iter().any(|f| check_collision(*f, map));
    panic!()
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
    level: &Level,
    projectiles: &mut Vec<Projectile>,
    frame_time: f32,
    bullets: &mut Vec<Bullet>,
    particles: &mut Vec<Particle>,
) {
    for enemy in enemies.iter_mut() {
        bullets.retain_mut(|bullet| {
            for i in 0..2 {
                let bullet_pos =
                    bullet.pos - vec2(i as f32 * TILE_SIZE * bullet.direction.signum(), 0.);
                if check_collision_rectangle_collision(
                    (enemy.pos, enemy.size),
                    (bullet_pos, bullet.size),
                ) {
                    enemy.kill();
                    particles.push(Particle::preset(
                        crate::particles::Particles::Blood,
                        bullet_pos,
                    ));
                    return false;
                }
            }
            return true;
        });
        if let Some((time, fall_velocity)) = &mut enemy.die {
            *time += frame_time;
            enemy.animations.get("die").play_with_clock(
                enemy.pos + vec2(0., 4.),
                *time,
                Some(DrawTextureParams {
                    flip_x: enemy.flipped,
                    ..Default::default()
                }),
            );
            let (grounded, _) = is_grounded_and_check_bounds(
                &mut enemy.pos,
                &mut enemy.size,
                &mut vec2(0., *fall_velocity),
                frame_time,
                level,
            );
            if !grounded {
                *fall_velocity += GRAVITY * frame_time;
                enemy.pos.y += *fall_velocity * frame_time;
            }
        } else {
            enemy.beahavior.update(
                &mut enemy.pos,
                &mut enemy.size,
                player,
                level,
                projectiles,
                frame_time,
                &mut enemy.flipped,
            );
        }
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
        flipped: &mut bool,
    );
    fn new(pos: Vec2, level: &Level) -> Self
    where
        Self: Sized;
}
pub struct NewEnemy {
    pub pos: Vec2,
    pub size: Vec2,
    animations: &'static AnimationGroup,
    pub die: Option<(f32, f32)>,
    flipped: bool,
    beahavior: Box<dyn EnemyBehaviour>,
    enemy: PresetEnemies,
    clock: f32,
}
impl NewEnemy {
    pub fn kill(&mut self) {
        self.die = Some((0., 0.))
    }
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
            PresetEnemies::FireWagon => {
                animations = &ASSETS.fire_wagon;
                behaviour = Box::new(FireWagon::new(pos, level))
            }
            PresetEnemies::BombChain => {
                animations = &ASSETS.bomb_chain;
                behaviour = Box::new(BombChain::new(pos, level))
            }
            _ => panic!(),
        };

        Self {
            flipped: false,
            clock: 0.0,
            pos,
            animations,
            die: None,
            size: animations.size(),
            beahavior: behaviour,
            enemy: preset,
        }
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
        flipped: &mut bool,
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
    velocity: Vec2,
}
impl EnemyBehaviour for FireWagon {
    fn new(pos: Vec2, level: &Level) -> Self {
        Self {
            velocity: vec2(20., 0.),
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
        flipped: &mut bool,
    ) {
        let (grounded, collid_with_wall) =
            is_grounded_and_check_bounds(pos, size, &mut self.velocity, frame_time, map);
        if grounded {
            self.velocity.y = 0.;
        } else {
            self.velocity.y += GRAVITY * frame_time * 0.01;
        }
        for i in 0..2 {
            let i = i as f32 * TILE_SIZE;
            if let Some(tile) = map.get_tile(*pos + vec2(i, 0.0) + self.velocity * frame_time)
                && tile.collision
            {
                self.velocity.x *= -1.0;
            }
        }
        let params = Some(DrawTextureParams {
            flip_x: self.velocity.x.is_sign_negative(),
            ..Default::default()
        });
        let diff = (player.pos.x + player.size.x / 2.0) - (pos.x + size.x / 2.0);
        if diff.signum() == self.velocity.x.signum() && diff.abs() < 35.0 {
            let animation = ASSETS.fire_wagon.get("fire");
            animation.play(*pos, params.clone());
        } else {
            ASSETS.fire_wagon.get("jiggle").play(*pos, params.clone());
        }
        ASSETS.fire_wagon.get("drive").play(*pos, params);
        *pos += self.velocity * frame_time;
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
            origin: pos + TILE_SIZE / 2.,
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
        flipped: &mut bool,
    ) {
        self.rotation += 5.0 * frame_time;
        let bomb = &ASSETS.bomb_chain;
        self.bomb_pos = self.origin
            + (ASSETS.bomb_chain.base().size().x)
                * vec2((self.rotation).cos(), (self.rotation).sin())
            - ASSETS.bomb.size() / 2.;
        draw_texture_ex(
            ASSETS.bomb_chain.base().base(),
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
                    rotation: self.rotation + PI / 2.0 + 0.4,
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
        flipped: &mut bool,
    ) {
        if !self.hit {
            self.flipped = player.pos.x > self.pos.x;
            if self.shoot_clock >= 0.4 {
                // shoot
                self.shoot_clock = 0.0;
            } else {
                self.shoot_clock += frame_time;
            }
            ASSETS.machine_gunner.get("shoot").play(
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
    Stall(f32),
}
struct Jetpacker {
    fly_height: f32,
    origin: Vec2,
    direction: Vec2,
    flipped: bool,
    clock: f32,
    state: JetpackerState,
    animations: &'static AnimationGroup,
}

impl EnemyBehaviour for Jetpacker {
    fn new(pos: Vec2, level: &Level) -> Self {
        let mut min_y = pos.y - TILE_SIZE;

        while let Some(tile) = level.get_tile(vec2(pos.x, min_y))
            && tile.has_special(SpecialTileData::Path)
        {
            min_y = min_y - TILE_SIZE;
        }
        let fly_height = pos.y - min_y;
        dbg!(fly_height);
        Self {
            fly_height,
            origin: pos,
            direction: Vec2::NEG_Y,
            state: JetpackerState::Normal,
            clock: 0.0,
            flipped: false,
            animations: &ASSETS.jetpacker,
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
        flipped: &mut bool,
    ) {
        const SPEED: f32 = 75.0;
        const STALL_DURATION: f32 = 1.2;

        let mut check_view_direction = || self.flipped = player.center().x > pos.x;
        match &mut self.state {
            JetpackerState::Normal => {
                check_view_direction();
                self.animations.play_tag(
                    "fly",
                    *pos,
                    Some(DrawTextureParams {
                        flip_x: self.flipped,
                        ..Default::default()
                    }),
                );
                pos.y += self.direction.y * SPEED * frame_time;
                if pos.y > self.origin.y {
                    self.state = JetpackerState::Stall(STALL_DURATION);
                    return;
                }
                if pos.y < self.origin.y - self.fly_height {
                    self.state = JetpackerState::Stall(STALL_DURATION);
                    projectiles.push(Projectile::from(
                        *pos,
                        crate::projectiles::Projectiles::EnergyBall,
                        (player.pos - *pos).normalize(),
                    ));
                }
            }
            JetpackerState::Stall(duration) => {
                check_view_direction();

                self.animations.play_tag(
                    "fly",
                    *pos,
                    Some(DrawTextureParams {
                        flip_x: self.flipped,
                        ..Default::default()
                    }),
                );
                *duration -= frame_time;
                if duration.is_sign_negative() {
                    self.state = JetpackerState::Normal;
                    self.direction = vec2(0., -self.direction.y);
                }
            }
        }
        *flipped = self.flipped;
    }
}
