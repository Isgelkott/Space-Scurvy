use crate::{
    assets::ASSETS,
    level::{Layer, Level, MAP_SCALE_FACTOR, TILE_SIZE, Tile},
    utils::{Animation, *},
};
use macroquad::prelude::*;

fn check_collision(pos: Vec2, map: &Level) -> Option<Vec2> {
    let map_pos = pos / (TILE_SIZE * MAP_SCALE_FACTOR);
    if map_pos.y as usize * map.width as usize + map_pos.x as usize > map.tiles.len() - 1 {
        return None;
    }
    let pottential_collider =
        &map.tiles[map_pos.y as usize * map.width as usize + map_pos.x as usize];
    if pottential_collider
        .data
        .iter()
        .any(|f| f.0 == Layer::Collision)
    {
        let x0 = map_pos.x.floor() * TILE_SIZE * MAP_SCALE_FACTOR;
        let x1 = (map_pos.x.floor() + 1.0) * MAP_SCALE_FACTOR * TILE_SIZE;
        let y0 = map_pos.y.floor() * TILE_SIZE * MAP_SCALE_FACTOR;
        let y1 = map_pos.y.ceil() * MAP_SCALE_FACTOR * TILE_SIZE;
        Some(vec2(pos.x.clamp(x0, x1), pos.y.clamp(y0, y1)))
    } else {
        None
    }
}
pub struct Player {
    pub pos: Vec2,
    size: Vec2,
    velocity: Vec2,
    grounded: bool,
    speed: f32,
    current_top_animation: Option<(&'static Animation, f32)>,
    previous_flipped: bool,
}
const AIR_DRAG: f32 = 0.8;
const GRAVITY: f32 = 3.5;
impl Player {
    pub fn new(pos: Vec2) -> Self {
        Self {
            previous_flipped: true,
            current_top_animation: None,
            grounded: false,
            size: vec2(
                TILE_SIZE * MAP_SCALE_FACTOR,
                TILE_SIZE * MAP_SCALE_FACTOR * 2.0,
            ),
            pos: pos,

            velocity: Vec2::ZERO,
            speed: 100.0,
        }
    }

    pub fn update(&mut self, map: &Level) {
        let mut top_animation: &Animation = &ASSETS.top_player_animations.idle;
        let mut params = DrawTextureParams {
            flip_x: self.previous_flipped,

            ..Default::default()
        };
        let mut bot_animation: &Animation = &ASSETS.bottom_player_animations.idle;
        let mut direction = Vec2::ZERO;
        if is_key_down(KeyCode::A) {
            params.flip_x = true;
            direction.x = -1.0;
            bot_animation = &ASSETS.bottom_player_animations.walk;
        }
        if is_key_down(KeyCode::D) {
            direction.x = 1.0;
            params.flip_x = false;
            bot_animation = &ASSETS.bottom_player_animations.walk;
        }
        if is_key_down(KeyCode::S) {
            direction.y = 1.0;
        }
        if is_key_down(KeyCode::W) {
            direction.y = -1.0;
        }

        if self.grounded {
            self.velocity = direction.normalize_or_zero() * self.speed;
            if is_key_pressed(KeyCode::Space) {
                self.velocity.y = -200.0;
            }
        } else {
            self.velocity.x = direction.normalize_or_zero().x * self.speed * AIR_DRAG;
        }
        // draw_rectangle(self.pos.x, self.pos.y, self.size.x, self.size.y, WHITE);

        let collision_points = [
            (0.0, 0.0),
            (self.size.x, 0.0),
            (0.0, self.size.y / 2.0),
            (self.size.x, self.size.y / 2.0),
            (0.0, self.size.y),
            (self.size.x, self.size.y),
        ];
        self.velocity.y += GRAVITY;
        let mut grounded = false;
        for (index, point) in collision_points.iter().enumerate() {
            let map_pos = (self.pos + self.velocity * get_frame_time() + vec2(point.0, point.1))
                / (TILE_SIZE * MAP_SCALE_FACTOR);

            let tile_no = map_pos.y as usize * map.width as usize + map_pos.x as usize;
            if tile_no > map.tiles.len() - 1 {
                println!("out of bounds");
                break;
            }
            let pottential_collider = &map.tiles[tile_no];

            if pottential_collider // jank af code but could not bother
                .data
                .iter()
                .any(|f| f.0 == Layer::Collision)
            {
                println!("coolid w/ {}", tile_no);
                let x0 = map_pos.x.floor() * TILE_SIZE * MAP_SCALE_FACTOR - point.0;
                let x1 = (map_pos.x.floor() + 1.0) * MAP_SCALE_FACTOR * TILE_SIZE - point.0;
                let y0 = map_pos.y.floor() * TILE_SIZE * MAP_SCALE_FACTOR - point.1;
                let y1 = (map_pos.y.floor() + 1.0) * MAP_SCALE_FACTOR * TILE_SIZE - point.1;
                let mut clamped_x = false;
                if index < 4 {
                    println!("clampin with {}", index);
                    let wa = if self.pos.x == x0 { true } else { false };
                    self.pos.x = self.pos.x.clamp(x0, x1);
                    if self.pos.x == x0 || self.pos.x == x1 && !wa {
                        clamped_x = true;
                        self.velocity.x = 0.0;
                    }
                } else if self.pos.y != y0 {
                    {
                        println!("clampin with {}", index);
                        self.pos.x = self.pos.x.clamp(x0, x1);
                        if self.pos.x == x0 || self.pos.x == x1 {
                            clamped_x = true;

                            self.velocity.x = 0.0;
                        }
                    }
                }
                if index > 1 {
                    self.pos.y = self.pos.y.clamp(y0, y1);
                    if self.pos.y == y0 && !clamped_x {
                        self.velocity.y = 0.0;
                        println!("grounded");
                        grounded = true;
                    }
                }
            }
        }
        self.grounded = grounded;

        if let Some((top_animation, animation_clock)) = &mut self.current_top_animation {
            if !top_animation.play_with_clock(self.pos, Some(params.clone()), animation_clock) {
                top_animation.play(self.pos, Some(params.clone()));
            }
        } else {
            top_animation.play(self.pos, Some(params.clone()));
        };

        bot_animation.play(self.pos, Some(params.clone()));
        self.pos += self.velocity * get_frame_time();
        self.previous_flipped = params.flip_x;
    }
}
