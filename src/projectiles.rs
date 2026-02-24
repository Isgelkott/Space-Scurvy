use macroquad::prelude::*;

use crate::{assets::ASSETS, particles::Particles, player::DeathCause, utils::AnimationMethods};

pub enum Projectiles {
    Rocket,
    EnergyBall,
    Bullet,
}
pub enum ProjectileBehaviour {
    Static,
    FollowPlayer,
    ShootAtPlayer,
    Bullet,
}
pub struct Projectile {
    pub pos: Vec2,
    pub size: Vec2,
    pub direction: Vec2,
    pub speed: f32,
    pub draw: Box<dyn Fn(Vec2, Vec2, f32)>, // pos, size, rotation
    pub behaviour: ProjectileBehaviour,
    pub damage: u32,
    pub death_cause: DeathCause,
    pub particle: Option<Particles>,
}
impl Projectile {
    pub fn from(pos: Vec2, projectile: Projectiles, direction: Option<Vec2>) -> Self {
        let direction = direction.unwrap_or(Vec2::ZERO);
        match projectile {
            Projectiles::Rocket => Projectile {
                particle: Some(Particles::Explosion),
                behaviour: ProjectileBehaviour::FollowPlayer,
                pos,
                size: ASSETS.rocket.get_size(),
                damage: 200,
                direction,
                speed: 120.0,
                draw: Box::new(|pos, size, rotation| {
                    ASSETS.rocket.get("fly").play(
                        pos,
                        Some(DrawTextureParams {
                            rotation,
                            pivot: Some(pos + size / 2.0),
                            ..Default::default()
                        }),
                    )
                }),
                death_cause: DeathCause::Explode,
            },
            Projectiles::EnergyBall => Self {
                pos,
                size: ASSETS.energy_ball.get_size(),
                damage: 20,
                draw: Box::new(|pos, size, rotation| {
                    ASSETS.energy_ball.play(pos, None);
                }),
                speed: 40.0,
                direction,
                behaviour: ProjectileBehaviour::FollowPlayer,
                death_cause: DeathCause::Energy,
                particle: Some(Particles::EnergyBallShatter),
            },
            Projectiles::Bullet => {
                //   if !params.flip_x {
                //                 self.size.x - 2.0
                //             } else {
                //                 2.0
                //             },
                //             14.0,
                //         ),
                //    ,
                panic!()
                // Self {
                //     pos,
                //     size: 2.0,
                //     direction,
                //     speed: 20.0,
                //     draw: (),
                //     behaviour: ProjectileBehaviour::Bullet,
                //     damage: (),
                //     death_cause: (),
                //     particle: (),
                // };
            }
        }
    }
}
// pub struct Bullet {
//     pos: Vec2,
//     direction: Vec2,
//     speed: f32,
//     origin: Vec2,
// }

// fn update(&mut self, _player: &mut Player, map: &Level, frame_time: f32) {
//     self.pos += self.direction * self.speed * frame_time;
//     gl_use_material(&BULLET_MATERIAL);
//     BULLET_MATERIAL.set_uniform("alpha", 1.0 / (self.pos.x - self.origin.x).abs().powf(1.5));
//     draw_rectangle(
//         self.origin.x,
//         self.origin.y,
//         self.pos.x - self.origin.x,
//         2.0,
//         BLACK,
//     );
//     gl_use_default_material();
// }
