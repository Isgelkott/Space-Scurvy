use assets::*;
use enemies::*;
use level::*;
use macroquad::prelude::*;
use player::*;
use utils::*;
mod assets;
mod enemies;
mod level;
mod player;
mod utils;
const SCREEN_SIZE: (f32, f32) = (200.0, 200.0);

struct Game {
    map: Level,
    player: Player,
    camera: Camera2D,
    clock: f32,
    enemies: Vec<Box<dyn Enemy>>,
    projectiles: Vec,
}
impl Game {
    fn new() -> Self {
        let (map, special_data) = Level::new(Levels::TestLevel);
        dbg!(special_data.spawn_location);
        Self {
            enemies: special_data.enemies,
            clock: 0.0,
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
        clear_background(GRAY);

        self.map.draw();

        self.camera.target = self.player.pos;
        self.player.update(&self.map);
        update_enemies(&mut self.enemies, &self.map);
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
