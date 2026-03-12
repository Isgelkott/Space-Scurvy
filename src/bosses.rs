use std::{collections::HashSet, f32::consts::PI};

use macroquad::{prelude::*, rand::gen_range};

use crate::{
    assets::ASSETS,
    enemies::{NewEnemy, PresetEnemies},
    level::{Level, SpecialTileData, TILE_SIZE},
    particles::{self, Particle},
    player::Player,
    projectiles::Projectile,
    utils::*,
};
fn find_special_tile_data(level: &Level, data: SpecialTileData) -> Vec2 {
    panic!()
    // to_game_pos(
    //     level
    //         .tiles
    //         .iter()
    //         .enumerate()
    //         .find(|f| f.1.special_data.iter().any(|f| *f == data))
    //         .unwrap()
    //         .0,
    //     level,
    // )
}
pub enum Bosses {
    RedGuy,
}
impl Bosses {
    pub fn to_boss(&self, pos: Vec2, level: &Level) -> Box<dyn Boss> {
        match self {
            Bosses::RedGuy => RedGuy::new(pos, level),
        }
    }
}
pub trait Boss {
    fn new(tile: Vec2, level: &Level) -> Box<dyn Boss>
    where
        Self: Sized;
    fn update(
        &mut self,
        map: &Level,
        enemies: &mut Vec<NewEnemy>,
        projectiles: &mut Vec<Projectile>,
        time: f32,
        player: &Player,
        particles: &mut Vec<Particle>,
    );
}

#[derive(Debug, PartialEq, Clone, Copy)]
#[expect(dead_code)]

enum RedGuyPhase {
    ShootRocket,
    Idle(Vec2, f32),
    MoveTo(Vec2, Vec2),
    Load(PresetEnemies),
    Shoot(PresetEnemies),
    Entry,
}
const HOVER_RANGE: (f32, f32) = (40.0, 20.0);
#[derive(PartialEq, Debug)]
enum CannonActions {
    Shoot,
    Idle,
    OnCooldown(f32),
}

struct Cannon {
    pos: Vec2,
    angle: f32,
    clock: f32,
    action: CannonActions,
    shot: Option<CannonShot>,
}

impl Cannon {
    fn cooldown() -> f32 {
        gen_range(10.0, 12.0)
    }
    fn new(pos: Vec2, level: &Level) -> Self {
        Self {
            shot: None,
            angle: 0.0,
            pos,
            clock: 0.0,
            action: CannonActions::Idle,
        }
    }
}
struct CannonShot {
    pos: Vec2,
    speed: f32,
    past_pos: Vec<Vec2>,
}
impl CannonShot {
    fn new(pos: Vec2) -> Self {
        Self {
            pos,
            speed: 20.0,
            past_pos: Vec::new(),
        }
    }
}
struct RedGuy {
    pos: Vec2,
    crane: Vec<(f32, Vec2)>,
    catapult: Vec<(f32, Vec2)>,
    allowed_area: (Vec2, Vec2),
    actions: Vec<(RedGuyPhase, f32)>,
    fallings_enemeies: Vec<(PresetEnemies, Vec2, f32)>,
    attack_cooldowns: Vec<(RedGuyPhase, f32, f32)>,
    incoming_rocket: Option<(Vec2, f32)>,
    cannon: Cannon,
}
impl RedGuy {
    fn update_cannon(&mut self, frame_time: f32, player: &Player) {
        self.cannon.clock += frame_time;
        let direction = (self.pos - self.cannon.pos).normalize();
        let stand_animation;
        let barrel_animation;
        let center =
            self.cannon.pos + vec2(65., 14.) - vec2(ASSETS.cannon_barrel.get_size().x / 2.0, 0.0);

        let shoot_point = vec2(center.x, center.y);
        let boss_center = vec2(
            self.pos.x + ASSETS.red_boss.get_size().x,
            self.pos.y + ASSETS.red_boss.get_size().y,
        );
        let desired_angle = (self.pos + ASSETS.red_boss.get_size() / 2.0 - center);
        let difference =
            desired_angle.angle_between(vec2(self.cannon.angle.cos(), self.cannon.angle.sin()));
        if (difference - PI).abs() < 0.2 {
        } else {
            self.cannon.angle += difference.signum() * frame_time;
        }

        match self.cannon.action {
            CannonActions::OnCooldown(duration) => {
                stand_animation = ASSETS.cannon.get("cooldown");
                barrel_animation = ASSETS.cannon_barrel.get("cooldown");
                if self.cannon.clock > duration {
                    self.cannon.action = CannonActions::Idle;
                    self.cannon.clock = 0.0;
                }
            }
            CannonActions::Shoot => {
                let flipped = direction.x.is_sign_positive();
                barrel_animation = ASSETS.cannon_barrel.get("shoot");
                stand_animation = ASSETS.cannon.get("idle");
                let duration = barrel_animation.get_duration();

                if self.cannon.clock > duration {
                    self.cannon.shot = Some(CannonShot::new(shoot_point));
                    dbg!("wa");
                    // self.cannon.action = CannonActions::OnCooldown(Cannon::cooldown());
                    self.cannon.clock = 0.0;
                }
            }
            CannonActions::Idle => {
                stand_animation = ASSETS.cannon.get("idle");
                barrel_animation = ASSETS.cannon_barrel.get("idle");
                let switch_pos = self.cannon.pos + vec2(66.0, 72.0);

                if is_key_pressed(KeyCode::E) && (player.pos.x - switch_pos.x).abs() < 100.0 {
                    self.cannon.action = CannonActions::Shoot;
                    self.cannon.clock = 0.0;
                }
            }
        }

        stand_animation.play(self.cannon.pos, None);
        barrel_animation.play(
            center,
            Some(DrawTextureParams {
                rotation: self.cannon.angle + PI / 2.0,
                pivot: Some(center + vec2(ASSETS.cannon_barrel.get_size().x / 2.0, 0.0)),

                ..Default::default()
            }),
        );
        if self.cannon.action == CannonActions::Shoot
            && self.cannon.clock > ASSETS.cannon_barrel.get("shoot").get_duration() - 0.2
        {
            // draw_line(
            //     shoot_point.x,
            //     shoot_point.y,
            //     boss_center.x,
            //     boss_center.y,
            //     8.0,
            //     Color::from_hex(0xffeb57),
            // );
        }
    }

    fn new_location(allowed_area: (Vec2, Vec2)) -> Vec2 {
        vec2(
            gen_range(
                allowed_area.0.x + HOVER_RANGE.0,
                allowed_area.1.x - 2.0 * ASSETS.red_boss.get_size().x - HOVER_RANGE.0,
            ),
            gen_range(
                allowed_area.1.y - HOVER_RANGE.1 - TILE_SIZE * 4.0,
                allowed_area.1.y - ASSETS.red_boss.get_size().y * 2.0 - HOVER_RANGE.1,
            ),
        )
    }
    fn rand_enemy() -> PresetEnemies {
        match gen_range(0, 2) {
            0..2 => PresetEnemies::FireWagon,
            _ => panic!(),
        }
    }
    fn get_crane() -> Vec<(f32, Vec2)> {
        let mut points = Vec::new();
        for (frame, duiration) in ASSETS.red_boss.get("crane").0.iter() {
            let width = frame.width();
            let heigt = frame.height();
            for (index, pixels) in frame
                .get_texture_data()
                .bytes
                .windows(4)
                .step_by(4)
                .enumerate()
            {
                if pixels == [255, 200, 37, 255] {
                    points.push((
                        *duiration as f32 / 1000.0,
                        vec2(
                            (index % width as usize) as f32,
                            (index / width as usize) as f32,
                        ),
                    ));
                }
            }
        }
        points
    }
}
impl Boss for RedGuy {
    fn new(pos: Vec2, level: &Level) -> Box<dyn Boss> {
        panic!()
        // fn valid_tiles_around(tile: usize, level: &Level) -> Vec<usize> {
        //     let mut without_collision = Vec::new();
        //     let tiles = [
        //         tile.saturating_sub(level.width),
        //         tile + level.width,
        //         tile.saturating_sub(1),
        //         tile + 1,
        //     ];
        //     for tile in tiles {
        //         if tile > level.tiles.len() {
        //             break;
        //         }
        //         let object = &level.tiles[tile];
        //         if !(object.collision
        //             || object
        //                 .special_data
        //                 .iter()
        //                 .any(|f| *f == SpecialTileData::OutOfBounds))
        //         {
        //             without_collision.push(tile);
        //         }
        //     }
        //     without_collision
        // }
        // let mut min_x = tile % level.width;
        // let mut max_x = min_x;
        // let mut min_y = tile / level.width;
        // let mut max_y = min_y;
        // let mut tiles = vec![tile];
        // let mut checked = HashSet::new();
        // while !tiles.is_empty() {
        //     let mut buffer = Vec::new();
        //     tiles.retain_mut(|tile| {
        //         if checked.contains(tile) {
        //             return false;
        //         }
        //         checked.insert(*tile);
        //         min_x = min_x.min(*tile % level.width);
        //         max_x = max_x.max(min_x);
        //         min_y = min_y.min(*tile / level.width);
        //         max_y = max_y.max(*tile / level.width);
        //         for neighbour in valid_tiles_around(*tile, level) {
        //             buffer.push(neighbour);
        //         }
        //         return false;
        //     });
        //     tiles.append(&mut buffer);
        // }

        // Box::new(Self {
        //     cannon: Cannon::new(
        //         to_game_pos(
        //             level
        //                 .tiles
        //                 .iter()
        //                 .enumerate()
        //                 .find(|f| {
        //                     f.1.special_data
        //                         .iter()
        //                         .any(|f| *f == SpecialTileData::Cannon)
        //                 })
        //                 .unwrap()
        //                 .0,
        //             level,
        //         ) - vec2(0.0, ASSETS.cannon.get_size().y),
        //         level,
        //     ),
        //     incoming_rocket: None,
        //     fallings_enemeies: Vec::new(),
        //     attack_cooldowns: Vec::new(),
        //     catapult: load_pixel_map(&ASSETS.red_boss.get("catapult"), [61, 61, 61, 255]),
        //     crane: Self::get_crane(),
        //     actions: vec![(RedGuyPhase::Entry, 0.0)],
        //     pos: to_game_pos(tile, level),

        //     allowed_area: (
        //         vec2(min_x as f32 * TILE_SIZE, min_y as f32 * TILE_SIZE),
        //         vec2(max_x as f32 * TILE_SIZE, max_y as f32 * TILE_SIZE),
        //     ),
        // })
    }

    fn update(
        &mut self,
        map: &Level,
        enemies: &mut Vec<NewEnemy>,
        projectiles: &mut Vec<Projectile>,
        frame_time: f32,
        player: &Player,
        particles: &mut Vec<Particle>,
    ) {
        let params = DrawTextureParams {
            dest_size: Some(ASSETS.red_boss.get_size() * 2.0),
            ..Default::default()
        };
        let draw_pos = self.pos;
        let draw_first = ["sack", "idle"];
        for animation in draw_first {
            ASSETS
                .red_boss
                .get(animation)
                .play(draw_pos, Some(params.clone()));
        }
        // let mut animations = Vec::new();
        let mut new_actions = Vec::new();
        if is_key_down(KeyCode::F) {}
        self.attack_cooldowns.retain_mut(|f| {
            if f.1 > f.2 {
                return false;
            } else {
                f.1 += frame_time;
                true
            }
        });

        self.actions.retain_mut(|f| {
            f.1 += frame_time;
            match f.0 {
                RedGuyPhase::ShootRocket => {
                    self.incoming_rocket =
                        Some((self.pos + (vec2(38.0, 43.0) - vec2(3.0, 4.)) * 2.0, 0.0));

                    return false;
                }
                RedGuyPhase::Idle(point, duration) => {
                    self.pos = point + vec2(HOVER_RANGE.0 * f.1.sin(), HOVER_RANGE.1 * f.1.cos());
                    let attacks = [
                        RedGuyPhase::Load(RedGuy::rand_enemy()),
                        RedGuyPhase::ShootRocket,
                    ];
                    let attack = attacks[gen_range(0, attacks.len())];
                    if !self.attack_cooldowns.iter().any(|f| f.0 == attack) {
                        new_actions.push((attack, 0.0));
                        self.attack_cooldowns
                            .push((attack, 0.0, gen_range(5.0, 8.0)));
                    }

                    // if f.1 > duration {
                    //     return false;
                    // }
                }
                RedGuyPhase::Entry => {
                    if f.1 > 5.0 {
                        new_actions.push((
                            RedGuyPhase::MoveTo(Self::new_location(self.allowed_area), self.pos),
                            0.0,
                        ));
                        return false;
                    }
                }
                RedGuyPhase::MoveTo(destination, start) => {
                    if (self.pos - destination).element_sum().abs() < 50.0 {
                        new_actions.push((
                            RedGuyPhase::Idle(self.pos - vec2(0.0, 20.0), gen_range(4.0, 9.0)),
                            0.0,
                        ));
                        return false;
                    } else {
                        self.pos = start.lerp(destination, f.1 / 2.0)
                    }
                }

                RedGuyPhase::Load(enemy) => {
                    let duratio: f32 = self.crane.iter().map(|f| f.0).sum();
                    let lift_time = 1.0;
                    let plot = [
                        (0.0, vec2(0.0, 0.0)),
                        (lift_time, vec2(0.0, -10.0)),
                        ((duratio / 3.0, vec2(12.5, -10.0))),
                    ];
                    if f.1 > plot.iter().map(|f| f.0).sum() {
                        new_actions.push((RedGuyPhase::Shoot(enemy), 0.0));
                        return false;
                    } else {
                        let mut time = f.1;
                        for (index, p) in plot.iter().enumerate() {
                            if index > plot.len() - 2 {
                                panic!()
                            }

                            let next = plot[index + 1];
                            let next = (next.0, next.1 * 2.0);
                            let p = (p.0, p.1);
                            if time > next.0 {
                                time -= next.0;
                                continue;
                            }
                            let pos = p.1.lerp(next.1, time / next.0)
                                + vec2(102.0, 35.0) * 2.0
                                + self.pos;
                            draw_texture(enemy.default_texture(), pos.x, pos.y, WHITE);
                            let mut time = f.1 + lift_time;
                            let mut crane_pos = Vec2::ZERO;
                            for (index, p) in self.crane.iter().enumerate() {
                                if time < p.0 + lift_time {
                                    crane_pos = p.1;

                                    break;
                                } else {
                                    time -= p.0;
                                }
                            }
                            crane_pos = crane_pos.floor();
                            ASSETS.red_boss.get("crane").play_with_clock(
                                self.pos,
                                f.1,
                                Some(params.clone()),
                            );
                            draw_line(
                                pos.x + enemy.default_texture().width() / 2.0,
                                pos.y + 5.0,
                                (crane_pos.x) * 2.0 + self.pos.x,
                                self.pos.y + 2.0 * crane_pos.y,
                                2.,
                                BROWN,
                            );
                            break;
                        }
                    }
                }
                RedGuyPhase::Shoot(enemy) => {
                    let duration = self.catapult.iter().map(|f| f.0).sum::<f32>();

                    let mut time = f.1;
                    let mut catapult_pos = self.catapult.last().unwrap().1;
                    for (index, p) in self.catapult.iter().enumerate() {
                        if time < p.0 {
                            catapult_pos = p.1;
                            break;
                        } else {
                            time -= p.0;
                        }
                    }
                    ASSETS.red_boss.get("catapult").play_with_clock(
                        self.pos,
                        f.1,
                        Some(params.clone()),
                    );
                    let texture = enemy.default_texture();
                    let size =
                        texture.size() / 1.5 + ((f.1 / duration) * texture.size() / 2.0) / 2.0;
                    let pos = vec2(
                        self.pos.x + catapult_pos.x * 2.0 - size.x / 2.0 + 2.0,
                        self.pos.y + catapult_pos.y * 2.0 - size.y / 2.0,
                    );
                    if f.1 > duration {
                        self.fallings_enemeies.push((enemy, pos, 0.0));
                        return false;
                    }
                    draw_texture_ex(
                        texture,
                        pos.x,
                        pos.y,
                        WHITE,
                        DrawTextureParams {
                            dest_size: Some(vec2(texture.width(), texture.height())),
                            ..Default::default()
                        },
                    );
                }
                _ => todo!(),
            }
            return true;
        });

        let draw_after = vec!["wings_bot", "bag_edge", "sack_bot"];

        for animation in draw_after {
            ASSETS
                .red_boss
                .get(animation)
                .play(draw_pos, Some(params.clone()));
        }
        if let Some(cooldown) = &mut self
            .attack_cooldowns
            .iter()
            .find(|f| f.0 == RedGuyPhase::ShootRocket)
        {
            if cooldown.1 > cooldown.2 / 2.0 {
                ASSETS
                    .red_boss
                    .get("rocket")
                    .play(self.pos, Some(params.clone()));
            }
        } else {
            ASSETS
                .red_boss
                .get("rocket")
                .play(self.pos, Some(params.clone()));
        }
        self.update_cannon(frame_time, player);

        if let Some(shot) = &mut self.cannon.shot {
            dbg!(shot.pos);
            let boss_size = ASSETS.red_boss.get_size();
            if shot.pos.x > self.pos.x
                && shot.pos.x < self.pos.x + boss_size.x
                && shot.pos.y > self.pos.y
                && shot.pos.y < self.pos.y + boss_size.y
            {
                particles.push(Particle::new(
                    Box::new(|f| ASSETS.cannon_shot_particle.play(f, None)),
                    particles::Lifetime::ByTime(ASSETS.cannon_shot_particle.get_duration()),
                    None,
                    vec2(shot.pos.x - ASSETS.cannon_shot_particle.get_size().x, 0.0),
                ));
                self.cannon.shot = None;
            } else {
                shot.past_pos.push(self.pos);
                shot.pos +=
                    ((self.pos + boss_size / 2.0) - shot.pos).normalize() * shot.speed * frame_time;

                for (index, pos) in shot.past_pos.iter().enumerate() {
                    if index == shot.past_pos.len() - 1 {
                        break;
                    }
                    let next_pos = shot.past_pos[index + 1];
                    draw_line(
                        pos.x.ceil(),
                        pos.y.ceil(),
                        next_pos.x.ceil(),
                        next_pos.y.ceil(),
                        2.0,
                        YELLOW,
                    );
                }
            }
        }
        self.fallings_enemeies.retain_mut(|enemy| {
            enemy.2 += frame_time;
            let func = -(-170. * enemy.2.powi(2) + 261.5 * enemy.2);
            let heigth = func + enemy.1.y;

            let pos = vec2(enemy.1.x, heigth);
            if check_collision(pos, map) && func.is_sign_positive() {
                let pos = vec2(pos.x, (pos.y / 16.0).floor() * 16.0 - 16.0);
                // enemies.push(enemy.0.spawn(pos, map));
                return false;
            }

            // draw_texture(enemy.0.default_texture(), pos.x, pos.y, WHITE);
            return true;
        });
        self.actions.append(&mut new_actions);
        if let Some(rocket) = &mut self.incoming_rocket {
            dbg!(&rocket);
            rocket.1 += frame_time;
            let animation = ASSETS.red_boss.get("rocket_enter");
            if rocket.1 > animation.get_duration() {
                self.incoming_rocket = None;
                projectiles.push(Projectile::from(
                    self.pos + vec2(38.0, 43.0) * 2.0,
                    crate::projectiles::Projectiles::Rocket,
                    None,
                ));
            } else {
                animation.play_with_clock(rocket.0, rocket.1, None);
            }
        }
    }
}
