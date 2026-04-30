use crate::level::{Level, TILE_SIZE, Tile};
use asefile::*;
use image::*;
use macroquad::{
    miniquad::{BlendFactor, BlendState, BlendValue, Equation},
    prelude::*,
};
use std::{collections::HashMap, sync::LazyLock};
#[derive(Default)]
pub struct DebugFlags {
    pub show_path: bool,
    pub print_specials: bool,
    pub still: bool,
    pub speed: Option<f32>,
    pub show_collisions: bool,
}
pub static DEBUG_FLAGS: LazyLock<DebugFlags> = LazyLock::new(|| {
    let mut flags = DebugFlags::default();
    let mut iter = std::env::args().into_iter();
    while let Some(arg) = iter.next() {
        let arg = &arg as &str;
        match arg {
            "showpath" => flags.show_path = true,
            "specials" => flags.print_specials = true,
            "still" => flags.still = true,
            "speed" => flags.speed = Some(iter.next().unwrap().parse::<f32>().unwrap()),
            "coll" => flags.show_collisions = true,
            _ => {
                warn!("unknown flag! {}", arg);
            }
        }
    }
    flags
});
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
    let rt = render_target(dimensions.x as u32, dimensions.y as u32);
    rt.texture.set_filter(FilterMode::Nearest);

    Camera2D {
        render_target: Some(rt),
        zoom: Vec2::new(1.0 / dimensions.x * 2.0, 1.0 / dimensions.y * 2.0),
        target: vec2((dimensions.x / 2.0).floor(), (dimensions.y / 2.0).floor()),
        ..Default::default()
    }
}
pub fn check_collision_rectangle_collision(obj1: (Vec2, Vec2), obj2: (Vec2, Vec2)) -> bool {
    ((obj1.0.x > obj2.0.x && obj1.0.x < obj2.0.x + obj2.1.x)
        || (obj1.0.x + obj1.1.x > obj2.0.x) && obj1.0.x + obj1.1.x < obj2.0.x + obj2.1.x)
        && ((obj1.0.y > obj2.0.y && obj1.0.y < obj2.0.y + obj2.1.y)
            || (obj1.0.y + obj1.1.y > obj2.0.y && obj1.0.y + obj1.1.y < obj2.0.y + obj2.1.y))
}
pub struct Spritesheet {
    spritesheet: Texture2D,
    size: (f32, f32),
}
impl Spritesheet {
    pub fn draw_from(&self, coord: (u16, u16), pos: Vec2, params: Option<DrawTextureParams>) {
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
pub fn load_pixel_map(animation: &Animation, color: [u8; 4]) -> Vec<(f32, Vec2)> {
    let mut data = Vec::new();
    for (frame, duiration) in animation.0.iter() {
        let width = frame.width();
        let heigt = frame.height();
        for (index, pixels) in frame
            .get_texture_data()
            .bytes
            .windows(4)
            .step_by(4)
            .enumerate()
        {
            if pixels == &color {
                data.push((
                    *duiration as f32 / 1000.0,
                    vec2(
                        (index % width as usize) as f32,
                        (index / width as usize) as f32,
                    ),
                ));
            }
        }
    }

    data
}
pub fn load_animation_group(data: &[u8]) -> AnimationGroup {
    let file = AsepriteFile::read(data).unwrap();
    let mut map = HashMap::new();
    let mut name;
    if file.num_tags() == 0 {
        name = "base".to_string();

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
        map.insert(name, (frames, duration));
    } else {
        for i in 0..file.num_tags() {
            let tag = file.get_tag(i).unwrap();
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
            name = tag.name().to_string();
            map.insert(name, (frames, duration));
        }
    }
    AnimationGroup(map)
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

pub fn load_animation_by_layer(data: &[u8]) -> AnimationGroup {
    let file = AsepriteFile::read(data).unwrap();
    let mut layers: HashMap<String, (Vec<(Texture2D, u32)>, u32)> = HashMap::new();
    for i in 0..file.num_layers() {
        let mut frames = Vec::new();
        let mut duration = 0;
        let name = file.layer(i).name().to_string();
        let mut counter = 0;
        loop {
            if counter >= file.num_frames() {
                break;
            }
            if file.layer(i).frame(counter).is_empty() {
                break;
            }
            let img = file.frame(counter);
            let time = img.duration();
            duration += time;
            let img = img.layer(i).image();
            let texture = Texture2D::from_image(&Image {
                width: img.width() as u16,
                height: img.height() as u16,
                bytes: img.as_bytes().to_vec(),
            });
            texture.set_filter(FilterMode::Nearest);
            frames.push((texture, time));
            counter += 1;
        }
        layers.insert(name, (frames, duration));
    }
    AnimationGroup(layers)
}

pub trait DrawTexture {
    fn draw(&self, pos: Vec2, params: Option<DrawTextureParams>);
}
impl DrawTexture for Texture2D {
    fn draw(&self, pos: Vec2, params: Option<DrawTextureParams>) {
        draw_texture_ex(self, pos.x, pos.y, WHITE, params.unwrap_or_default());
    }
}
pub type Animation = (Vec<(Texture2D, u32)>, u32);
pub trait AnimationMethods {
    fn base(&self) -> &Texture2D;
    fn play(&self, pos: Vec2, params: Option<DrawTextureParams>);
    fn play_with_clock(&self, pos: Vec2, clock: f32, params: Option<DrawTextureParams>);
    fn size(&self) -> Vec2;
    fn get_duration(&self) -> f32;
    fn draw_index(&self, pos: Vec2, index: usize, params: Option<DrawTextureParams>);
}
impl AnimationMethods for Animation {
    fn base(&self) -> &Texture2D {
        &self.0[0].0
    }
    fn draw_index(&self, pos: Vec2, index: usize, params: Option<DrawTextureParams>) {
        self.0[index].0.draw(pos, params);
    }
    fn get_duration(&self) -> f32 {
        self.1 as f32 / 1000.0
    }
    fn size(&self) -> Vec2 {
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
        let mut indexus = self.0.len() - 1;
        let mut frame = (clock * 1000.0) as u32;
        for (index, i) in self.0.iter().enumerate() {
            if frame > i.1 {
                frame -= i.1
            } else {
                indexus = index;

                break;
            }
        }
        draw_texture_ex(
            &self.0[indexus].0,
            pos.x,
            pos.y,
            WHITE,
            params.unwrap_or_default(),
        );
    }
}
pub fn mouse_pos() -> Vec2 {
    let mouse_pos = mouse_position();
    return vec2(mouse_pos.0, mouse_pos.1);
}
#[derive(Debug)]
pub struct AnimationGroup(pub HashMap<String, Animation>);
impl AnimationGroup {
    pub fn default(&self) -> &Texture2D {
        &self.get("ref").0[0].0
    }
    pub fn get(&self, key: &str) -> &Animation {
        self.0.get(key).unwrap_or_else(|| {
            dbg!(key);
            panic!()
        })
    }
    pub fn base(&self) -> &Animation {
        self.get("base")
    }
    pub fn size(&self) -> Vec2 {
        self.0.values().next().unwrap().size()
    }
    pub fn play_tag(&self, tag: &str, pos: Vec2, params: Option<DrawTextureParams>) {
        self.get(tag).play(pos, params);
    }
}

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
fn load_shader_material(
    fragment_shader: &str,
    uniforms: Option<Vec<(&str, UniformType)>>,
) -> Material {
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
    let uniforms = if let Some(uniforms) = uniforms {
        uniforms
            .iter()
            .map(|f| UniformDesc::new(f.0, f.1))
            .collect()
    } else {
        vec![]
    };
    load_material(
        ShaderSource::Glsl {
            vertex: DEFAULT_VERTEX_SHADER,
            fragment: fragment_shader,
        },
        MaterialParams {
            pipeline_params: pipeline,
            uniforms,
            ..Default::default()
        },
    )
    .unwrap()
}
pub static GRAYSCALE_MAT: LazyLock<Material> = LazyLock::new(|| {
    // stolen from https://godotshaders.com/shader/simple-grayscale-shader-for-canvasitem/ SimranZenov
    load_shader_material(
        "#version 100
precision lowp float;

varying vec2 uv;
uniform sampler2D Texture;



void main() {
  vec4 color = texture2D(Texture, uv);
 float gray =  dot(color.rgb, vec3(0.299, 0.587, 0.114 ));
 vec3 result = mix(vec3(gray), color.rgb, 0.);
    gl_FragColor = vec4(result, color.a);

}

",
        None,
    )
});
const FISH_SHADER: &'static str = "#version 100
precision lowp float;

uniform lowp float acidy;
varying vec2 uv;
uniform sampler2D Texture;



void main() {
    if (uv.y> acidy){
    gl_FragColor = texture2D(Texture, uv);

    }else{
 gl_FragColor = vec4(0.0,0.0,0.0,0.0);

    }
}
";
pub static FISH_MATERIAL: LazyLock<Material> = std::sync::LazyLock::new(|| {
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
            fragment: FISH_SHADER,
        },
        MaterialParams {
            pipeline_params: pipeline,
            uniforms: vec![UniformDesc::new("acidy", UniformType::Float1)],
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
