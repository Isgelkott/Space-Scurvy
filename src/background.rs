use std::sync::LazyLock;

use macroquad::{math::Vec2, prelude::*, rand::gen_range, texture::Texture2D};

use crate::{
    assets::*,
    utils::{Animation, AnimationMethods},
};

struct BackgroundObject {
    display: &'static DisplayType,
    pos: Vec2,
    direction: Vec2,
    speed: f32,
}
impl BackgroundObject {
    fn new(pos: Vec2) -> Self {
        let rand = gen_range(0, ASSETS.background_objects.len());
        let object = &ASSETS.background_objects[rand];
        let top = gen_range(0, 2) == 1;

        Self {
            speed: gen_range(5.0, 10.0),
            display: object,
            pos,
            direction: (vec2(1.0, 1.0).normalize()),
        }
    }
    fn update(&mut self) {
        let mut size;
        match &self.display {
            DisplayType::Animation(animation) => {
                size = animation.get_size();
                animation.play(self.pos, None)
            }
            DisplayType::Texture(texture) => {
                size = texture.size();
                draw_texture(&texture, self.pos.x, self.pos.y, WHITE)
            }
        }
        draw_circle(
            self.pos.x + size.x / 2.0,
            self.pos.y + size.y / 2.0,
            size.y * 0.6,
            Color {
                r: 1.0,
                g: 1.0,
                b: 1.0,
                a: (0.1),
            },
        );
        self.pos += self.direction * self.speed * get_frame_time()
    }
}
struct SpaceShip {
    pos: Vec2,
    direction: Vec2,
    animation: &'static Animation,
    origin: Vec2,
    loop_timer: f32,
    loop_clock: f32,
}
impl SpaceShip {
    fn new(pos: Vec2, direction: Vec2) -> Self {
        Self {
            loop_timer: 10.0,
            loop_clock: 0.0,
            pos,
            direction,
            animation: &ASSETS.spaceship,
            origin: pos,
        }
    }
    fn update(&mut self) {
        self.pos += self.direction * get_frame_time() * 25.0;
        let params = Some(DrawTextureParams {
            flip_x: self.direction.x.is_sign_positive(),
            ..Default::default()
        });
        if self.loop_timer < 0.0 {
            self.animation
                .play_with_clock(self.pos, self.loop_clock, params.clone());
            self.loop_clock += get_frame_time();
            if self.loop_clock > self.animation.get_duration() {
                self.loop_timer = 7.0;
                self.loop_clock = 0.0;
            }
        } else {
            self.loop_timer -= get_frame_time();
            draw_texture_ex(
                &self.animation.0[0].0,
                self.pos.x,
                self.pos.y,
                WHITE,
                params.unwrap(),
            );
        }
        let colors = [RED, ORANGE, YELLOW, GREEN, BLUE, DARKBLUE];
        let middle = self.pos + self.animation.get_size() / 2.0;
        for (index, color) in colors.iter().enumerate() {
            draw_rectangle(
                self.origin.x,
                middle.y - 3.0 + index as f32,
                self.pos.x - self.origin.x + 5.0,
                1.0,
                *color,
            );
        }
    }
}
pub struct Background {
    objects: Vec<BackgroundObject>,
    spaceship: Option<SpaceShip>,
    spawn_chunks: Vec<f32>,
    size: Vec2,
}
impl Background {
    pub fn new(size: Vec2) -> Self {
        let amount = size.x as usize / 128;
        let mut spawn_chunks = Vec::with_capacity(amount);
        for i in 0..amount {
            spawn_chunks.push(gen_range(0.0, 10.0));
        }
        Self {
            objects: Vec::new(),
            spawn_chunks,
            spaceship: None,
            size,
        }
    }
    pub fn update(&mut self) {
        if self.spaceship.is_none() {
            let right_edge = gen_range(0, 2) == 1;
            let pos = vec2(0., gen_range(0.0, self.size.y));
            self.spaceship = Some(SpaceShip::new(
                pos,
                if right_edge {
                    vec2(-1.0, 0.0)
                } else {
                    vec2(1.0, 0.0)
                },
            ));
        }
        for chunk in &mut self.spawn_chunks {
            *chunk -= get_frame_time();
        }
        self.objects.retain(|f| {
            return f.pos.x < self.size.x || f.pos.y < self.size.y;
        });
        let mut checked = Vec::new();
        while self.objects.len() < self.spawn_chunks.len() {
            if checked.len() >= self.spawn_chunks.len() {
                break;
            }
            let rand = gen_range(0, self.spawn_chunks.len());
            if checked.contains(&rand) {
                continue;
            }
            let chunk = &mut self.spawn_chunks[rand];
            if *chunk <= 0.0 {
                dbg!(rand);
                *chunk = 10.0;
                self.objects
                    .push(BackgroundObject::new(vec2((rand * 128) as f32, -64.0)));
            } else {
                checked.push(rand);
            }
        }
        for object in &mut self.objects {
            object.update();
        }
        if let Some(spaceship) = &mut self.spaceship {
            spaceship.update();
        }
    }
}
