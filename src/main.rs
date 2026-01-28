#![allow(unused_variables)]

use enemies::*;
use level::*;
use macroquad::prelude::*;
use player::*;
use utils::*;

use crate::{
    assets::ASSETS,
    background::Background,
    bosses::Boss,
    particles::{Particle, update_particle_generators, update_particles},
};
mod assets;
mod background;
mod bosses;
mod enemies;
mod level;
mod particles;
mod player;
mod utils;

const SCREEN_SIZE: (f32, f32) = (304.0, 171.0);

pub struct Game {
    win: bool,
    die: bool,
    scale_factor: f32,
    map: Level,
    backgrounds: Background,
    player: Player,
    camera: Camera2D,
    enemies: Vec<Box<dyn Enemy>>,
    boss: Option<Box<dyn Boss>>,
    pickups: Vec<Pickup>,
    projectiles: Vec<Box<dyn Projectile>>,
    particles: Vec<Particle>,
    // map_animations: Vec<MapAnimation>,
    map_animations: Vec<MapAnimation>,
}
impl Game {
    fn draw_hud(&self) {
        set_default_camera();

        for x in 0..5 {
            let x = x as f32;
            let black_x = (self.player.hp as f32 - x * 20.0) / 20.0;

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
            boss: if let Some((boss, tile)) = special_data.boss {
                Some(boss.to_boss(tile, &map))
            } else {
                None
            },
            die: false,
            win: false,
            pickups: special_data.pickups,
            backgrounds: Background::new(map.world_size),
            scale_factor: 1.0,
            particles: Vec::new(),
            projectiles: Vec::new(),
            // map_animations: special_data.map_animations,
            map_animations: Vec::new(),
            enemies,
            map,
            player: Player::new(special_data.spawn_location),
            camera: create_camera(vec2(SCREEN_SIZE.0, SCREEN_SIZE.1)),
        }
    }
    fn draw_camera(&mut self) {
        set_default_camera();
        clear_background(BLACK);

        self.scale_factor = (screen_width() / SCREEN_SIZE.0)
            .min(screen_height() / SCREEN_SIZE.1)
            .floor();
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
                DeathCause::Explode => "explode",
            });
            if death.1 > animation.get_duration() {
                self.die = true;
            } else {
                animation.play_with_clock(
                    self.player.pos - ASSETS.death_animations.get_size() / 2.0
                        + self.player.size / 2.0,
                    death.1,
                    Some(DrawTextureParams {
                        flip_x: self.player.previous_flipped,
                        ..Default::default()
                    }),
                );
            }
        }
    }
    async fn update(&mut self, frame_time: f32) {
        clear_background(BLACK);
        self.backgrounds.update(frame_time);
        if let Some(boss) = &mut self.boss {
            boss.update(
                &self.map,
                &mut self.enemies,
                &mut self.projectiles,
                frame_time,
                &self.player,
                &mut self.particles,
            );
        }
        self.map.draw_level();
        update_map_animations(&mut self.map_animations);

        update_projectiles(
            &mut self.player,
            &self.map,
            &mut self.projectiles,
            &mut self.particles,
            &mut self.enemies,
            frame_time,
        );

        if !self.win && !self.die {
            self.player.update(
                &mut self.map,
                &mut self.projectiles,
                &mut self.enemies,
                &mut self.particles,
                frame_time,
            );
        }
        #[cfg(debug_assertions)]
        {
            if is_key_down(KeyCode::B) {
                dbg!(self.player.pos);
            }
        }
        self.camera.target = self.player.pos - vec2(0.0, 40.0);
        draw_rectangle(
            SCREEN_SIZE.0 - 10.0,
            SCREEN_SIZE.1 - 10.0,
            10.0,
            10.0,
            WHITE,
        );
        draw_rectangle(0.0, 0.0, 10.0, 10.0, RED);
        update_pickups(self);
        update_enemies(
            &self.player,
            &mut self.enemies,
            &self.map,
            &mut self.projectiles,
            frame_time,
        );

        update_particle_generators(&mut self.map.tiles, &mut self.particles, frame_time);
        update_particles(&mut self.particles, frame_time);
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
            let frame_time = get_frame_time().min(1. / 60.0);

            self.game.update(frame_time).await;
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
                self.clock += frame_time;
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
