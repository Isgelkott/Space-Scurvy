use macroquad::prelude::*;
use std::sync::LazyLock;

use crate::utils::*;

#[derive(PartialEq, Debug)]
pub enum DisplayType {
    Texture(Texture2D),
    Animation(Animation),
}
pub struct Assets {
    pub spritesheet: Spritesheet,
    pub player: AnimationGroup,
    pub jetpacker: AnimationGroup,
    pub energy_ball: Animation,
    pub spike_ball: AnimationGroup,
    pub laser: Animation,
    pub machine_gunner: AnimationGroup,
    pub energy_ball_shatter: Animation,
    pub laughing_man: AnimationGroup,
    pub acid: AnimationGroup,
    pub lemon: Texture2D,
    pub fire_wagon: AnimationGroup,
    pub blood: Animation,
    pub bomb_chain: AnimationGroup,
    pub bomb: Texture2D,
    pub bomb_explode: Animation,
    pub background_objects: Vec<(DisplayType, Option<f32>)>,
    pub spaceship: Animation,
    pub star: Texture2D,
    pub debris: Texture2D,
    pub lemon_pickup: Animation,
    pub win_animation: Animation,
    pub fish: AnimationGroup,
    pub death_animations: AnimationGroup,
    pub play_again: Texture2D,
    pub red_boss: AnimationGroup,
    pub rocket: AnimationGroup,
    pub cannon: AnimationGroup,
    pub cannon_barrel: AnimationGroup,
    pub lever: AnimationGroup,
    pub cannon_shot_particle: Animation,
    pub gun_inside: Texture2D,
    pub bullet_in_gun: Texture2D,
    pub bullet: AnimationGroup,
}

impl Assets {
    fn new() -> Self {
        Self {
            bullet_in_gun: load_ase_texture(
                include_bytes!("../assets/bullet_in_gun.aseprite"),
                None,
                None,
            ),
            cannon_barrel: load_animation_group(include_bytes!("../assets/cannon_barrel.aseprite")),
            cannon: load_animation_group(include_bytes!("../assets/cannon.aseprite")),
            rocket: load_animation_group(include_bytes!("../assets/rocket.aseprite")),
            red_boss: load_animation_group(include_bytes!("../assets/boss1.aseprite")),
            play_again: load_ase_texture(
                include_bytes!("../assets/play_again.aseprite"),
                None,
                None,
            ),
            death_animations: load_animation_group(include_bytes!("../assets/deaths.aseprite")),
            acid: load_animation_group(include_bytes!("../assets/acid.aseprite")),
            laser: load_animation(include_bytes!("../assets/laser.aseprite")),
            laughing_man: load_animation_group(include_bytes!("../assets/talking_dude.aseprite")),
            spike_ball: load_animation_group(include_bytes!("../assets/spikeball.aseprite")),
            player: load_animation_group(include_bytes!("../assets/pirate.aseprite")),
            machine_gunner: load_animation_group(include_bytes!(
                "../assets/machine_gunner.aseprite"
            )),
            fire_wagon: load_animation_group(include_bytes!("../assets/fire_wagon.aseprite")),
            fish: load_animation_group(include_bytes!("../assets/fish.aseprite")),
            win_animation: load_animation(include_bytes!("../assets/pirate_win.aseprite")),
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
            bomb_chain: load_animation_group(include_bytes!("../assets/bomb_chain.aseprite")),

            bomb: load_ase_texture(include_bytes!("../assets/bomb.aseprite"), None, None),
            blood: load_animation(include_bytes!("../assets/blood.aseprite")),

            lemon: load_ase_texture(include_bytes!("../assets/lemon.aseprite"), None, None),

            energy_ball_shatter: load_animation(include_bytes!(
                "../assets/energy_ball_shatter.aseprite"
            )),

            energy_ball: load_animation(include_bytes!("../assets/energy_ball.aseprite")),
            jetpacker: load_animation_group(include_bytes!("../assets/jetpacker.aseprite")),
            spritesheet: Spritesheet::new(
                (16.0, 16.0),
                load_ase_texture(include_bytes!("../assets/spritesheet.aseprite"), None, None),
            ),
            lever: load_animation_group(include_bytes!("../assets/lever.aseprite")),
            cannon_shot_particle: load_animation(include_bytes!(
                "../assets/laserparticle.aseprite"
            )),
            gun_inside: load_ase_texture(
                include_bytes!("../assets/gun_inside.aseprite"),
                None,
                None,
            ),
            bullet: load_animation_group(include_bytes!("../assets/bullet.aseprite")),
        }
    }
}
pub static ASSETS: LazyLock<Assets> = LazyLock::new(|| Assets::new());
