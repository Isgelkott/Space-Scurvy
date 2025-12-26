use assets::*;
use level::*;
use macroquad::prelude::*;
use player::*;
use utils::*;
mod assets;
mod level;
mod player;
mod utils;
struct Game {
    map: Level,
    player: Player,
}
impl Game {
    fn new() -> Self {
        Self {
            map: Level::new(Levels::TestLevel),
            player: Player::new(),
        }
    }
    async fn update(&mut self) {
        self.map.draw();
        next_frame().await;
    }
}
enum State {
    Game,
    Menu,
}
struct GameManger {
    game: Game,
}
impl GameManger {
    fn new() -> Self {
        Self { game: Game::new() }
    }
}
#[macroquad::main("krusbar")]
async fn main() {
    let mut game = Game::new();
    loop {
        game.update().await;
    }
}
