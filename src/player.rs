use macroquad::prelude::*;
pub struct Player {
    pos: Vec2,
    direction: Vec2,
}
impl Player {
    pub fn new() -> Self {
        Self {
            pos: Vec2::ZERO,
            direction: Vec2::ZERO,
        }
    }
}
