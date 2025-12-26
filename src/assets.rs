use macroquad::prelude::*;
use std::sync::LazyLock;

use crate::utils::*;
pub struct Spritesheet {
    spritesheet: Texture2D,
    size: (f32, f32),
}
impl Spritesheet {
    pub fn draw_from(&self, coord: (u8, u8), pos: Vec2, params: Option<DrawTextureParams>) {
        let mut params = params.unwrap_or_default();
        params.source = Some(Rect {
            x: coord.0 as f32 * self.size.0,
            y: coord.1 as f32 * self.size.1,
            w: self.size.0,
            h: self.size.1,
        });
        draw_texture_ex(&self.spritesheet, pos.x, pos.y, WHITE, params);
    }
    fn new(size: (f32, f32), texture: Texture2D) -> Self {
        Self {
            spritesheet: texture,
            size,
        }
    }
}
pub struct Assets {
    pub spritesheet: Spritesheet,
}

impl Assets {
    fn new() -> Self {
        Self {
            spritesheet: Spritesheet::new(
                (16.0, 16.0),
                load_ase_texture(include_bytes!("../assets/spritesheet.aseprite"), None, None),
            ),
        }
    }
}
pub static ASSETS: LazyLock<Assets> = LazyLock::new(|| Assets::new());
