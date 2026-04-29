use crate::{
    Game,
    assets::ASSETS,
    bosses::Bosses,
    enemies::{ENEMY_IDS, PresetEnemies, check_collision_with_size},
    particles::ParticleGenerator,
    player::DeathCause,
    utils::{Animation, AnimationMethods},
};
use line_ending::LineEnding;
use macroquad::{prelude::*, rand::gen_range};
use std::{
    fmt::Debug,
    panic,
};
pub const TILE_SIZE: f32 = 16.0;
// #[derive(PartialEq, Clone, Copy)]
// pub enum SpecialLayer {
//     Collision,
//     Decor,
//     OverPlayer,
//     OneWayCollision,
//     Enemies,
//     Special,
//     Path,
//     Death,
// }
// impl SpecialLayer {
//     fn from_str(input: &str) -> Self {
//         match input {
//             "collision" => Self::Collision,
//             input if input.contains("decor") => Self::Decor,
//             "one_way" => Self::OneWayCollision,
//             "over_player" => Self::OverPlayer,
//             "enemies" => Self::Enemies,
//             "special" => Self::Special,
//             "path" => Self::Path,
//             "death" => Self::Death,
//             _ => panic!("no layer named {}", input),
//         }
//     }
// }
#[derive(PartialEq, Debug, Clone, Copy)]
pub enum VisualData {
    ID(u16),
    Animation(&'static Animation),
}
#[derive(PartialEq, Clone, Copy, Debug)]
pub enum SpecialTileData {
    Path,
    OutOfBounds,
    Acid,
    Cannon,
    Switch,
}
#[derive(Clone, Copy)]
pub enum TriggerBehaviour {
    PlayAnimationOnce(&'static Animation),
}
pub struct MapAnimation {
    clock: f32,
    animation: &'static Animation,
    pos: Vec2,
}
impl MapAnimation {
    pub fn new(pos: Vec2, animation: &'static Animation) -> Self {
        Self {
            clock: gen_range(0., animation.get_duration()),
            animation,
            pos,
        }
    }
    pub fn update(&mut self, frame_time:f32){
        self.clock+= frame_time;
        if self.clock > self.animation.get_duration(){
            self.clock = 0.;
        }   
              self.animation.play_with_clock(self.pos, self.clock, None);

    }
}
#[derive(Default, Clone)]
pub struct Tile {
    pub visual: Vec<VisualData>,
    pub special_data: Vec<SpecialTileData>,
    pub collision: bool,
    pub one_way_collision: bool,
    pub death_cause: Option<DeathCause>,
    pub particle_generator: Option<ParticleGenerator>,
    pub trigger: Option<bool>,
    pub trigger_behaviour: Option<TriggerBehaviour>,
    pub ground: bool,
}
impl Tile {
    pub fn has_special(&self, special: SpecialTileData) -> bool {
        self.special_data.iter().any(|f| matches!(f, special))
    }
    fn trigger(&mut self, pos: Vec2, map_animations: &mut Vec<MapAnimation>) {
        let trigger_behaviour = self.trigger_behaviour.as_ref().unwrap();
        match trigger_behaviour {
            TriggerBehaviour::PlayAnimationOnce(animation) => {
                map_animations.push(MapAnimation::new(pos, animation));
            }
        }
    }
}
pub struct Chunk {
    pub pos: (i16, i16),
    pub tiles: Vec<Tile>,
}
pub fn floored_pos(pos: Vec2) -> Vec2 {
    return vec2(
        (pos.x / TILE_SIZE).floor() * TILE_SIZE,
        (pos.y / TILE_SIZE).floor() * TILE_SIZE,
    );
}
pub fn load_tilemap(tilemap: &str, tileset: &str) -> (Level, SpecialData) {
    fn parse_id(tile: &mut Tile, id: u16, special_data: &mut SpecialData, pos: Vec2) {
        let mut visual = None;
        if id == 0 {
            return;
        }
            #[allow(non_contiguous_range_endpoints)]

        match id {
            1..20 => {
                tile.collision = true;
                visual = Some(id);
            }
            60..80 => {
                let map_animation = match id {
                    62 =>  &ASSETS.laughing_man,
                    _=> panic!()
                };
                special_data.map_animations.push(MapAnimation::new(pos, map_animation.base()));
          
            }

            81 => {
                tile.particle_generator = Some(ParticleGenerator::new(
                    pos,
                    crate::particles::ParticleType::Acid,
                ));
                tile.death_cause = Some(DeathCause::Acid);
                tile.visual
                    .push(VisualData::Animation(&ASSETS.acid.get("surface")));
            }
            82 => {
                tile.death_cause = Some(DeathCause::Acid);

                tile.visual
                    .push(VisualData::Animation(&ASSETS.acid.get("inside")));
            }

            140..160 => {
                // enemies
                let enemy = *ENEMY_IDS.get(&id).unwrap();
                dbg!(enemy, pos, id);
                special_data.enemies.push((enemy, pos));
            }
            160..180 => {
                // enemies with tiles
                let enemy = *ENEMY_IDS.get(&id).unwrap();
                dbg!(enemy, pos, id);
                special_data.enemies.push((enemy, pos));
            }
            200..220 => {
                // one way collision
                tile.one_way_collision = true;
            }
            221 => {
                special_data.spawn_location = pos;
            }
            241 => tile.special_data.push(SpecialTileData::Path),
            243 => tile.special_data.push(SpecialTileData::OutOfBounds),
            340..360 => {
                let pickup = match id {
                    341 => Pickup {
                        pickup_effect: PickupEffects::Win,
                        origin: pos,
                        size: ASSETS.lemon_pickup.get_size(),
                        animation: &ASSETS.lemon_pickup,
                    },
                    _ => panic!(),
                };
                special_data.pickups.push(pickup);
            }
            360..380 => {
                let boss = match id {
                    361 => Bosses::RedGuy,
                    _ => unreachable!(),
                };
                special_data.boss = Some((boss, pos));
            }
            380..400 => match id {
                381 => {
                    tile.special_data.push(SpecialTileData::Cannon);
                }
                382 => {
                    tile.special_data.push(SpecialTileData::Switch);
                }
                _ => panic!(),
            },
            480..500 => {
                tile.ground = true;
                visual = Some(id);
                tile.collision = true;
            }
            _ => visual = Some(id),
        };
        if let Some(visual) = visual {
            tile.visual.push(VisualData::ID(visual));
        }
    }
    let tilemap = &LineEnding::normalize(tilemap);
    let tileset_width = tileset
        .split_once("columns=\"")
        .unwrap()
        .1
        .split_once("\"")
        .unwrap()
        .0
        .parse::<u16>()
        .unwrap();
    let mut end_x = i16::MIN;
    let mut start_x = i16::MAX;
    let mut end_y = i16::MIN;
    let mut start_y = i16::MAX;
    let mut special_data = SpecialData::default();
    let mut chunks: Vec<Chunk> = Vec::new();
    for layer in tilemap.split("<layer").skip(1) {
        let layer_name = layer
            .split_once("name=\"")
            .unwrap()
            .1
            .split_once("\"")
            .unwrap()
            .0;

        for chunk in layer.split("<chunk").skip(1) {
            let mut chunk_data = [0; 256];
            let chunk_x: i16 = chunk
                .split_once("x=\"")
                .unwrap()
                .1
                .split_once("\"")
                .unwrap()
                .0
                .parse()
                .unwrap();
            let chunk_y: i16 = chunk
                .split_once("y=\"")
                .unwrap()
                .1
                .split_once("\"")
                .unwrap()
                .0
                .parse()
                .unwrap();
            if !chunks.iter().any(|f| f.pos == (chunk_x, chunk_y)) {
                chunks.push(Chunk {
                    pos: (chunk_x, chunk_y),
                    tiles: vec![Tile::default(); 256],
                });
            }
            let chunk_data = chunks
                .iter_mut()
                .find(|f| f.pos == (chunk_x, chunk_y))
                .unwrap();
            for (index, id) in chunk
                .split_once("\n")
                .unwrap()
                .1
                .split_once("\n</chunk")
                .unwrap()
                .0
                .replace("\n", "")
                .split(",")
                .enumerate()
            {
                let x = index as i16 % 16 + chunk_x;
                let y = index as i16 / 16 + chunk_y;
                let id = id.parse().unwrap();

                if id != 0 {
                    if x > end_x {
                        end_x = x
                    }
                    if x < start_x {
                        start_x = x
                    }
                    if y < start_y {
                        start_y = y
                    }
                    if y > end_y {
                        end_y = y
                    }
                }
                let pos = vec2(x as f32, y as f32) * TILE_SIZE;
                let mut tile = &mut chunk_data.tiles[index];
                parse_id(&mut tile, id, &mut special_data, pos);
            }
        }
    }
    (
        Level {
            chunks,
            tileset_width,
        },
        special_data,
    )
}

#[derive(Clone, Copy)]
pub enum Levels {
    TestLevel,
}


#[derive(PartialEq)]
enum PickupEffects {
    Win,
    Heal,
}
pub struct Pickup {
    origin: Vec2,
    size: Vec2,
    animation: &'static Animation,
    pickup_effect: PickupEffects,
}
pub fn update_pickups(game: &mut Game) {
    game.pickups.retain(|pickup| {
        let pos = vec2(
            pickup.origin.x,
            pickup.origin.y + (get_time() * 5.0).sin() as f32 * 5.0,
        );
        pickup.animation.play(pos, None);
        if check_collision_with_size((pos, pickup.size), (game.player.pos, game.player.size)) {
            if pickup.pickup_effect == PickupEffects::Win {
                game.win = true;
            }
            return false;
        }
        return true;
    });
}
#[derive(Default)]
pub struct SpecialData {
    pub spawn_location: Vec2,
    pub enemies: Vec<(PresetEnemies, Vec2)>,
    pub boss: Option<(Bosses, Vec2)>,
    pub pickups: Vec<Pickup>,
    pub map_animations: Vec<MapAnimation>
}
pub struct Level {
    pub chunks: Vec<Chunk>, //pub width: usize,
    //pub world_size: Vec2,
    //pub map_animations: Vec<MapAnimation>,
    tileset_width: u16,
}
impl Level {
    pub fn get_tile(&self, pos: Vec2) -> Option<&Tile> {
        let ipos = ((pos.x / TILE_SIZE) as i16, (pos.y / TILE_SIZE) as i16);
        let chunk_x = ((ipos.0 as f32 / 16.0).floor() * 16.0) as i16;
        let chunk_y = ((ipos.1 as f32 / 16.0).floor() * 16.0) as i16;
        if let Some(chunk) = self.chunks.iter().find(|f| f.pos == (chunk_x, chunk_y)) {
            let local_x = ipos.0 - chunk_x;
            assert!(!local_x.is_negative());
            let local_y = ipos.1 - chunk_y;
            let index = (local_x % 16 + local_y * 16) as usize;
            return Some(&chunk.tiles[index]);
        }
        return None;
    }
    pub fn draw_level(&self) {
        for chunk in &self.chunks {
            for (index, tile) in chunk.tiles.iter().enumerate() {
                let pos = vec2(
                    (chunk.pos.0 + index as i16 % 16) as f32,
                    (chunk.pos.1 + index as i16 / 16) as f32,
                ) * TILE_SIZE;
                for visual in &tile.visual {
                    match visual {
                        VisualData::Animation(animation) => {
                            animation.play(
                                pos,
                                Some(DrawTextureParams {
                                    dest_size: Some(vec2(TILE_SIZE, TILE_SIZE)),
                                    ..Default::default()
                                }),
                            );
                        }
                        VisualData::ID(id) => {
                            if *id == 0 {
                                continue;
                            }
                            let id = id - 1;
                            ASSETS.spritesheet.draw_from(
                                (id % self.tileset_width, id / self.tileset_width),
                                pos,
                                Some(DrawTextureParams {
                                    dest_size: Some(vec2(TILE_SIZE, TILE_SIZE)),
                                    ..Default::default()
                                }),
                            );
                        }
                    }
                }
            }
        }
    }
}
