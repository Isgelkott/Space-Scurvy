use crate::assets::ASSETS;
use macroquad::prelude::*;
use std::collections::HashMap;
pub const TILE_SIZE: f32 = 16.0;
enum Layer {
    Collision,
    Decor,
    Enemies,
    Actuators,
}
impl Layer {
    fn from_str(input: &str) -> Self {
        match input {
            "collision" => Self::Collision,
            "decor" => Self::Decor,
            "enemies" => Self::Enemies,
            "actuators" => Self::Actuators,
            _ => panic!("no layer named {}", input),
        }
    }
}
pub struct Tile {
    textures: Vec<(Layer, (u8, u8))>,
}
pub fn load_tilemap(tilemap: &str, tileset: &str) -> (Vec<Tile>, u32) {
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
    let mut tiles: Vec<Tile> = Vec::with_capacity(((area.2 - area.0) * (area.3 - area.1)) as usize);

    for y in area.1..area.3 + 1 {
        for x in area.0..area.2 + 1 {
            let mut tile = Tile { textures: vec![] };
            for (chunks, name) in layers.iter() {
                if let Some(chunk) = chunks.get(&(
                    ((x as f32 / 16.0).floor() * 16.0) as i32,
                    ((y as f32 / 16.0).floor() * 16.0) as i32,
                )) {
                    let id = chunk[(y % 16 * 16 + x % 16).max(0) as usize];

                    if id != 0 {
                        let id = id - 1;
                        tile.textures.push((
                            Layer::from_str(*name),
                            (id % tile_set_width, id / tile_set_width),
                        ));
                    }
                }
            }
            tiles.push(tile);
        }
    }
    (tiles, (area.2 + 1 - area.0) as u32)
}
pub enum Levels {
    TestLevel,
}
pub struct Level {
    tiles: Vec<Tile>,
    width: u32,
}
impl Level {
    pub fn new(level: Levels) -> Self {
        let data = match level {
            Levels::TestLevel => include_str!("../assets/testlvl.tmx"),
        };
        let tilemap = load_tilemap(data, include_str!("../assets/tileset.tsx"));
        Self {
            tiles: tilemap.0,
            width: tilemap.1,
        }
    }
    pub fn draw(&self) {
        for (index, tile) in self.tiles.iter().enumerate() {
            let index = index as u32;
            for (_, texture_coord) in tile.textures.iter() {
                ASSETS.spritesheet.draw_from(
                    *texture_coord,
                    vec2(
                        (index % self.width) as f32 * 16.0,
                        (index / self.width) as f32 * 16.0,
                    ),
                );
            }
        }
    }
}
