use macroquad::prelude::*;

use crate::{
    assets::ASSETS,
    level::Level,
    particles::{self, Particle, Particles},
    player::{DeathCause, Player},
    utils::AnimationMethods,
};

pub enum Projectiles {
    Rocket,
    EnergyBall,
}
pub enum ProjectileBehaviour {
    Constant,
    FollowPlayer,
    ShootAtPlayer,
    Bullet,
}
#[derive(Debug)]
enum Collision {
    Player,
    Map,
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
    pub lifetime: Option<f32>,
}
impl Projectile {
    pub fn update(
        &mut self,
        player: &mut Player,
        frame_time: f32,
        level: &Level,
        particles: &mut Vec<Particle>,
    ) -> bool {
        if let Some(duration) = &mut self.lifetime {
            *duration -= frame_time;
            if duration.is_sign_negative() {
                particles.push(Particle::preset(Particles::Explosion, self.pos));

                return false;
            }
        }
        (self.draw)(
            self.pos,
            self.size,
            (player.pos - self.pos).normalize().to_angle(),
        );

        match &self.behaviour {
            ProjectileBehaviour::Constant => {
                self.pos += self.direction * self.speed * frame_time;
            }
            ProjectileBehaviour::FollowPlayer => {
                self.pos += (player.pos - self.pos).normalize() * self.speed * frame_time;
            }
            _ => panic!(),
        };
        let collision = self.check_collision(player, level);
        if let Some(collision) = &collision {
            particles.push(Particle::preset(Particles::EnergyBallShatter, self.pos));
            if let Collision::Player = collision {
                player.damage(self.damage, self.death_cause);
            }
        }
        return collision.is_none();
    }
    fn check_collision(&self, player: &Player, level: &Level) -> Option<Collision> {
        if ((self.pos.x > player.pos.x && self.pos.x < player.pos.x + player.size.x)
            || (self.pos.x + self.size.x > player.pos.x)
                && self.pos.x + self.size.x < player.pos.x + player.size.x)
            && ((self.pos.y > player.pos.y && self.pos.y < player.pos.y + player.size.y)
                || (self.pos.y + self.size.y > player.pos.y
                    && self.pos.y + self.size.y < player.pos.y + player.size.y))
        {
            return Some(Collision::Player);
        }
        for i in 0..2 {
            let x = self.pos.x + i as f32 * self.size.x;
            for j in 0..2 {
                let y = self.pos.y + j as f32 * self.size.y;
                if let Some(tile) = level.get_tile(vec2(x, y))
                    && tile.collision
                {
                    return Some(Collision::Map);
                }
            }
        }
        return None;
    }
    pub fn from(pos: Vec2, projectile: Projectiles, direction: Vec2) -> Self {
        match projectile {
            Projectiles::Rocket => Projectile {
                lifetime: Some(5.),
                particle: Some(Particles::Explosion),
                behaviour: ProjectileBehaviour::FollowPlayer,
                pos,
                size: ASSETS.rocket.size(),
                damage: 200,
                direction,
                speed: 30.0,

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
                lifetime: None,
                pos,
                size: ASSETS.energy_ball.size(),
                damage: 20,
                draw: Box::new(|pos, size, rotation| {
                    ASSETS.energy_ball.play(pos, None);
                }),
                speed: 40.0,
                direction,
                behaviour: ProjectileBehaviour::Constant,
                death_cause: DeathCause::Energy,
                particle: Some(Particles::EnergyBallShatter),
            },
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
