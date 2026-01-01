use crate::{
    assets::ASSETS,
    enemies::{ENEMY_IDS, Enemy, PresetEnemies},
};
use macroquad::prelude::*;
use std::{collections::HashMap, sync::LazyLock};
pub const TILE_SIZE: f32 = 16.0;
pub const MAP_SCALE_FACTOR: f32 = 1.0;
#[derive(PartialEq)]
pub enum Layer {
    Collision,
    Decor,
    Enemies,
    Special,
    Path,
}
impl Layer {
    fn from_str(input: &str) -> Self {
        match input {
            "collision" => Self::Collision,
            input if input.contains("decor") => Self::Decor,
            "enemies" => Self::Enemies,
            "special" => Self::Special,
            "path" => Self::Path,
            _ => panic!("no layer named {}", input),
        }
    }
}
pub struct Tile {
    pub data: Vec<(Layer, (u8, u8))>,
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
    let mut layers: Vec<(HashMap<(i32, i32), [u8; 256]>, &str)> = Vec::new();
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
        layers.push((chunks, name));
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
        for x in area.0..=area.2 {
            let mut tile = Tile { data: vec![] };
            for (chunks, name) in layers.iter() {
                if let Some(chunk) = chunks.get(&(
                    ((x as f32 / TILE_SIZE).floor() * TILE_SIZE) as i32,
                    ((y as f32 / TILE_SIZE).floor() * TILE_SIZE) as i32,
                )) {
                    let id = chunk[(y % 16 * 16 + x % 16).max(0) as usize];
                    let world_pos = vec2(x as f32, y as f32) * MAP_SCALE_FACTOR * TILE_SIZE;
                    if id != 0 {
                        let id = id - 1;
                        match id {
                            140..160 => {
                                println!("found enemy, at {}", world_pos);
                                let enemy = *ENEMY_IDS.get(&id).unwrap();
                                special_data.enemies.push((enemy, world_pos));
                            }
                            220 => {
                                special_data.spawn_location = world_pos;
                            }
                            _ => {
                                tile.data.push((
                                    Layer::from_str(*name),
                                    (id % tile_set_width, id / tile_set_width),
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

#[derive(Default)]
pub struct SpecialData {
    pub spawn_location: Vec2,
    pub enemies: Vec<(PresetEnemies, Vec2)>,
}
pub struct Level {
    pub tiles: Vec<Tile>,
    pub width: u32,
}
impl Level {
    pub fn new(level: Levels) -> (Self, SpecialData) {
        let data = match level {
            Levels::TestLevel => include_str!("../assets/testlvl.tmx"),
        };
        let (map, special_data) = load_tilemap(data, include_str!("../assets/tileset.tsx"));
        (
            Self {
                tiles: map.0,
                width: map.1,
            },
            special_data,
        )
    }
    pub fn draw(&self) {
        for (index, tile) in self.tiles.iter().enumerate() {
            let index = index as u32;
            for (layer, texture_coord) in tile.data.iter() {
                if *layer != Layer::Path {
                    ASSETS.spritesheet.draw_from(
                        *texture_coord,
                        vec2(
                            (index % self.width) as f32 * TILE_SIZE * MAP_SCALE_FACTOR,
                            (index / self.width) as f32 * TILE_SIZE * MAP_SCALE_FACTOR,
                        ),
                        Some(DrawTextureParams {
                            dest_size: Some(vec2(
                                TILE_SIZE * MAP_SCALE_FACTOR,
                                TILE_SIZE * MAP_SCALE_FACTOR,
                            )),
                            ..Default::default()
                        }),
                    );
                }
            }
        }
    }
}
