use asefile::*;
use image::*;
use macroquad::prelude::*;

use crate::level::{MAP_SCALE_FACTOR, TILE_SIZE};
pub fn load_ase_texture(bytes: &[u8], layer: Option<u32>, frame: Option<u32>) -> Texture2D {
    let img = AsepriteFile::read(bytes).unwrap();
    let frame = frame.unwrap_or(0);
    let img = if let Some(layer) = layer {
        img.layer(layer).frame(frame).image()
    } else {
        img.frame(0).image()
    };

    let new = Image {
        width: img.width() as u16,
        height: img.height() as u16,
        bytes: img.as_bytes().to_vec(),
    };
    let texture = Texture2D::from_image(&new);
    texture.set_filter(FilterMode::Nearest);
    texture
}
pub fn create_camera(dimensions: Vec2) -> Camera2D {
    let rt = render_target(dimensions.y as u32, dimensions.y as u32);
    rt.texture.set_filter(FilterMode::Nearest);

    Camera2D {
        render_target: Some(rt),
        zoom: Vec2::new(1.0 / dimensions.x * 2.0, 1.0 / dimensions.y * 2.0),
        target: vec2((dimensions.x / 2.0).floor(), (dimensions.y / 2.0).floor()),
        ..Default::default()
    }
}
pub fn to_map_pos(pos: Vec2, map_width: usize) -> usize {
    let map_pos = pos / (TILE_SIZE * MAP_SCALE_FACTOR);
    map_pos.y as usize * map_width as usize + map_pos.x as usize
}

pub fn load_animation_from_tag(data: &[u8], tag: &str) -> (Vec<(Texture2D, u32)>, u32) {
    let file = AsepriteFile::read(data).unwrap();
    dbg!(tag);
    let tag = file.tag_by_name(tag).unwrap();
    let start = tag.from_frame();
    let end = tag.to_frame();
    let mut frames = Vec::new();
    let mut duration = 0;
    for frame in start..=end {
        let img = file.frame(frame);
        let time = img.duration();
        duration += time;
        let img = img.image();
        let texture = Texture2D::from_image(&Image {
            width: img.width() as u16,
            height: img.height() as u16,
            bytes: img.as_bytes().to_vec(),
        });
        texture.set_filter(FilterMode::Nearest);
        frames.push((texture, time));
    }
    (frames, duration)
}
pub struct Spritesheet {
    texture: Texture2D,
    widht: f32,
    height: f32,
}
impl Spritesheet {
    pub fn draw_from(&self, world_pos: Vec2, texture_coord: (u8, u8), scale: f32) {
        draw_texture_ex(
            &self.texture,
            world_pos.x,
            world_pos.y,
            WHITE,
            DrawTextureParams {
                source: Some(Rect {
                    x: texture_coord.0 as f32 * self.widht,
                    y: texture_coord.1 as f32 * self.height,
                    w: self.widht,
                    h: self.height,
                }),
                dest_size: Some(vec2(self.widht, self.height) * scale),
                ..Default::default()
            },
        )
    }
}
pub type Animation = (Vec<(Texture2D, u32)>, u32);
pub trait Play {
    fn play(&self, pos: Vec2, params: Option<DrawTextureParams>);
    fn play_with_clock(
        &self,
        pos: Vec2,
        params: Option<DrawTextureParams>,
        clock: &mut f32,
    ) -> bool;
}
impl Play for Animation {
    fn play(&self, pos: Vec2, params: Option<DrawTextureParams>) {
        let mut time = (get_time() * 1000.0) % self.1 as f64;
        for i in &self.0 {
            if time <= i.1 as f64 {
                draw_texture_ex(&i.0, pos.x, pos.y, WHITE, params.unwrap_or_default());
                break;
            } else {
                time -= i.1 as f64;
            }
        }
    }
    fn play_with_clock(
        &self,
        pos: Vec2,
        params: Option<DrawTextureParams>,
        clock: &mut f32,
    ) -> bool {
        let wa = (*clock * 1000.0) as u32;
        if wa < self.1 {
            let mut frame = wa;
            for i in self.0.iter() {
                if frame > i.1 {
                    frame -= i.1
                } else {
                    draw_texture_ex(&i.0, pos.x, pos.y, WHITE, params.unwrap_or_default());

                    break;
                }
            }
            *clock += get_frame_time();
            return true;
        } else {
            return false;
        }
    }
}
