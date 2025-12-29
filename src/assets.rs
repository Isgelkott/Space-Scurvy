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

pub struct TopPlayerAnimations {
    pub idle: Animation,
}

impl TopPlayerAnimations {
    fn new() -> Self {
        let data = include_bytes!("../assets/pirate.aseprite");
        Self {
            idle: load_animation_from_tag(data, "idle_top"),
        }
    }
}
pub struct BotttomPlayerAnimations {
    pub idle: Animation,
    pub walk: Animation,
}
impl BotttomPlayerAnimations {
    fn new() -> Self {
        let data = include_bytes!("../assets/pirate.aseprite");
        Self {
            idle: load_animation_from_tag(data, "idle_bot"),
            walk: load_animation_from_tag(data, "walk"),
        }
    }
}
pub struct JetpackerAnimation {
    pub idle: Animation,
    // pub fly: Animation,
}
impl JetpackerAnimation {
    fn new() -> Self {
        let data = include_bytes!("../assets/jetpacker.aseprite");
        Self {
            idle: load_animation_from_tag(data, "idle"),
            // fly: load_animation_from_tag(data, "fly"),
        }
    }
}
pub struct Assets {
    pub spritesheet: Spritesheet,
    pub top_player_animations: TopPlayerAnimations,
    pub bottom_player_animations: BotttomPlayerAnimations,
    pub jetpacker: JetpackerAnimation,
}

impl Assets {
    fn new() -> Self {
        Self {
            jetpacker: JetpackerAnimation::new(),
            spritesheet: Spritesheet::new(
                (16.0, 16.0),
                load_ase_texture(include_bytes!("../assets/spritesheet.aseprite"), None, None),
            ),
            top_player_animations: TopPlayerAnimations::new(),
            bottom_player_animations: BotttomPlayerAnimations::new(),
        }
    }
}
pub static ASSETS: LazyLock<Assets> = LazyLock::new(|| Assets::new());
