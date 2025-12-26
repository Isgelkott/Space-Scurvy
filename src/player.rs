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
    speed: f32,
}
impl Player {
    pub fn new(pos: Vec2) -> Self {
        let animations = PlayerAnimations::new();
        Self {
            grounded: false,
            size: vec2(16.0, 32.0),
            pos: pos,
            animations,
            direction: Vec2::ZERO,
            speed: 1.0,
        }
    }

    fn collision(&mut self, map: &Level) {
        let mut animation = &self.animations.idle;
        let mut direction = Vec2::ZERO;
        if is_key_down(KeyCode::A) {
            direction.x = -1.0;
            animation = &self.animations.walk;
        }
        if is_key_down(KeyCode::D) {
            direction.x = 1.0;
            animation = &self.animations.walk;
        }
        if is_key_down(KeyCode::S) {
            direction.y = 1.0;
        }
        if is_key_down(KeyCode::W) {
            direction.y = -1.0;
        }
        if is_key_down(KeyCode::Space) && self.grounded {
            self.direction.y = -200.0;
        }
        self.direction += direction.normalize_or_zero() * self.speed;
        draw_rectangle(self.pos.x, self.pos.y, self.size.x, self.size.y, WHITE);

        // let mut time = (get_time() * 1000.0) % animation.1 as f64;
        // for i in &animation.0 {
        //     if time <= i.1 as f64 {
        //         draw_texture_ex(
        //             &i.0,
        //             self.pos.x,
        //             self.pos.y,
        //             WHITE,
        //             DrawTextureParams {
        //                 ..Default::default()
        //             },
        //         );
        //         break;
        //     } else {
        //         time -= i.1 as f64;
        //     }
        // }
        let collision_points = [
            (0.0, 0.0),
            (self.size.x, 0.0),
            (self.size.x, self.size.y / 2.0),
            (0.0, self.size.y / 2.0),
            (0.0, self.size.y),
            (self.size.x, self.size.y),
        ];
        self.direction.y += 2.0;
        for (index, point) in collision_points.iter().enumerate() {
            let map_pos = (self.pos + self.direction * get_frame_time() + vec2(point.0, point.1))
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

                if index < 4 {
                    println!("clampin with {}", index);
                    self.pos.x = self.pos.x.clamp(x0, x1);
                    if self.pos.x == x0 || self.pos.x == x1 {
                        self.direction.x = 0.0;
                    }
                } else if self.pos.y != y0 {
                    {
                        println!("clampin with {}", index);
                        self.pos.x = self.pos.x.clamp(x0, x1);
                        if self.pos.x == x0 || self.pos.x == x1 {
                            self.direction.x = 0.0;
                        }
                    }
                }
                self.pos.y = self.pos.y.clamp(y0, y1);

                if self.pos.y == y0 {
                    self.direction.y = 0.0;
                    println!("grounded");
                    self.grounded = true;
                }
            }
        }
        self.pos += self.direction * get_frame_time();
    }
    pub fn update(&mut self, map: &Level) {
        self.collision(map);
    }
}
