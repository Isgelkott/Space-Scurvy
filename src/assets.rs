use macroquad::prelude::*;
use std::sync::LazyLock;

use crate::utils::*;

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
    pub fly: Animation,
}
impl JetpackerAnimation {
    fn new() -> Self {
        let data = include_bytes!("../assets/jetpacker.aseprite");
        Self {
            idle: load_animation_from_tag(data, "idle"),
            fly: load_animation_from_tag(data, "fly"),
        }
    }
}
pub struct Assets {
    pub spritesheet: Spritesheet,
    pub top_player_animations: TopPlayerAnimations,
    pub bottom_player_animations: BotttomPlayerAnimations,
    pub jetpacker: JetpackerAnimation,
    pub energy_ball: Animation,
    pub spike_ball: Animation,
    pub laser: Animation,
    pub machine_gunner: Animation,
}

impl Assets {
    fn new() -> Self {
        Self {
            machine_gunner: load_animation_from_tag(
                include_bytes!("../assets/machine_gunner.aseprite"),
                "idle",
            ),
            laser: load_animation_from_tag(include_bytes!("../assets/laser.aseprite"), "idle"),
            spike_ball: load_animation_from_tag(
                include_bytes!("../assets/spikeball.aseprite"),
                "idle",
            ),
            energy_ball: load_animation_from_tag(
                include_bytes!("../assets/energy_ball.aseprite"),
                "idle",
            ),
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
