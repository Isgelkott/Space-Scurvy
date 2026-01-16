#![allow(unused_variables)]

use enemies::*;
use level::*;
use macroquad::prelude::*;
use player::*;
use utils::*;

use crate::{
    assets::ASSETS,
    background::Background,
    particles::{Particle, update_particle_generators, update_particles},
};
mod assets;
mod background;
mod enemies;
mod level;
mod particles;
mod player;
mod utils;

const SCREEN_SIZE: (f32, f32) = (200.0, 200.0);

pub struct Game {
    win: bool,
    die: bool,
    scale_factor: f32,
    map: Level,
    backgrounds: Background,
    player: Player,
    camera: Camera2D,
    enemies: Vec<Box<dyn Enemy>>,
    pickups: Vec<Pickup>,
    projectiles: Vec<Box<dyn Projectile>>,
    particles: Vec<Particle>,
    map_animations: Vec<MapAnimation>,
}
impl Game {
    fn draw_hud(&self) {
        set_default_camera();
        let hp = self.player.hp;

        for x in 0..5 {
            let x = x as f32;
            let black_x = (hp as f32 - x * 20.0) / 20.0;

            HP_MATERIAL.set_uniform("black_x", black_x);
            gl_use_material(&HP_MATERIAL);
            draw_texture_ex(
                &ASSETS.lemon,
                (7.0 + x * (4.0 + ASSETS.lemon.width())) * self.scale_factor,
                10.0 * self.scale_factor,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(ASSETS.lemon.size() * self.scale_factor),
                    ..Default::default()
                },
            );
        }
        gl_use_default_material();
        set_camera(&self.camera);
    }

    fn new(level: Levels) -> Self {
        let (map, special_data) = Level::new(level);
        let mut enemies = Vec::new();
        for enemy in special_data.enemies.iter() {
            enemies.push(enemy.0.spawn(enemy.1, &map))
        }
        Self {
            die: false,
            win: false,
            pickups: special_data.pickups,
            backgrounds: Background::new(map.world_size),
            scale_factor: 1.0,
            particles: Vec::new(),
            projectiles: Vec::new(),
            map_animations: special_data.map_animations,
            enemies,
            map,
            player: Player::new(special_data.spawn_location),
            camera: create_camera(vec2(SCREEN_SIZE.0, SCREEN_SIZE.1)),
        }
    }
    fn draw_camera(&mut self) {
        set_default_camera();
        clear_background(BLACK);

        self.scale_factor = (screen_width() / SCREEN_SIZE.0).min(screen_height() / SCREEN_SIZE.1);
        draw_texture_ex(
            &self.camera.render_target.as_ref().unwrap().texture,
            0.0,
            0.0,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(
                    SCREEN_SIZE.0 * self.scale_factor,
                    SCREEN_SIZE.1 * self.scale_factor,
                )),
                ..Default::default()
            },
        );
    }
    fn death(&mut self) {
        if let Some(death) = self.player.death {
            draw_rectangle(
                0.0,
                0.0,
                2000.0,
                2000.0,
                Color {
                    r: 1.0,
                    g: 0.0,
                    b: 0.0,
                    a: 0.2,
                },
            );
            let animation = ASSETS.death_animations.get(match death.0 {
                DeathCause::Acid => "acid",
                DeathCause::Energy => "energy",
                DeathCause::Default => "default",
            });
            if death.1 > animation.get_duration() {
                self.die = true;
            } else {
                animation.play_with_clock(
                    self.player.pos - 8.,
                    death.1,
                    Some(DrawTextureParams {
                        flip_x: self.player.previous_flipped,
                        ..Default::default()
                    }),
                );
            }
        }
    }
    async fn update(&mut self) {
        clear_background(BLACK);
        self.backgrounds.update();
        self.map.draw_level();
        update_map_animations(&mut self.map_animations);

        self.camera.target = self.player.pos - vec2(0.0, 10.0);
        update_projectiles(
            &mut self.player,
            &self.map,
            &mut self.projectiles,
            &mut self.particles,
            &mut self.enemies,
        );
        if !self.win && !self.die {
            self.player.update(
                &self.map,
                &mut self.projectiles,
                &mut self.enemies,
                &mut self.particles,
            );
        }
        update_pickups(self);
        update_enemies(
            &self.player,
            &mut self.enemies,
            &self.map,
            &mut self.projectiles,
        );

        update_particle_generators(&mut self.map.tiles, &mut self.particles);
        update_particles(&mut self.particles);
        self.death();
    }
}

struct GameManger {
    game: Game,
    level_index: usize,
    levels: Vec<Levels>,
    clock: f32,
}
impl GameManger {
    fn new() -> Self {
        let levels = vec![Levels::TestLevel];
        Self {
            clock: 0.0,
            game: Game::new(levels[0]),
            levels: levels,
            level_index: 0,
        }
    }
    fn die_screen(&mut self) {
        set_default_camera();
        let button = &ASSETS.play_again;
        let size = button.size() / 2.0
            * (screen_width() / button.width()).min(screen_height() / button.height());
        let pos = vec2(
            (screen_width() - size.x) / 2.0,
            (screen_height() - size.y) / 2.0,
        );
        draw_texture_ex(
            button,
            pos.x,
            pos.y,
            WHITE,
            DrawTextureParams {
                dest_size: Some(size),
                ..Default::default()
            },
        );
        if is_mouse_button_pressed(MouseButton::Left)
            && mouse_position().0 > pos.x
            && mouse_position().0 < pos.x + size.x
            && mouse_position().1 > pos.y
            && mouse_position().1 < pos.y + size.y
        {
            self.game = Game::new(self.levels[self.level_index]);
        }
    }
    async fn update(&mut self) {
        set_camera(&self.game.camera);
        if self.game.die {
            self.die_screen();
        } else {
            self.game.update().await;
            let mut black_bars: bool = false;

            let transition_length = 2.0;
            if self.game.win {
                let pos = vec2(
                    self.game.player.pos.x
                        - if self.game.player.previous_flipped {
                            (ASSETS.jetpacker.get("idle").get_size().x
                                + ASSETS.win_animation.get_size().x)
                                / 2.0
                        } else {
                            0.0
                        },
                    self.game.player.pos.y
                        - (ASSETS.win_animation.get_size().y
                            - ASSETS.jetpacker.get("idle").get_size().y),
                );
                self.clock += get_frame_time();
                if self.clock > ASSETS.win_animation.get_duration() + transition_length {
                    self.level_index += 1;
                    self.game = Game::new(self.levels[self.level_index]);
                } else if self.clock > ASSETS.win_animation.get_duration() {
                    draw_texture_ex(
                        &ASSETS.win_animation.0.last().unwrap().0,
                        pos.x,
                        pos.y,
                        WHITE,
                        DrawTextureParams {
                            flip_x: self.game.player.previous_flipped,
                            ..Default::default()
                        },
                    );
                    black_bars = true;
                } else {
                    ASSETS.win_animation.play_with_clock(
                        vec2(pos.x, pos.y),
                        self.clock,
                        Some(DrawTextureParams {
                            flip_x: self.game.player.previous_flipped,
                            ..Default::default()
                        }),
                    );
                }
            }
            self.game.draw_camera();
            if black_bars {
                draw_rectangle(
                    0.0,
                    0.0,
                    screen_width(),
                    (self.clock - ASSETS.win_animation.get_duration()) / transition_length
                        * screen_height()
                        / 2.0,
                    BLACK,
                );
                draw_rectangle(
                    0.0,
                    screen_height(),
                    screen_width(),
                    -(self.clock - ASSETS.win_animation.get_duration()) / transition_length
                        * screen_height()
                        / 2.0,
                    BLACK,
                );
            }
            self.game.draw_hud();
        }
    }
}
#[macroquad::main("krusbar")]
async fn main() {
    let mut game = GameManger::new();
    rand::srand((get_time() * 1000.0) as u64);
    loop {
        game.update().await;
        next_frame().await;
    }
}
