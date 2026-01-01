use enemies::*;
use level::*;
use macroquad::prelude::*;
use player::*;
use utils::*;

use crate::particles::{Particle, update_particles};
mod assets;
mod enemies;
mod level;
mod particles;
mod player;
mod utils;
const SCREEN_SIZE: (f32, f32) = (200.0, 200.0);

struct Game {
    map: Level,
    player: Player,
    camera: Camera2D,
    enemies: Vec<Box<dyn Enemy>>,
    projectiles: Vec<Box<dyn Projectile>>,
    particles: Vec<Box<dyn Particle>>,
}
impl Game {
    fn new() -> Self {
        let (map, special_data) = Level::new(Levels::TestLevel);
        let mut enemies = Vec::new();
        for enemy in special_data.enemies.iter() {
            enemies.push(enemy.0.spawn(enemy.1, &map))
        }
        Self {
            particles: Vec::new(),
            projectiles: Vec::new(),
            enemies,
            map,
            player: Player::new(special_data.spawn_location),
            camera: create_camera(vec2(SCREEN_SIZE.0, SCREEN_SIZE.1)),
        }
    }
    fn draw_camera(&self) {
        set_default_camera();
        clear_background(BLACK);

        let scale_factor = (screen_width() / SCREEN_SIZE.0).min(screen_height() / SCREEN_SIZE.1);
        draw_texture_ex(
            &self.camera.render_target.as_ref().unwrap().texture,
            0.0,
            0.0,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(
                    SCREEN_SIZE.0 * scale_factor,
                    SCREEN_SIZE.1 * scale_factor,
                )),
                ..Default::default()
            },
        );
        set_camera(&self.camera);
    }

    async fn update(&mut self) {
        clear_background(BLACK);

        self.map.draw();

        self.camera.target = self.player.pos;
        self.player.update(&self.map, &mut self.projectiles);
        update_enemies(
            &self.player,
            &mut self.enemies,
            &self.map,
            &mut self.projectiles,
        );
        update_projectiles(
            &mut self.player,
            &self.map,
            &mut self.projectiles,
            &mut self.particles,
            &mut self.enemies,
        );
        update_particles(&mut self.particles);
        self.draw_camera();
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
