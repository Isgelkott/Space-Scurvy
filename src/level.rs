fn load_tilemap(tilemap: &str, tileset: &str) -> (Vec<Tile>, u32) {
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
    fn get_area(chunks: &HashMap<(i32, i32), [u8; 256]>) -> (i32, i32, i32, i32) {
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
        let lowest_x = posses.iter().map(|f| f.0).min().unwrap_or(posses[0].0);
        let highest_x = posses.iter().map(|f| f.2).max().unwrap();
        let lowest_y = posses.iter().map(|f| f.1).min().unwrap_or(posses[0].1);
        let highest_y = posses.iter().map(|f| f.3).max().unwrap_or(posses[0].3);

        (lowest_x, lowest_y, highest_x, highest_y)
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
    let layers_pos: Vec<(i32, i32, i32, i32)> = layers.iter().map(|f| get_area(&f.0)).collect();
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
            let mut tile = Tile {
                textures: vec![],
                collision: false,
            };
            for (chunks, name) in layers.iter() {
                if let Some(chunk) = chunks.get(&(
                    ((x as f32 / 16.0).floor() * 16.0) as i32,
                    ((y as f32 / 16.0).floor() * 16.0) as i32,
                )) {
                    let id = chunk[(y % 16 * 16 + x % 16) as usize];

                    if id != 0 {
                        let id = id - 1;
                        if name.contains("collision") {
                            tile.collision = true;
                        }
                        tile.textures
                            .push((id % tile_set_width, id / tile_set_width));
                    }
                }
            }
            tiles.push(tile);
        }
    }
    (tiles, (area.2 + 1 - area.0) as u32)
}

struct Level {}
impl Level {
    fn new(level_data: &str) -> Self {
        load
    }
}
