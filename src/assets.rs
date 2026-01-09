use macroquad::prelude::*;
use std::sync::LazyLock;

use crate::utils::*;

pub struct TopPlayerAnimations {
    pub idle: Animation,
    pub shoot: Animation,
}

impl TopPlayerAnimations {
    fn new() -> Self {
        let data = include_bytes!("../assets/pirate.aseprite");
        Self {
            shoot: load_animation_from_tag(data, "shoot"),
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
    pub fall: Animation,
    pub hit: Animation,
    pub getup: Animation,
}
impl JetpackerAnimation {
    fn new() -> Self {
        let data = include_bytes!("../assets/jetpacker.aseprite");
        Self {
            hit: load_animation_from_tag(data, "hit"),
            fall: load_animation_from_tag(data, "fall"),
            idle: load_animation_from_tag(data, "idle"),
            fly: load_animation_from_tag(data, "fly"),
            getup: load_animation_from_tag(data, "getup"),
        }
    }
}
#[derive(PartialEq)]
pub enum DisplayType {
    Texture(Texture2D),
    Animation(Animation),
}
pub struct Assets {
    pub spritesheet: Spritesheet,
    pub top_player_animations: TopPlayerAnimations,
    pub bottom_player_animations: BotttomPlayerAnimations,
    pub jetpacker: JetpackerAnimation,
    pub energy_ball: Animation,
    pub spike_ball: Animation,
    pub laser: Animation,
    pub machine_gunner_shoot: Animation,
    pub energy_ball_shatter: Animation,
    pub laughing_man: (Animation, Animation, Animation),
    pub acid: Animation,
    pub lemon: Texture2D,
    pub machine_gunner_inactive: Animation,
    pub fire_wagon_wheel: Animation,
    pub fire_wagon_jiggle: Animation,
    pub fire_wagon_fire: Animation,
    pub blood: Animation,
    pub bomb_chain: Texture2D,
    pub bomb: Texture2D,
    pub bomb_explode: Animation,
    pub background_objects: Vec<(DisplayType, Option<f32>)>,
    pub spaceship: Animation,
    pub star: Texture2D,
    pub debris: Texture2D,
    pub lemon_pickup: Animation,
}

impl Assets {
    fn new() -> Self {
        let laughing_man = include_bytes!("../assets/talking_dude.aseprite");
        let machine_gunner = include_bytes!("../assets/machine_gunner.aseprite");
        let fire_wagon = include_bytes!("../assets/fire_wagon.aseprite");
        Self {
            lemon_pickup: load_animation(include_bytes!("../assets/lemon_pickup.aseprite")),
            debris: load_ase_texture(include_bytes!("../assets/debris.aseprite"), None, None),
            star: load_ase_texture(include_bytes!("../assets/star.aseprite"), None, None),
            spaceship: load_animation(include_bytes!("../assets/spaceship.aseprite")),
            background_objects: vec![
                (
                    DisplayType::Animation(load_animation(include_bytes!(
                        "../assets/earth.aseprite"
                    ))),
                    None,
                ),
                (
                    DisplayType::Texture(load_ase_texture(
                        include_bytes!("../assets/abandonded.aseprite"),
                        None,
                        None,
                    )),
                    Some(0.02),
                ),
                (
                    DisplayType::Texture(load_ase_texture(
                        include_bytes!("../assets/rock.aseprite"),
                        None,
                        None,
                    )),
                    Some(0.03),
                ),
            ],
            bomb_explode: load_animation(include_bytes!("../assets/bomb_explode.aseprite")),
            bomb_chain: load_ase_texture(
                include_bytes!("../assets/bomb_chain.aseprite"),
                None,
                None,
            ),
            bomb: load_ase_texture(include_bytes!("../assets/bomb.aseprite"), None, None),
            blood: load_animation(include_bytes!("../assets/blood.aseprite")),
            fire_wagon_fire: load_animation_from_tag(fire_wagon, "fire"),
            fire_wagon_jiggle: load_animation_from_tag(fire_wagon, "jiggle"),
            fire_wagon_wheel: load_animation_from_tag(fire_wagon, "drive"),
            lemon: load_ase_texture(include_bytes!("../assets/lemon.aseprite"), None, None),
            acid: load_animation(include_bytes!("../assets/acid.aseprite")),
            laughing_man: (
                load_animation_from_tag(laughing_man, "off"),
                load_animation_from_tag(laughing_man, "active"),
                load_animation_from_tag(laughing_man, "turn_off"),
            ),

            energy_ball_shatter: load_animation(include_bytes!(
                "../assets/energy_ball_shatter.aseprite"
            )),
            machine_gunner_shoot: load_animation_from_tag(
                include_bytes!("../assets/machine_gunner.aseprite"),
                "idle",
            ),
            machine_gunner_inactive: load_animation_from_tag(machine_gunner, "inactive"),
            laser: load_animation_from_tag(include_bytes!("../assets/laser.aseprite"), "idle"),
            spike_ball: load_animation_from_tag(
                include_bytes!("../assets/spikeball.aseprite"),
                "4",
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
