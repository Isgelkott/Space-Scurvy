use std::collections::HashSet;

use crate::{
    CameraHolder,
    assets::ASSETS,
    enemies::NewEnemy,
    level::*,
    particles::Particle,
    projectiles::Projectile,
    utils::{Animation, *},
};
use macroquad::prelude::*;
pub struct Bullet {
    pub pos: Vec2,
    pub size: Vec2,
    origin: Vec2,
    pub direction: f32,
}
impl Bullet {
    pub fn update(&mut self, frame_time: f32, camera: &CameraHolder, level: &Level) -> bool {
        const BULLET_SPEED: f32 = 480.;
        self.pos.x += self.direction * BULLET_SPEED * frame_time;
        if camera.is_obj_in_view(self.pos) {
            ASSETS.bullet.base().play(self.pos, None);
        }

        return !level.is_collider(self.pos);
    }
    pub fn new(pos: Vec2, direction: f32) -> Self {
        let pos = pos + vec2(0., 16.);
        let size = ASSETS.bullet.size();

        Self {
            pos,
            origin: pos,
            direction,
            size,
        }
    }
}
pub struct Player {
    pub hp: u32,
    pub pos: Vec2,
    pub size: Vec2,
    pub velocity: Vec2,
    pub grounded: bool,
    speed: f32,
    current_top_animation: Option<(&'static Animation, f32)>,
    pub previous_flipped: bool,
    iframes: Option<f32>,
    pub death: Option<(DeathCause, f32)>,
    last_pos: Vec2,
    pub ammo: u8,
}
const FRICITON: f32 = 1.0;
pub const GRAVITY: f32 = 900.;
#[derive(Clone, Copy, Debug)]
pub enum DeathCause {
    Acid,
    Default,
    Energy,
    Explode,
}
pub const GUN_ANIMATION_LENGHT: f32 = 1.;
impl Player {
    pub fn center(&self) -> Vec2 {
        self.pos + self.size / 2.
    }
    pub fn damage(&mut self, dmg: u32, death_cause: DeathCause) {
        if self.death.is_none() {
            if self.iframes.is_none() {
                self.iframes = Some(3.0);
                self.hp = self.hp.saturating_sub(dmg);
                if self.hp == 0 {
                    self.death = Some((death_cause, 0.0));
                }
            }
        }
    }
    pub fn knockback(&mut self, point: Vec2, strength: f32) {
        self.velocity += strength * ((self.pos + self.size / 2.0) - point).normalize_or_zero()
    }
    pub fn new(pos: Vec2) -> Self {
        Self {
            ammo: 6,
            death: None,
            iframes: None,
            hp: 100,
            previous_flipped: true,
            current_top_animation: None,
            grounded: false,
            size: vec2(TILE_SIZE, TILE_SIZE * 2.0),
            pos,
            last_pos: pos,
            velocity: Vec2::ZERO,
            speed: 150.0,
        }
    }

    pub fn update(
        &mut self,
        level: &mut Level,
        projectiles: &mut Vec<Projectile>,
        enemies: &mut Vec<NewEnemy>,
        particles: &mut Vec<Particle>,
        frame_time: f32,
        camera: &mut CameraHolder,
        bullets: &mut Vec<Bullet>,
        gun_animation: &mut f32,
    ) {
        const JUMP_HEIGHT: f32 = -320.0;

        if let Some(death) = &mut self.death {
            death.1 += frame_time;
        } else {
            if let Some(iframes) = &mut self.iframes {
                if *iframes > 0.0 {
                    *iframes -= frame_time;
                } else {
                    self.iframes = None;
                }
            }

            let top_animation: &Animation = &ASSETS.player.get("idle_top");
            let mut params = DrawTextureParams {
                flip_x: self.previous_flipped,

                ..Default::default()
            };
            let mut bot_animation: &Animation = &ASSETS.player.get("idle_bot");
            let mut direction = 0.0;
            if is_key_down(KeyCode::A) {
                params.flip_x = true;
                direction = -1.0;
                bot_animation = &ASSETS.player.get("walk");
            }
            if is_key_down(KeyCode::D) {
                direction = 1.0;
                params.flip_x = false;
                bot_animation = &ASSETS.player.get("walk");
            }

            if self.grounded {
                self.velocity.x = direction * self.speed;
                if is_key_pressed(KeyCode::Space) {
                    self.velocity.y = JUMP_HEIGHT;
                }
            } else {
                self.velocity.x = direction * self.speed;
            }

            self.velocity.y += GRAVITY * frame_time;
            self.grounded = false;
            const HITBOX_SHRINK_AMOUNT: f32 = 4.;
            enemies.retain_mut(|enemy| {
                if enemy.die.is_some() {
                    return true;
                }
                let is_coliding = check_collision_rectangle_collision(
                    (self.pos, self.size),
                    (
                        enemy.pos + HITBOX_SHRINK_AMOUNT,
                        enemy.size - HITBOX_SHRINK_AMOUNT,
                    ),
                );
                if is_coliding {
                    if enemy.jumpable && self.last_pos.y + self.size.y < enemy.pos.y + 4. {
                        self.velocity.y = JUMP_HEIGHT;
                        enemy.kill();
                        return false;
                    } else {
                        self.knockback(enemy.pos + enemy.size / 2., 30.);
                        self.damage(15, DeathCause::Default);
                    }
                }
                return true;
            });

            for y in (0..(self.size.y / 16.0) as i16 + 1).rev() {
                let y = ((y * 16) as f32).min(self.size.y);
                for x in 0..((self.size.x / 16.0).ceil()) as i16 + 1 {
                    let x = ((x * 16) as f32).min(self.size.x - 1.0);
                    let point = (x, y);
                    let mut map_pos = (self.pos
                        + vec2(1.0, 0.0)
                        + vec2(0.0, self.velocity.y) * frame_time
                        + vec2(point.0, point.1));
                    if x != 0.0 && map_pos.x.fract() == 0.0 {
                        map_pos.x -= 1.0;
                    }
                    let tile = level.get_tile(map_pos);
                    if let Some(tile) = tile {
                        let tile_pos = floored_pos(map_pos);
                        if tile.collision {
                            if DEBUG_FLAGS.show_collisions {
                                dbg!(tile_pos);
                                draw_rectangle(tile_pos.x, tile_pos.y, 5.0, 5.0, BLUE);
                            }

                            self.pos.y = self
                                .pos
                                .y
                                .clamp(tile_pos.y - point.1, tile_pos.y + TILE_SIZE - point.1);
                            if self.pos.y == (tile_pos.y - point.1)
                                || self.pos.y == tile_pos.y + TILE_SIZE - point.1
                            {
                                self.grounded = true;

                                self.velocity.y = 0.;
                            }
                        } else {
                            if DEBUG_FLAGS.show_collisions {
                                draw_rectangle(tile_pos.x, tile_pos.y, 5.0, 5.0, RED);
                            }
                        }
                    }
                }
            }
            for y in (0..(self.size.y / 16.0) as i16 + 1).rev() {
                let y = ((y * 16) as f32).min(self.size.y - 1.0);
                for x in 0..((self.size.x / 16.0).ceil()) as i16 + 1 {
                    let x = ((x * 16) as f32).min(self.size.x);
                    let point = (x, y);
                    let mut map_pos = (self.pos
                        + vec2(self.velocity.x, 0.0) * frame_time
                        + vec2(point.0, point.1));
                    if x != 0.0 && map_pos.x.fract() == 0.0 {
                        map_pos.x -= 1.0;
                    }
                    let (tile) = level.get_tile(map_pos);

                    if let Some((tile)) = tile {
                        let tile_pos = floored_pos(map_pos);

                        if let Some(death_cause) = tile.death_cause
                            && self.death.is_none()
                        {
                            self.death = Some((death_cause, 0.0));
                        }
                        if tile.collision {
                            if DEBUG_FLAGS.show_collisions {
                                draw_rectangle(tile_pos.x, tile_pos.y, 5.0, 5.0, YELLOW);
                            }

                            let x1 = tile_pos.x - point.0;
                            let x2 = tile_pos.x + TILE_SIZE - point.0;
                            self.pos.x = if self.velocity.x.is_sign_positive() {
                                x1
                            } else {
                                x2
                            };

                            // self.pos.x = self.pos.x.clamp(x1, x2);
                            self.velocity.x = 0.;
                        }
                    } else {
                        dbg!("out of bounds :(");
                    }
                }
            }

            if self.grounded {
                camera.calculate_y_up(&self);
                if self.velocity.x.is_sign_positive() {
                    self.velocity.x = (self.velocity.x - FRICITON).max(0.0);
                } else {
                    self.velocity.x = (self.velocity.x + FRICITON).min(0.0);
                };
            }
            let shader = self.iframes.is_some() && (get_time() * 8.0).sin().is_sign_negative();

            if shader {
                gl_use_material(&IFRAMES_MATERIAL);
            }
            if is_key_pressed(KeyCode::F) && self.ammo > 0 {
                self.current_top_animation = Some((&ASSETS.player.get("shoot"), 0.0));
                bullets.push(Bullet::new(
                    self.pos,
                    if self.previous_flipped { -1. } else { 1. },
                ));
                *gun_animation = GUN_ANIMATION_LENGHT;
                self.ammo -= 1;
            }

            let jump_anim = ASSETS.player.get("jump");

            if !self.grounded {
                if self.velocity.y < JUMP_HEIGHT * 0.5 {
                    jump_anim.draw_index(self.pos, 0, Some(params.clone()));
                } else if self.velocity.y > 60.0 {
                    jump_anim.draw_index(self.pos, 2, Some(params.clone()));
                } else {
                    jump_anim.draw_index(self.pos, 1, Some(params.clone()));
                }
            } else {
                if let Some((current_top_animation, animation_clock)) =
                    &mut self.current_top_animation
                {
                    if current_top_animation.1 as f32 / 1000.0 < *animation_clock {
                        self.current_top_animation = None;
                        top_animation.play(self.pos, Some(params.clone()));
                    } else {
                        current_top_animation.play_with_clock(
                            self.pos,
                            *animation_clock,
                            Some(params.clone()),
                        );
                        *animation_clock += frame_time;
                    }
                } else {
                    top_animation.play(self.pos, Some(params.clone()));
                };

                bot_animation.play(self.pos, Some(params.clone()));
            }

            if !DEBUG_FLAGS.still {
                self.last_pos = self.pos;
                self.pos += self.velocity * frame_time;
            }

            self.previous_flipped = params.flip_x;
            if shader {
                gl_use_default_material();
            }
        }
    }
}
