use std::f32::consts::PI;

use macroquad::{prelude::*, rand::gen_range};

use crate::{
    assets::ASSETS,
    enemies::{NewEnemy, PresetEnemies},
    level::{Level, TILE_SIZE},
    particles::{self, Particle},
    player::Player,
    projectiles::Projectile,
    utils::*,
};

const PAD_TIME: f32 = 10.;
const PAD_COOLDOWN: f32 = 0.;
const PAD_EXPIRE: f32 = 14.;
const SHOOT_DURATION: f32 = 0.4;
const HOVER_RANGE: f32 = 20.;
#[derive(Debug, PartialEq, Clone, Copy)]
enum RedGuyPhase {
    ShootRocket,
    Idle(Vec2, f32),
    MoveTo(Vec2, Vec2),
    Load(PresetEnemies),
    Shoot(PresetEnemies),
    Entry,
}
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
    shot: Option<f32>,
}

impl Cannon {
    fn cooldown() -> f32 {
        gen_range(10.0, 12.0)
    }
    fn new() -> Self {
        Self {
            shot: None,
            angle: 0.0,
            pos: vec2(69., 4.) * TILE_SIZE - vec2(0., ASSETS.cannon.size().y),
            clock: 0.0,
            action: CannonActions::Idle,
        }
    }
}
struct CannonShot {
    pos: Vec2,
    dest: Vec2,
    clock: f32,
}
impl CannonShot {
    fn update(&mut self, frame_time: f32) {
        self.clock += frame_time;
    }
    fn new(pos: Vec2, dest: Vec2) -> Self {
        Self {
            pos,
            dest,
            clock: 0.,
        }
    }
}
enum PadAction {
    Die,
    Pressed,
}
struct Pad {
    pos: Vec2,
    clock: f32,
}
impl Pad {
    fn new(pos: Vec2) -> Self {
        Self { pos, clock: 0. }
    }
    fn update(&mut self, frame_time: f32, player: &Player) -> Option<PadAction> {
        self.clock += frame_time;
        if self.clock >= PAD_EXPIRE {
            Some(PadAction::Die);
        }
        if self.clock < 5. {
            ASSETS.pad.get("help").play(self.pos, None);
        } else {
            ASSETS.pad.play_tag("base", self.pos, None);
        }
        if check_collision_rectangle_collision(
            (player.pos, player.size),
            (
                self.pos + vec2(0., ASSETS.pad.size().y - 4.0),
                vec2(ASSETS.pad.size().x, 4.),
            ),
        ) {
            return Some(PadAction::Pressed);
        }
        return None;
    }
}
enum PadState {
    Pad(Pad),
    Timer(f32),
}
pub struct RedGuy {
    pos: Vec2,
    crane: Vec<(f32, Vec2)>,
    catapult: Vec<(f32, Vec2)>,
    allowed_area: (Vec2, Vec2),
    actions: Vec<(RedGuyPhase, f32)>,
    fallings_enemeies: Vec<(PresetEnemies, Vec2, f32)>,
    attack_cooldowns: Vec<(RedGuyPhase, f32, f32)>,
    incoming_rocket: Option<(Vec2, f32)>,
    cannon: Cannon,
    pad: PadState,
    pub lives: u8,
    active: bool,
}
impl RedGuy {
    fn update_cannon(&mut self, frame_time: f32, player: &Player) {
        self.cannon.clock += frame_time;
        let direction = (self.pos - self.cannon.pos).normalize();
        let stand_animation;
        let barrel_animation;
        let center =
            self.cannon.pos + vec2(65., 14.) - vec2(ASSETS.cannon_barrel.size().x / 2.0, 0.0);

        let boss_center = vec2(
            self.pos.x + ASSETS.red_boss.size().x,
            self.pos.y + ASSETS.red_boss.size().y,
        );
        let desired_angle = (self.pos + ASSETS.red_boss.size() / 2.0 - center);
        let difference =
            desired_angle.angle_between(vec2(self.cannon.angle.cos(), self.cannon.angle.sin()));
        if (difference - PI).abs() < 0.2 {
        } else {
            self.cannon.angle += difference.signum() * frame_time;
        }

        let shoot_point = vec2(
            center.x
                + -(ASSETS.cannon_barrel.size().y) * self.cannon.angle.cos()
                + (self.cannon.angle - PI / 2.).cos() * -8.,
            center.y
                + -(ASSETS.cannon_barrel.size().y) * self.cannon.angle.sin()
                + (self.cannon.angle - PI / 2.).sin() * -8.,
        );

        stand_animation = ASSETS.cannon.get("idle");
        barrel_animation = ASSETS.cannon_barrel.get("cooldown");

        stand_animation.play(self.cannon.pos, None);
        barrel_animation.play(
            center,
            Some(DrawTextureParams {
                rotation: self.cannon.angle + PI / 2.0,
                pivot: Some(center + vec2(ASSETS.cannon_barrel.size().x / 2.0, 0.0)),

                ..Default::default()
            }),
        );
        if let Some(duration) = &mut self.cannon.shot {
            *duration -= frame_time;
            draw_line(
                shoot_point.x,
                shoot_point.y,
                boss_center.x,
                boss_center.y,
                8.0,
                Color::from_hex(0xffeb57),
            );
            if duration.is_sign_negative() {
                self.cannon.shot = None;
            }
        }
    }

    fn new_location(allowed_area: (Vec2, Vec2)) -> Vec2 {
        vec2(
            gen_range(
                allowed_area.0.x,
                allowed_area.1.x - 2.0 * ASSETS.red_boss.size().x,
            ),
            gen_range(
                allowed_area.0.y - TILE_SIZE * 5.0,
                allowed_area.1.y - ASSETS.red_boss.size().y - TILE_SIZE * 4.,
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
    pub fn new(pos: Vec2) -> Self {
        Self {
            active: false,
            lives: 3,
            pad: PadState::Timer(PAD_COOLDOWN),
            cannon: Cannon::new(),
            incoming_rocket: None,
            fallings_enemeies: Vec::new(),
            attack_cooldowns: Vec::new(),
            catapult: load_pixel_map(&ASSETS.red_boss.get("catapult"), [61, 61, 61, 255]),
            crane: Self::get_crane(),
            actions: vec![(RedGuyPhase::Entry, 0.0)],
            pos: TILE_SIZE * vec2(84., 3.),

            allowed_area: (vec2(66., 1.) * TILE_SIZE, vec2(116., 9. as f32) * TILE_SIZE),
        }
    }
    fn gen_pad_pos(&self) -> Vec2 {
        let x = gen_range(self.allowed_area.0.x, self.allowed_area.1.x);
        return vec2(x, self.allowed_area.1.y - TILE_SIZE);
    }
    pub fn update(
        &mut self,
        map: &Level,
        enemies: &mut Vec<NewEnemy>,
        projectiles: &mut Vec<Projectile>,
        frame_time: f32,
        player: &Player,
        particles: &mut Vec<Particle>,
    ) {
        if player.pos.x > 55. * TILE_SIZE {
            self.active = true;
        }
        if !self.active {
            return;
        }
        let params = DrawTextureParams {
            dest_size: Some(ASSETS.red_boss.size() * 2.0),
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
        self.attack_cooldowns.retain_mut(|f| {
            if f.1 > f.2 {
                return false;
            } else {
                f.1 += frame_time;
                true
            }
        });
        if self.cannon.shot.is_some() {
            gl_use_material(&BOSS_DAMAGE_MAT);
        }
        self.actions.retain_mut(|f| {
            f.1 += frame_time;
            match f.0 {
                RedGuyPhase::ShootRocket => {
                    self.incoming_rocket =
                        Some((self.pos + (vec2(38.0, 43.0) - vec2(3.0, 4.)) * 2.0, 0.0));

                    return false;
                }
                RedGuyPhase::Idle(point, duration) => {
                    self.pos = point + vec2(HOVER_RANGE * f.1.sin(), HOVER_RANGE * f.1.cos());
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
        if self.cannon.shot.is_some() {
            gl_use_default_material();
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

        self.fallings_enemeies.retain_mut(|enemy| {
            enemy.2 += frame_time;
            let func = -(-170. * enemy.2.powi(2) + 261.5 * enemy.2);
            let heigth = func + enemy.1.y;

            let pos = vec2(enemy.1.x, heigth);
            if map.is_collider(pos) && func.is_sign_positive() {
                let pos = vec2(pos.x, (pos.y / 16.0).floor() * 16.0 - 16.0);
                enemies.push(NewEnemy::new(enemy.0, pos, map));
                return false;
            }

            draw_texture(enemy.0.default_texture(), pos.x, pos.y, WHITE);
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
                    Vec2::ZERO,
                ));
            } else {
                animation.play_with_clock(rocket.0, rocket.1, None);
            }
        };
        if self.cannon.shot.is_some() {
            gl_use_default_material();
        }
        match &mut self.pad {
            PadState::Timer(clock) => {
                *clock -= frame_time;
                if clock.is_sign_negative() {
                    self.pad = PadState::Pad(Pad::new(self.gen_pad_pos()))
                }
            }
            PadState::Pad(pad) => {
                dbg!(pad.pos);
                if let Some(action) = pad.update(frame_time, player) {
                    match action {
                        PadAction::Die => {}
                        PadAction::Pressed => {
                            self.cannon.shot = Some(SHOOT_DURATION);
                            self.lives -= 1;
                        }
                    }
                    self.pad = PadState::Timer(PAD_TIME);
                }
            }
        }
    }
}
