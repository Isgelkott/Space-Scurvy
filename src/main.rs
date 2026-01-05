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

struct Game {
    scale_factor: f32,
    map: Level,
    backgrounds: Background,
    player: Player,
    camera: Camera2D,
    enemies: Vec<Box<dyn Enemy>>,
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
    fn new() -> Self {
        let (map, special_data) = Level::new(Levels::TestLevel);
        let mut enemies = Vec::new();
        for enemy in special_data.enemies.iter() {
            enemies.push(enemy.0.spawn(enemy.1, &map))
        }
        Self {
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
        set_camera(&self.camera);
    }

    async fn update(&mut self) {
        clear_background(BLACK);
        self.backgrounds.update();
        self.map.draw_level();
        update_map_animations(&mut self.map_animations);

        self.camera.target = self.player.pos;
        update_projectiles(
            &mut self.player,
            &self.map,
            &mut self.projectiles,
            &mut self.particles,
            &mut self.enemies,
        );
        self.player.update(
            &self.map,
            &mut self.projectiles,
            &mut self.enemies,
            &mut self.particles,
        );
        update_enemies(
            &self.player,
            &mut self.enemies,
            &self.map,
            &mut self.projectiles,
        );
        self.map.draw_foreground();

        update_particle_generators(&mut self.map.tiles, &mut self.particles);
        update_particles(&mut self.particles);
        self.draw_camera();
        self.draw_hud();
    }
}

struct GameManger {
    game: Game,
}
impl GameManger {
    fn new() -> Self {
        Self { game: Game::new() }
    }
    async fn update(&mut self) {
        self.game.update().await;
    }
}
#[macroquad::main("krusbar")]
async fn main() {
    let mut game = GameManger::new();
    loop {
        game.update().await;
        next_frame().await;
    }
}
