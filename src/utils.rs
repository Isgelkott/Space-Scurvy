use crate::level::{Layer, Level, MAP_SCALE_FACTOR, TILE_SIZE};
use asefile::*;
use image::*;
use macroquad::{
    miniquad::{BlendFactor, BlendState, BlendValue, Equation},
    prelude::*,
};
use std::sync::LazyLock;
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
pub fn load_animation(data: &[u8]) -> (Vec<(Texture2D, u32)>, u32) {
    let file = AsepriteFile::read(data).unwrap();
    let mut frames = Vec::new();
    let mut duration = 0;
    for frame in 0..file.num_frames() {
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
pub fn check_collision(pos: Vec2, map: &Level) -> bool {
    let map_pos = pos / (TILE_SIZE * MAP_SCALE_FACTOR);
    if map_pos.y as usize * map.width as usize + map_pos.x as usize > map.tiles.len() - 1 {
        return false;
    }
    let pottential_collider =
        &map.tiles[map_pos.y as usize * map.width as usize + map_pos.x as usize];
    if pottential_collider
        .data
        .iter()
        .any(|f| f.0 == Layer::Collision)
    {
        true
    } else {
        false
    }
}
pub struct Spritesheet {
    spritesheet: Texture2D,
    size: (f32, f32),
}
impl Spritesheet {
    pub fn draw_from(&self, coord: (u8, u8), pos: Vec2, params: Option<DrawTextureParams>) {
        let mut params = params.unwrap_or_default();
        params.source = Some(Rect {
            x: coord.0 as f32 * self.size.0,
            y: coord.1 as f32 * self.size.1,
            w: self.size.0,
            h: self.size.1,
        });
        draw_texture_ex(&self.spritesheet, pos.x, pos.y, WHITE, params);
    }
    pub fn new(size: (f32, f32), texture: Texture2D) -> Self {
        Self {
            spritesheet: texture,
            size,
        }
    }
}
pub type Animation = (Vec<(Texture2D, u32)>, u32);
const DEFAULT_FRAGMENT_SHADER: &'static str = "#version 100
precision lowp float;

varying vec2 uv;

uniform sampler2D Texture;

void main() {
    gl_FragColor = texture2D(Texture, uv);
}
";
const BULLET_SHADER: &'static str = "#version 100
precision lowp float;

uniform lowp float alpha; 
varying vec2 uv;
void main() {
    gl_FragColor = vec4(0.0,0.0,0.0,alpha*uv.x*100.0);
}
";
pub static BULLET_MATERIAL: LazyLock<Material> = std::sync::LazyLock::new(|| {
    let pipeline = PipelineParams {
        alpha_blend: Some(BlendState::new(
            Equation::Add,
            BlendFactor::Value(BlendValue::SourceAlpha),
            BlendFactor::OneMinusValue(BlendValue::SourceAlpha),
        )),
        color_blend: Some(BlendState::new(
            Equation::Add,
            BlendFactor::Value(BlendValue::SourceAlpha),
            BlendFactor::OneMinusValue(BlendValue::SourceAlpha),
        )),
        ..Default::default()
    };
    load_material(
        ShaderSource::Glsl {
            vertex: DEFAULT_VERTEX_SHADER,
            fragment: BULLET_SHADER,
        },
        MaterialParams {
            pipeline_params: pipeline,
            uniforms: vec![UniformDesc::new("alpha", UniformType::Float1)],
            ..Default::default()
        },
    )
    .unwrap()
});

const DEFAULT_VERTEX_SHADER: &'static str = "#version 100
precision lowp float;

attribute vec3 position;
attribute vec2 texcoord;

varying vec2 uv;

uniform mat4 Model;
uniform mat4 Projection;

void main() {
    gl_Position = Projection * Model * vec4(position, 1);
    uv = texcoord;
}
";

pub static HP_MATERIAL: LazyLock<Material> = std::sync::LazyLock::new(|| {
    let pipeline = PipelineParams {
        alpha_blend: Some(BlendState::new(
            Equation::Add,
            BlendFactor::Value(BlendValue::SourceAlpha),
            BlendFactor::OneMinusValue(BlendValue::SourceAlpha),
        )),
        color_blend: Some(BlendState::new(
            Equation::Add,
            BlendFactor::Value(BlendValue::SourceAlpha),
            BlendFactor::OneMinusValue(BlendValue::SourceAlpha),
        )),
        ..Default::default()
    };
    load_material(
        ShaderSource::Glsl {
            vertex: DEFAULT_VERTEX_SHADER,
            fragment: HP_SHADER,
        },
        MaterialParams {
            pipeline_params: pipeline,
            uniforms: vec![UniformDesc::new("black_x", UniformType::Float1)],
            ..Default::default()
        },
    )
    .unwrap()
});
const IFRAMES_SHADER: &'static str = "#version 100
precision lowp float;

varying vec2 uv;
uniform sampler2D Texture;

void main() {
 
    gl_FragColor = texture2D(Texture, uv) + vec4(0.5,0.5,0.5,0.0);
}
";
pub static IFRAMES_MATERIAL: LazyLock<Material> = LazyLock::new(|| {
    let pipeline = PipelineParams {
        alpha_blend: Some(BlendState::new(
            Equation::Add,
            BlendFactor::Value(BlendValue::SourceAlpha),
            BlendFactor::OneMinusValue(BlendValue::SourceAlpha),
        )),
        color_blend: Some(BlendState::new(
            Equation::Add,
            BlendFactor::Value(BlendValue::SourceAlpha),
            BlendFactor::OneMinusValue(BlendValue::SourceAlpha),
        )),
        ..Default::default()
    };
    load_material(
        ShaderSource::Glsl {
            vertex: DEFAULT_VERTEX_SHADER,
            fragment: IFRAMES_SHADER,
        },
        MaterialParams {
            pipeline_params: pipeline,
            ..Default::default()
        },
    )
    .unwrap()
});
const HP_SHADER: &'static str = "#version 100
precision lowp float;

varying vec2 uv;
uniform lowp float black_x; 
uniform sampler2D Texture;

void main() {
 if (texture2D(Texture, uv).a != 0.0){
 if (uv.x < black_x) {
        gl_FragColor = texture2D(Texture, uv);
    }  else{
      gl_FragColor = vec4(0.0,0.0,0.0,1.0); 
        
    }
    
}else{
gl_FragColor = vec4(0.0,0.0,0.0,0.0); 
  
}}
";
pub trait AnimationMethods {
    fn play(&self, pos: Vec2, params: Option<DrawTextureParams>);
    fn play_with_clock(&self, pos: Vec2, clock: f32, params: Option<DrawTextureParams>);
    fn get_size(&self) -> Vec2;
}
impl AnimationMethods for Animation {
    fn get_size(&self) -> Vec2 {
        vec2(self.0[0].0.width(), self.0[0].0.height())
    }
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
    fn play_with_clock(&self, pos: Vec2, clock: f32, params: Option<DrawTextureParams>) {
        let wa = (clock * 1000.0) as u32;

        let mut frame = wa;
        for i in self.0.iter() {
            if frame > i.1 {
                frame -= i.1
            } else {
                draw_texture_ex(&i.0, pos.x, pos.y, WHITE, params.unwrap_or_default());

                break;
            }
        }
    }
}
