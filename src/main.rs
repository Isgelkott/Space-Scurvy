use macroquad::prelude::*;
mod utils;
use utils::*;
mod level;
use level::*;
struct Tile {
    textures: Vec<(u8, u8)>,
}
struct Game {
    map: Level,
    player: Player,
}
impl Game {
    fn new() -> Self {
        Self {
            map: Level::new(),
            player: Player::new(),
        }
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
    let game = Game::new();
}
