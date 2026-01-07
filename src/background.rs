use std::{f32::consts::PI, sync::LazyLock};

use macroquad::{math::Vec2, prelude::*, rand::gen_range, texture::Texture2D};

use crate::{
    assets::*,
    utils::{Animation, AnimationMethods},
};
struct Star {
    pos: Vec2,
    direction: Vec2,
    speed: f32,
    delay: f32,
}
impl Star {
    fn new(pos: Vec2, world_size: Vec2) -> Self {
        let speed = gen_range(10.0, 20.0);
        Self {
            delay: gen_range(
                0.0,
                (world_size.y / vec2(1.0, 1.0).to_angle().cos()) / speed,
            ),
            pos,
            direction: vec2(1.0, 1.0),
            speed,
        }
    }
    fn update(&mut self) {
        if self.delay < 0.0 {
            draw_texture(&ASSETS.star, self.pos.x.floor(), self.pos.y.floor(), WHITE);
            self.pos += self.direction * self.speed * get_frame_time();
        } else {
            self.delay -= get_frame_time();
        }
    }
}
struct BackgroundObject {
    display: &'static DisplayType,
    pos: Vec2,
    direction: Vec2,
    speed: f32,
    rotation: f32,
    rotation_mod: f32,
}
impl BackgroundObject {
    fn new(pos: Vec2, last_object: &DisplayType) -> Self {
        let rand = gen_range(0, ASSETS.background_objects.len());
        let mut object = &ASSETS.background_objects[rand];
        while object.0 == *last_object {
            object = &ASSETS.background_objects[rand]
        }

        let size = match &object.0 {
            DisplayType::Animation(animation) => animation.get_size(),
            DisplayType::Texture(texture) => texture.size(),
        };
        let rotation_mod = if let Some(rotation_mod) = object.1 {
            rotation_mod
        } else {
            0.0
        };

        Self {
            speed: (gen_range(300.0, 420.0) / size.x).ceil(),
            display: &object.0,
            pos,
            direction: (vec2(1.0, 1.0)),
            rotation: 0.0,
            rotation_mod,
        }
    }
    fn update(&mut self) {
        let size;
        self.rotation += self.rotation_mod;
        match &self.display {
            DisplayType::Animation(animation) => {
                size = animation.get_size();
                animation.play(
                    self.pos,
                    Some(DrawTextureParams {
                        rotation: self.rotation,
                        ..Default::default()
                    }),
                )
            }
            DisplayType::Texture(texture) => {
                size = texture.size();
                draw_texture_ex(
                    &texture,
                    self.pos.x,
                    self.pos.y,
                    WHITE,
                    DrawTextureParams {
                        rotation: self.rotation,
                        ..Default::default()
                    },
                )
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
                self.origin.x.clamp(self.pos.x - 100.0, self.pos.x + 100.0),
                middle.y - 3.0 + index as f32,
                (self.pos.x - self.origin.x.clamp(self.pos.x - 100.0, self.pos.x + 100.0))
                    + if self.direction.x.is_sign_negative() {
                        self.animation.get_size().x - 5.0
                    } else {
                        5.0
                    },
                1.0,
                *color,
            );
        }
    }
}
pub struct Background {
    objects: Vec<BackgroundObject>,
    spaceships: Vec<SpaceShip>,
    last_object: DisplayType,
    stars: Vec<Star>,
    spawn_chunks: Vec<f32>,
    size: Vec2,
    star_amount: u32,
}
const OBJECT_SIZE: usize = 128;

impl Background {
    pub fn new(size: Vec2) -> Self {
        let amount = size.x as usize / OBJECT_SIZE;
        let mut spawn_chunks = Vec::with_capacity(amount);
        for i in 0..amount {
            spawn_chunks.push(gen_range(0.0, 10.0));
        }
        let stars = Vec::new();

        Self {
            stars,
            objects: Vec::new(),
            last_object: DisplayType::Texture(Texture2D::empty()),
            spawn_chunks,
            star_amount: size.x as u32 / 5,
            spaceships: Vec::new(),
            size,
        }
    }
    pub fn update(&mut self) {
        for (index, spaceship) in self.spaceships.iter_mut().enumerate() {
            if spaceship.pos.x < 0.0 || spaceship.pos.x > self.size.x {
                let right_edge = gen_range(0, 2) == 1;
                let pos = Vec2 {
                    x: if right_edge { self.size.x } else { 0.0 },
                    y: gen_range((index * 400) as f32, (index * 400) as f32),
                };
                *spaceship = SpaceShip::new(
                    pos,
                    if right_edge {
                        vec2(-1.0, 0.0)
                    } else {
                        vec2(1.0, 0.0)
                    },
                );
            } else {
                spaceship.update();
            }
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
                self.objects.push(BackgroundObject::new(
                    vec2(
                        (rand * OBJECT_SIZE) as f32 + gen_range(0.0, OBJECT_SIZE as f32)
                            - self.size.y * f32::to_radians(45.0).cos(),
                        -64.0,
                    ),
                    &self.last_object,
                ));
            } else {
                checked.push(rand);
            }
        }
        self.stars
            .retain(|f| f.pos.x < self.size.x || f.pos.y < self.size.y);
        while self.stars.len() < self.star_amount as usize {
            self.stars.push(Star::new(
                vec2(gen_range(0.0, self.size.x) - self.size.y * PI / 4.0, 0.0).floor(),
                self.size,
            ));
        }
        for star in &mut self.stars {
            star.update();
        }
        for object in &mut self.objects {
            object.update();
        }
    }
}
