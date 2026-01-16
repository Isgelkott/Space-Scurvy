use crate::{
    Game,
    assets::ASSETS,
    enemies::{ENEMY_IDS, Enemy, PresetEnemies, check_collision_with_size},
    particles::ParticleGenerator,
    player::DeathCause,
    utils::{Animation, AnimationMethods},
};
use macroquad::prelude::*;
use std::{collections::HashMap, sync::LazyLock};
pub const TILE_SIZE: f32 = 16.0;
pub const MAP_SCALE_FACTOR: f32 = 1.0;
#[derive(PartialEq, Clone, Copy)]
pub enum Layer {
    Collision,
    Decor,
    OverPlayer,
    OneWayCollision,
    Enemies,
    Special,
    Path,
    Death,
}
impl Layer {
    fn from_str(input: &str) -> Self {
        match input {
            "collision" => Self::Collision,
            input if input.contains("decor") => Self::Decor,
            "one_way" => Self::OneWayCollision,
            "over_player" => Self::OverPlayer,
            "enemies" => Self::Enemies,
            "special" => Self::Special,
            "path" => Self::Path,
            "death" => Self::Death,
            _ => panic!("no layer named {}", input),
        }
    }
}
#[derive(PartialEq)]
pub enum VisualData {
    ID(usize),
    Animation(&'static Animation),
}
#[derive(PartialEq, Clone, Copy)]
pub enum SpecialTileData {
    Path,
    Acid,
}
#[derive(Default)]
pub struct Tile {
    pub visual: Vec<VisualData>,
    pub special_data: Vec<SpecialTileData>,
    pub collision: bool,
    pub one_way_collision: bool,
    pub death_cause: Option<DeathCause>,
    pub particle_generator: Option<ParticleGenerator>,
}

pub fn load_tilemap(tilemap: &str, tileset: &str) -> ((Vec<Tile>, usize), SpecialData, usize) {
    let mut special_data = SpecialData::default();
    let tile_set_width = tileset
        .split_once("columns=\"")
        .unwrap()
        .1
        .split_once("\"")
        .unwrap()
        .0
        .parse::<usize>()
        .unwrap();
    dbg!(tile_set_width);
    fn get_area(chunks: &HashMap<(i32, i32), [u16; 256]>) -> Option<(i32, i32, i32, i32)> {
        let posses: Vec<(i32, i32, i32, i32)> = chunks
            .iter()
            .map(|f| {
                let lowest_x = f.0.0
                    + f.1
                        .iter()
                        .enumerate()
                        .filter(|f| *f.1 != 0)
                        .map(|f| f.0 % 16)
                        .min()
                        .unwrap() as i32;
                let highest_x = f.0.0
                    + f.1
                        .iter()
                        .enumerate()
                        .filter(|f| *f.1 != 0)
                        .map(|f| f.0 % 16)
                        .max()
                        .unwrap() as i32;
                let lowest_y = f.0.1
                    + f.1
                        .iter()
                        .enumerate()
                        .filter(|f| *f.1 != 0)
                        .map(|f| f.0 / 16)
                        .min()
                        .unwrap() as i32;
                let highest_y = f.0.1
                    + f.1
                        .iter()
                        .enumerate()
                        .filter(|f| *f.1 != 0)
                        .map(|f| f.0 / 16)
                        .max()
                        .unwrap() as i32;
                (lowest_x, lowest_y, highest_x, highest_y)
            })
            .collect();
        dbg!(&posses);
        if posses.is_empty() {
            return None;
        }
        let lowest_x = posses.iter().map(|f| f.0).min().unwrap_or(posses[0].0);
        let highest_x = posses.iter().map(|f| f.2).max().unwrap();
        let lowest_y = posses.iter().map(|f| f.1).min().unwrap_or(posses[0].1);
        let highest_y = posses.iter().map(|f| f.3).max().unwrap_or(posses[0].3);

        Some((lowest_x, lowest_y, highest_x, highest_y))
    }
    let mut layers: Vec<(HashMap<(i32, i32), [u16; 256]>, Layer)> = Vec::new();
    for layer in tilemap.split("<layer").skip(1) {
        let name = layer
            .split_once("name=\"")
            .unwrap()
            .1
            .split_once("\"")
            .unwrap()
            .0;
        dbg!(name);
        let mut chunks: HashMap<(i32, i32), [u16; 256]> = HashMap::new();
        for chunk in layer.split("<chunk").skip(1) {
            let x = chunk
                .split_once("x=\"")
                .unwrap()
                .1
                .split_once("\"")
                .unwrap()
                .0
                .parse::<i32>()
                .unwrap();
            let y = chunk
                .split_once("y=\"")
                .unwrap()
                .1
                .split_once("\"")
                .unwrap()
                .0
                .parse::<i32>()
                .unwrap();

            let chunk = chunk
                .split_once("\r\n")
                .unwrap()
                .1
                .split_once("\r\n</")
                .unwrap()
                .0;
            let mut data = [0; 256];

            for (index, id) in chunk.split(",").enumerate() {
                let id = if id.contains("\r\n") {
                    &id.replace("\r\n", "")
                } else {
                    id
                };

                data[index] = id.parse::<u16>().unwrap();
            }
            if data.iter().all(|f| *f == 0) {
                println!("chunk x: {},y: {} is empty ", x, y);
                continue;
            } else {
                println!("chunk is full of juice x: {}y:{}", x, y)
            }

            chunks.insert((x, y), data);
        }
        layers.push((chunks, Layer::Collision));
    }
    let layers_pos: Vec<(i32, i32, i32, i32)> = layers
        .iter()
        .map(|f| get_area(&f.0))
        .filter(|f| f.is_some())
        .map(|f| f.unwrap())
        .collect();

    dbg!(&layers_pos);
    let area: (i32, i32, i32, i32) = (
        layers_pos.iter().map(|f| f.0).min().unwrap(),
        layers_pos.iter().map(|f| f.1).min().unwrap(),
        layers_pos.iter().map(|f| f.2).max().unwrap(),
        layers_pos.iter().map(|f| f.3).max().unwrap(),
    );
    dbg!(area);
    let width = area.2 - area.0;
    let height = area.3 - area.1;
    let mut tiles: Vec<Tile> = Vec::with_capacity(((width) * (area.3 - area.1)) as usize);

    for y in area.1..=area.3 {
        dbg!(y);
        for x in area.0..area.2 {
            let mut tile = Tile {
                ..Default::default()
            };
            for (chunks, layer) in layers.iter() {
                if let Some(chunk) =
                    chunks.get(&(((x / 16) * 16 * x.signum()), ((y / 16) * 16 * x.signum())))
                {
                    let id = chunk[(y * 16 + x % 16).max(0) as usize] as usize;

                    let world_pos = vec2((x - area.0) as f32, (y - area.1) as f32) * TILE_SIZE;
                    if id != 0 {
                        match id {
                            0..20 => {
                                tile.collision = true;
                                tile.visual.push(VisualData::ID(id));
                            }
                            60..80 => {
                                let map_animation = match id {
                                    62 => (20.0, &ASSETS.laughing_man),
                                    _ => unreachable!(),
                                };
                                special_data.map_animations.push(MapAnimation {
                                    pos: world_pos,
                                    clock: 0.0,
                                    turn_off_value: 5.0,
                                    turn_on_value: 20.0,
                                    inactive: ASSETS.laughing_man.get("inactive"),
                                    active: ASSETS.laughing_man.get("active"),
                                    turn_off: ASSETS.laughing_man.get("turn_off"),
                                });
                            }
                            80..100 => match id {
                                81 => {
                                    tile.particle_generator = Some(ParticleGenerator::new(
                                        world_pos,
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
                                _ => panic!(),
                            },
                            140..160 => {
                                // enemies
                                dbg!(id);
                                let enemy = *ENEMY_IDS.get(&id).unwrap();
                                dbg!(enemy, world_pos, id);
                                special_data.enemies.push((enemy, world_pos));
                            }
                            160..180 => {
                                // enemies with tiles

                                let enemy = *ENEMY_IDS.get(&id).unwrap();
                                dbg!(enemy, world_pos, id);
                                special_data.enemies.push((enemy, world_pos));

                                tile.visual.push(VisualData::ID(id));
                            }

                            221 => {
                                special_data.spawn_location = world_pos;
                            }
                            340..360 => {
                                let pickup = match id {
                                    341 => Pickup {
                                        pickup_effect: PickupEffects::Win,
                                        origin: world_pos,
                                        size: ASSETS.lemon_pickup.get_size(),
                                        animation: &ASSETS.lemon_pickup,
                                    },
                                    _ => panic!(),
                                };
                                special_data.pickups.push(pickup);
                            }
                            _ => {
                                tile.visual.push(VisualData::ID(id));
                            }
                        }
                    }
                }
            }
            tiles.push(tile);
        }
    }
    ((tiles, (width) as usize), special_data, tile_set_width)
}

#[derive(Clone, Copy)]
pub enum Levels {
    TestLevel,
}
pub struct MapAnimation {
    pos: Vec2,
    clock: f32,
    turn_off_value: f32,
    turn_on_value: f32,
    inactive: &'static Animation,
    active: &'static Animation,
    turn_off: &'static Animation,
}
impl MapAnimation {
    fn update(&mut self) {
        self.clock += get_frame_time();
        if self.clock > self.turn_on_value {
            self.clock = 0.0;
            self.inactive.play(self.pos, None);
        } else if self.clock > self.turn_off_value + self.turn_off.1 as f32 / 1000.0 {
            self.inactive.play(self.pos, None);
        } else if self.clock > self.turn_off_value {
            self.turn_off
                .play_with_clock(self.pos, self.clock - self.turn_off_value, None);
        } else {
            self.active.play(self.pos, None);
        }
    }
}
pub fn update_map_animations(animations: &mut Vec<MapAnimation>) {
    for animation in animations.iter_mut() {
        animation.update();
    }
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
    pub map_animations: Vec<MapAnimation>,
    pub pickups: Vec<Pickup>,
}
pub struct Level {
    pub tiles: Vec<Tile>,
    pub width: usize,
    pub world_size: Vec2,
    tileset_width: usize,
}
impl Level {
    pub fn new(level: Levels) -> (Self, SpecialData) {
        let data = match level {
            Levels::TestLevel => include_str!("../assets/testlvl.tmx"),
        };
        let (map, special_data, tileset_width) =
            load_tilemap(data, include_str!("../assets/tileset.tsx"));
        let height = map.0.len() as f32 / map.1 as f32;
        (
            Self {
                tileset_width,
                tiles: map.0,
                width: map.1,
                world_size: vec2(map.1 as f32 * TILE_SIZE, height * TILE_SIZE),
            },
            special_data,
        )
    }
    fn draw(&self, tile_data: &VisualData, index: usize) {
        let pos = vec2(
            (index % self.width) as f32 * TILE_SIZE,
            (index / self.width) as f32 * TILE_SIZE,
        );
        match tile_data {
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
    pub fn draw_level(&self) {
        for (index, tile) in self.tiles.iter().enumerate() {
            for tile_data in tile.visual.iter() {
                self.draw(tile_data, index);
            }
            draw_rectangle(-400.0, -400.0, self.world_size.x + 400.0, 400.0, BLACK);
            draw_rectangle(
                -400.0,
                self.world_size.y,
                self.world_size.x + 400.0,
                400.0,
                BLACK,
            );
        }
    }
}
