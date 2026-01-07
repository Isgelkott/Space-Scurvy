use crate::{
    assets::ASSETS,
    enemies::{ENEMY_IDS, Enemy, PresetEnemies},
    particles::ParticleGenerator,
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
            _ => panic!("no layer named {}", input),
        }
    }
}
pub enum TileData {
    SpritesheetCoord((u8, u8)),
    Animation(&'static Animation),
}
pub struct Tile {
    pub data: Vec<(Layer, TileData)>,
    pub particle_generator: Option<ParticleGenerator>,
}

pub fn load_tilemap(tilemap: &str, tileset: &str) -> ((Vec<Tile>, u32), SpecialData) {
    let mut special_data = SpecialData::default();
    let tile_set_width = tileset
        .split_once("columns=\"")
        .unwrap()
        .1
        .split_once("\"")
        .unwrap()
        .0
        .parse::<u8>()
        .unwrap();
    dbg!(tile_set_width);
    fn get_area(chunks: &HashMap<(i32, i32), [u8; 256]>) -> Option<(i32, i32, i32, i32)> {
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
    let mut layers: Vec<(HashMap<(i32, i32), [u8; 256]>, Layer)> = Vec::new();
    for layer in tilemap.split("<layer").skip(1) {
        let name = layer
            .split_once("name=\"")
            .unwrap()
            .1
            .split_once("\"")
            .unwrap()
            .0;
        dbg!(name);
        let mut chunks: HashMap<(i32, i32), [u8; 256]> = HashMap::new();
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

                data[index] = id.parse::<u8>().unwrap();
            }
            if data.iter().all(|f| *f == 0) {
                println!("chunk x: {},y: {} is empty ", x, y);
                continue;
            } else {
                println!("chunk is full of juice x: {}y:{}", x, y)
            }

            chunks.insert((x, y), data);
        }
        layers.push((chunks, Layer::from_str(name)));
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
        for x in area.0..=area.2 {
            let mut tile = Tile {
                data: vec![],
                particle_generator: None,
            };
            for (chunks, layer) in layers.iter() {
                if let Some(chunk) = chunks.get(&(((x / 16) * 16), ((y / 16) * 16))) {
                    let id = chunk[(y % 16 * 16 + x % 16).max(0) as usize];

                    let world_pos = vec2((x - area.0) as f32, (y - area.1) as f32) * TILE_SIZE;

                    if id != 0 {
                        match id {
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
                                    inactive: &map_animation.1.0,
                                    active: &map_animation.1.1,
                                    turn_off: &map_animation.1.2,
                                });
                            }
                            80..100 => match id {
                                80 => {
                                    tile.particle_generator = Some(ParticleGenerator::new(
                                        world_pos,
                                        crate::particles::ParticleType::Acid,
                                    ));
                                    tile.data.push((*layer, TileData::Animation(&ASSETS.acid)));
                                }
                                _ => panic!(),
                            },
                            140..160 => {
                                // enemies
                                let enemy = *ENEMY_IDS.get(&id).unwrap();
                                dbg!(enemy, world_pos, id);
                                special_data.enemies.push((enemy, world_pos));
                            }
                            160..180 => {
                                // enemies with tiles

                                let enemy = *ENEMY_IDS.get(&id).unwrap();
                                dbg!(enemy, world_pos, id);
                                special_data.enemies.push((enemy, world_pos));
                                let id = id - 1;
                                tile.data.push((
                                    *layer,
                                    TileData::SpritesheetCoord((
                                        id % tile_set_width,
                                        id / tile_set_width,
                                    )),
                                ));
                            }
                            221 => {
                                special_data.spawn_location = world_pos;
                            }
                            _ => {
                                let id = id - 1;
                                tile.data.push((
                                    *layer,
                                    TileData::SpritesheetCoord((
                                        id % tile_set_width,
                                        id / tile_set_width,
                                    )),
                                ));
                            }
                        }
                    }
                }
            }
            tiles.push(tile);
        }
    }
    ((tiles, (width + 1) as u32), special_data)
}

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
#[derive(Default)]
pub struct SpecialData {
    pub spawn_location: Vec2,
    pub enemies: Vec<(PresetEnemies, Vec2)>,
    pub map_animations: Vec<MapAnimation>,
}
pub struct Level {
    pub tiles: Vec<Tile>,
    pub width: u32,
    pub world_size: Vec2,
}
impl Level {
    pub fn new(level: Levels) -> (Self, SpecialData) {
        let data = match level {
            Levels::TestLevel => include_str!("../assets/testlvl.tmx"),
        };
        let (map, special_data) = load_tilemap(data, include_str!("../assets/tileset.tsx"));
        let height = map.0.len() as f32 / map.1 as f32;
        (
            Self {
                tiles: map.0,
                width: map.1,
                world_size: vec2(map.1 as f32 * TILE_SIZE, height * TILE_SIZE),
            },
            special_data,
        )
    }
    fn draw(&self, tile_data: &TileData, index: u32) {
        let pos = vec2(
            (index % self.width) as f32 * TILE_SIZE,
            (index / self.width) as f32 * TILE_SIZE,
        );
        match tile_data {
            TileData::Animation(animation) => {
                animation.play(
                    pos,
                    Some(DrawTextureParams {
                        dest_size: Some(vec2(TILE_SIZE, TILE_SIZE)),
                        ..Default::default()
                    }),
                );
            }
            TileData::SpritesheetCoord(spritesheet_coords) => {
                ASSETS.spritesheet.draw_from(
                    *spritesheet_coords,
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
            let index = index as u32;
            for (layer, tile_data) in tile.data.iter() {
                if *layer != Layer::Path && *layer != Layer::OverPlayer {
                    self.draw(tile_data, index);
                }
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
    pub fn draw_foreground(&self) {
        for (index, tile) in self.tiles.iter().enumerate() {
            let index = index as u32;
            for (layer, tile_data) in tile.data.iter() {
                if *layer == Layer::OverPlayer {
                    self.draw(tile_data, index);
                }
            }
        }
    }
}
