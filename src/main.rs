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
    level_index: usize,
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
            level_index: 0,

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
        if !self.win {
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
        self.map.draw_foreground();

        update_particle_generators(&mut self.map.tiles, &mut self.particles);
        update_particles(&mut self.particles);
    }
}

struct GameManger {
    game: Game,
    level_index: usize,
    levels: Vec<Levels>,
    win_animation_clock: f32,
}
impl GameManger {
    fn new() -> Self {
        let levels = vec![Levels::TestLevel];
        Self {
            win_animation_clock: 0.0,
            game: Game::new(levels[0]),
            levels: levels,
            level_index: 0,
        }
    }
    async fn update(&mut self) {
        set_camera(&self.game.camera);
        self.game.update().await;
        let mut black_bars = false;

        let transition_length = 2.0;
        if self.game.win {
            let pos = vec2(
                self.game.player.pos.x
                    - if self.game.player.previous_flipped {
                        (ASSETS.top_player_animations.idle.get_size().x
                            + ASSETS.win_animation.get_size().x)
                            / 2.0
                    } else {
                        0.0
                    },
                self.game.player.pos.y
                    - (ASSETS.win_animation.get_size().y
                        - ASSETS.top_player_animations.idle.get_size().y),
            );
            self.win_animation_clock += get_frame_time();
            if self.win_animation_clock > ASSETS.win_animation.get_duration() + transition_length {
                self.level_index += 1;
                self.game = Game::new(self.levels[self.level_index]);
            } else if self.win_animation_clock > ASSETS.win_animation.get_duration() {
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
                    self.win_animation_clock,
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
                (self.win_animation_clock - ASSETS.win_animation.get_duration())
                    / transition_length
                    * screen_height()
                    / 2.0,
                BLACK,
            );
            draw_rectangle(
                0.0,
                screen_height(),
                screen_width(),
                -(self.win_animation_clock - ASSETS.win_animation.get_duration())
                    / transition_length
                    * screen_height()
                    / 2.0,
                BLACK,
            );
        }
        self.game.draw_hud();
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
