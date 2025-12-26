use crate::{
    level::{Layer, Level, MAP_SCALE_FACTOR, TILE_SIZE, Tile},
    utils::{Animation, *},
};
use macroquad::prelude::*;
struct PlayerAnimations {
    idle: Animation,
    walk: Animation,
}
impl PlayerAnimations {
    fn new() -> Self {
        let data = include_bytes!("../assets/pirate.aseprite");
        Self {
            idle: load_animation_from_tag(data, "walk"),
            walk: load_animation_from_tag(data, "walk"),
        }
    }
}
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
    direction: Vec2,
    grounded: bool,
    animations: PlayerAnimations,
}
impl Player {
    pub fn new(pos: Vec2) -> Self {
        let animations = PlayerAnimations::new();
        Self {
            grounded: false,
            size: vec2(
                animations.walk.0[0].0.width(),
                animations.walk.0[0].0.height(),
            ),
            pos: pos,
            animations,
            direction: Vec2::ZERO,
        }
    }
    fn movement(&mut self) {
        let mut animation = &self.animations.idle;
        if self.grounded {
            if is_key_down(KeyCode::A) {
                self.direction.x = -1.0;
                animation = &self.animations.walk;
            }
            if is_key_down(KeyCode::D) {
                self.direction.x = 1.0;
                animation = &self.animations.walk;
            }
        }
        draw_rectangle(self.pos.x, self.pos.y, self.size.x, self.size.y, WHITE);

        let mut time = (get_time() * 1000.0) % animation.1 as f64;
        for i in &animation.0 {
            if time <= i.1 as f64 {
                draw_texture_ex(
                    &i.0,
                    self.pos.x,
                    self.pos.y,
                    WHITE,
                    DrawTextureParams {
                        ..Default::default()
                    },
                );
                break;
            } else {
                time -= i.1 as f64;
            }
        }
    }
    fn collision(&mut self, map: &Level) {
        let collision_points = [
            (0.0, 0.0),
            (self.size.x, 0.0),
            (0.0, self.size.y),
            (self.size.x, self.size.y),
        ];
        for point in collision_points.iter() {
            let collision =
                check_collision(self.pos + self.direction + vec2(point.0, point.1), map);
            if let Some(new_pos) = collision {
                // self.pos = new_pos
            }
        }
    }
    pub fn update(&mut self, map: &Level) {
        self.movement();
        self.collision(map);
    }
}
