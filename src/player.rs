use crate::{
    assets::ASSETS,
    enemies::{self, Bullet, Enemy, Projectile, check_collision_with_size},
    level::{Layer, Level, MAP_SCALE_FACTOR, TILE_SIZE},
    particles::Particle,
    utils::{Animation, *},
};
use macroquad::prelude::*;

pub struct Player {
    pub hp: u32,
    pub pos: Vec2,
    pub size: Vec2,
    velocity: Vec2,
    grounded: bool,
    speed: f32,
    current_top_animation: Option<(&'static Animation, f32)>,
    previous_flipped: bool,
    iframes: Option<f32>,
}
const AIR_DRAG: f32 = 0.3;
const FRICITON: f32 = 0.93;
const GRAVITY: f32 = 7.;
impl Player {
    pub fn damage(&mut self, dmg: u32) {
        if self.iframes.is_none() {
            self.iframes = Some(3.0);
            self.hp = self.hp.saturating_sub(dmg);
        }
    }
    pub fn knockback(&mut self, point: Vec2, strength: f32) {
        self.velocity += strength * ((self.pos + self.size / 2.0) - point).normalize_or_zero()
    }
    pub fn new(pos: Vec2) -> Self {
        Self {
            iframes: None,
            hp: 100,
            previous_flipped: true,
            current_top_animation: None,
            grounded: false,
            size: vec2(
                TILE_SIZE * MAP_SCALE_FACTOR,
                TILE_SIZE * MAP_SCALE_FACTOR * 2.0,
            ),
            pos,

            velocity: Vec2::ZERO,
            speed: 10.0,
        }
    }

    pub fn update(
        &mut self,
        map: &Level,
        projectiles: &mut Vec<Box<dyn Projectile>>,
        enemies: &mut Vec<Box<dyn Enemy>>,
        particles: &mut Vec<Particle>,
    ) {
        if let Some(iframes) = &mut self.iframes {
            if *iframes > 0.0 {
                *iframes -= get_frame_time();
            } else {
                self.iframes = None;
            }
        }

        let mut top_animation: &Animation = &ASSETS.top_player_animations.idle;
        let mut params = DrawTextureParams {
            flip_x: self.previous_flipped,

            ..Default::default()
        };
        let mut bot_animation: &Animation = &ASSETS.bottom_player_animations.idle;
        let mut direction = 0.0;
        if is_key_down(KeyCode::A) {
            params.flip_x = true;
            direction = -1.0;
            bot_animation = &ASSETS.bottom_player_animations.walk;
        }
        if is_key_down(KeyCode::D) {
            direction = 1.0;
            params.flip_x = false;
            bot_animation = &ASSETS.bottom_player_animations.walk;
        }

        if self.grounded {
            self.velocity.x += direction * self.speed;
            if is_key_pressed(KeyCode::Space) {
                self.velocity.y = -220.0;
            }
        } else {
            self.velocity.x = (self.velocity.x + direction * self.speed) * FRICITON;
        }

        let collision_points = [
            (0.0, 0.0),
            (self.size.x, 0.0),
            (0.0, self.size.y / 2.0),
            (self.size.x, self.size.y / 2.0),
            (0.0, self.size.y),
            (self.size.x, self.size.y),
        ];
        self.velocity.y += GRAVITY;
        self.grounded = false;
        for (index, point) in collision_points.iter().enumerate() {
            enemies.retain_mut(|f| {
                let bounds = f.get_bounds();
                let collision = check_collision_with_size(
                    (
                        self.pos + vec2(point.0, point.1) + self.velocity * get_frame_time(),
                        Vec2::ZERO,
                    ),
                    bounds,
                );
                if collision {
                    if self.pos.y + self.size.y < bounds.0.y && f.on_jumped_on_by_player() {
                        self.velocity.y = -200.0;
                        particles.push(Particle::new(
                            Box::new(|f| ASSETS.blood.play(f, None)),
                            crate::particles::Lifetime::ByTime(ASSETS.blood.get_duration()),
                            None,
                            self.pos,
                        ));
                        return false;
                    } else {
                        if let Some((knockback_origin, knockback_strenght, damage)) =
                            f.on_player_contact(particles)
                        {
                            self.knockback(knockback_origin, knockback_strenght);
                            self.damage(damage);
                        }
                    }
                    //  else {
                    //     let clamped = self.pos.x
                    //         != self
                    //             .pos
                    //             .x
                    //             .clamp(bounds.0.x - point.0, bounds.0.x + bounds.1.x - point.0);
                    //     if clamped {
                    //         self.pos.x = self
                    //             .pos
                    //             .x
                    //             .clamp(bounds.0.x - point.0, bounds.0.x + bounds.1.x - point.0);
                    //         self.velocity.x = 200.0
                    //             * if self.pos.x == bounds.0.x - point.0 {
                    //                 -1.0
                    //             } else if self.pos.x == bounds.0.x + bounds.1.x - point.0 {
                    //                 1.0
                    //             } else {
                    //                 0.0
                    //             };
                    //     }
                }
                return true;
            });

            // SO JANK PLS FORGIVE
            let map_pos = (self.pos + self.velocity * get_frame_time() + vec2(point.0, point.1))
                / (TILE_SIZE * MAP_SCALE_FACTOR);

            let mut tile_no = map_pos.y as usize * map.width as usize + map_pos.x as usize;
            if map_pos.x.floor() == map_pos.x && index % 2 == 1 {
                tile_no -= 1;
            }
            if tile_no > map.tiles.len() - 1 {
                println!("out of bounds");
                break;
            }
            let pottential_collider = &map.tiles[tile_no];

            if pottential_collider // DONT READ
                .data
                .iter()
                .any(|f| f.0 == Layer::Collision)
            {
                let x0 = map_pos.x.floor() * TILE_SIZE * MAP_SCALE_FACTOR - point.0;
                let x1 = (map_pos.x.floor() + 1.0) * MAP_SCALE_FACTOR * TILE_SIZE - point.0;
                let y0 = map_pos.y.floor() * TILE_SIZE * MAP_SCALE_FACTOR - point.1;
                let y1 = (map_pos.y.floor() + 1.0) * MAP_SCALE_FACTOR * TILE_SIZE - point.1;
                let mut clamped_x = false;
                let wa = if self.pos.x == x0 { true } else { false };
                if index < 4 || self.pos.y != y0 {
                    self.pos.x = self.pos.x.clamp(x0, x1);
                    if self.pos.x == x0 || self.pos.x == x1 && !wa {
                        clamped_x = true;
                        self.velocity.x = 0.0;
                    }
                }

                self.pos.y = self.pos.y.clamp(y0, y1);
                if self.pos.y == y0 && !clamped_x {
                    self.velocity.y = 0.0;
                    self.grounded = true;
                } else if self.pos.y == y1 {
                    self.velocity.y = 0.0;
                }
            } else if index > 3 {
                if pottential_collider
                    .data
                    .iter()
                    .any(|f| f.0 == Layer::OneWayCollision)
                {
                    if self.velocity.y.is_sign_positive() {
                        self.velocity.y = 0.0;
                        self.grounded = true;
                    }
                }
            } else {
                if pottential_collider.data.iter().any(|f| f.0 == Layer::Death) {
                    //Die
                }
            }
        }
        if self.grounded {
            self.velocity.x = self.velocity.x * FRICITON;
        }
        let shader = self.iframes.is_some() && (get_time() * 8.0).sin().is_sign_negative();

        if shader {
            gl_use_material(&IFRAMES_MATERIAL);
        }
        if let Some((current_top_animation, animation_clock)) = &mut self.current_top_animation {
            if current_top_animation.1 as f32 / 1000.0 < *animation_clock {
                self.current_top_animation = None;
                top_animation.play(self.pos, Some(params.clone()));
            } else {
                current_top_animation.play_with_clock(
                    self.pos,
                    *animation_clock,
                    Some(params.clone()),
                );
                *animation_clock += get_frame_time();
            }
        } else {
            top_animation.play(self.pos, Some(params.clone()));
        };

        bot_animation.play(self.pos, Some(params.clone()));

        self.pos += self.velocity * get_frame_time();
        if is_key_pressed(KeyCode::F) {
            self.current_top_animation = Some((&ASSETS.top_player_animations.shoot, 0.0));
            projectiles.push(Box::new(Bullet::new(
                self.pos
                    + vec2(
                        if !params.flip_x {
                            self.size.x - 2.0
                        } else {
                            2.0
                        },
                        14.0,
                    ),
                vec2(if !params.flip_x { 1.0 } else { -1.0 }, 0.0),
            )));
        }
        self.previous_flipped = params.flip_x;
        if shader {
            gl_use_default_material();
        }
    }
}
