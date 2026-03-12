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

pub struct Player {
    pub hp: u32,
    pub pos: Vec2,
    pub size: Vec2,
    pub velocity: Vec2,
    grounded: bool,
    speed: f32,
    current_top_animation: Option<(&'static Animation, f32)>,
    pub previous_flipped: bool,
    iframes: Option<f32>,
    pub death: Option<(DeathCause, f32)>,
}
const FRICITON: f32 = 1.0;
const GRAVITY: f32 = 900.;
#[derive(Clone, Copy)]
pub enum DeathCause {
    Acid,
    Default,
    Energy,
    Explode,
}
impl Player {
    pub fn damage(&mut self, dmg: Option<u32>, death_cause: DeathCause) {
        if self.death.is_none() {
            if let Some(dmg) = dmg {
                if self.iframes.is_none() {
                    self.iframes = Some(3.0);
                    self.hp = self.hp.saturating_sub(dmg);
                    if self.hp == 0 {
                        self.death = Some((death_cause, 0.0));
                    }
                }
            } else {
                self.death = Some((death_cause, 0.0));
            }
        }
    }
    pub fn knockback(&mut self, point: Vec2, strength: f32) {
        self.velocity += strength * ((self.pos + self.size / 2.0) - point).normalize_or_zero()
    }
    pub fn new(pos: Vec2) -> Self {
        Self {
            death: None,
            iframes: None,
            hp: 100,
            previous_flipped: true,
            current_top_animation: None,
            grounded: false,
            size: vec2(TILE_SIZE, TILE_SIZE * 2.0),
            pos,

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
            // while !to_check.is_empty() {
            //     to_check.retain_mut(|tile| {
            //         let tile = *tile;
            //         if !checked.contains(&tile)
            //             && let Some(trigger) = &mut pottential_collider.trigger
            //         {
            //             *trigger = true;
            //             let check = [
            //                 tile.saturating_sub(1),
            //                 tile + 1,
            //                 tile.saturating_sub(map.width),
            //                 tile + map.width,
            //             ];
            //             for i in check {
            //                 buffer.push(i);
            //             }
            //             checked.insert(tile);
            //             return false;
            //         } else {
            //             return false;
            //         }
            //     });
            //     to_check.append(&mut buffer);
            // }
            dbg!((self.velocity));

            for y in 0..(self.size.y / 16.0) as i16 + 1 {
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
                    dbg!(map_pos);
                    let (tile) = get_tile(map_pos, level);
                    if let Some((tile, tile_pos)) = tile {
                        if tile.collision {
                            if DEBUG_FLAGS.show_collisions {
                                dbg!(tile_pos);
                                draw_rectangle(tile_pos.x, tile_pos.y, 5.0, 5.0, BLUE);
                            }

                            dbg!("collid y");
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
                    } else {
                        dbg!("out of bounds :(");
                    }
                }
            }
            for y in 0..(self.size.y / 16.0) as i16 + 1 {
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
                    let (tile) = get_tile(map_pos, level);
                    if let Some((tile, tile_pos)) = tile {
                        if tile.collision {
                            if DEBUG_FLAGS.show_collisions {
                                draw_rectangle(tile_pos.x, tile_pos.y, 5.0, 5.0, YELLOW);
                            }

                            dbg!("collid x");
                            let x1 = tile_pos.x - point.0;
                            let x2 = tile_pos.x + TILE_SIZE - point.0;
                            if (x1 - self.pos.x).abs() < (x2 - self.pos.x).abs() {
                                self.pos.x = x1
                            } else {
                                self.pos.x = x2;
                            }
                            // self.pos.x = self.pos.x.clamp(x1, x2);
                            self.velocity.x = 0.;
                        }
                    } else {
                        dbg!("out of bounds :(");
                    }
                }
            }

            // if !self.grounded {
            //     if let Some(death_cause) = &pottential_collider.death_cause
            //         && self.death.is_none()
            //     {
            //         self.death = Some((*death_cause, 0.0));
            //     }
            // }
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
            if is_key_pressed(KeyCode::F) {
                self.current_top_animation = Some((&ASSETS.player.get("shoot"), 0.0));
                // projectiles.push(Box::new(Bullet::new(
                //     self.pos
                //         + vec2(
                //             if !params.flip_x {
                //                 self.size.x - 2.0
                //             } else {
                //                 2.0
                //             },
                //             14.0,
                //         ),
                //     vec2(if !params.flip_x { 1.0 } else { -1.0 }, 0.0),
                // )));
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
            // if self.velocity.x.abs() < 2. {
            //     self.velocity.x = 0.0
            // }
            // if self.velocity.y.abs() < 2. {
            //     self.velocity.y = 0.0
            // }
            if !DEBUG_FLAGS.still {
                self.pos += self.velocity * frame_time;
            }

            self.previous_flipped = params.flip_x;
            if shader {
                gl_use_default_material();
            }
        }
    }
}
